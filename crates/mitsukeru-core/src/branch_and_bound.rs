// src/branch_and_bound.rs

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::{Ordering, Reverse};
use core::mem;

use crate::bitset::BitSet;
use crate::clock::Clock;
use crate::model::{BusinessRule, CandidateSolution, Item, OptimizeError, SolutionStatus};
use crate::validation::{MAX_CAPACITY_DIMS, MAX_TOP_K, validate_problem_input};

#[derive(Clone, Debug)]
struct InternalItem {
    original_id: usize,
    weights: Vec<u32>,
    value: u32,
    surrogate_weight: f64,
    density: f64,
}

struct ExactSolverCore<'a, const W: usize> {
    capacities: &'a [u32],
    sorted_items: Vec<InternalItem>,
    conflicts: Vec<BitSet<W>>,
    requires_closure: Vec<BitSet<W>>,
    best_value: u32,
    best_mask: BitSet<W>,
    explored_nodes: u64,
    max_nodes: Option<u64>,
    orig_to_sorted: BTreeMap<usize, usize>,
    time_limit_ms: Option<f64>,
    clock: Option<&'a dyn Clock>,
    start_micros: u64,
    timed_out: bool,
    root_upper_bound: f64,
    top_k: usize,
    candidates: Vec<CandidateSolution>,
}

impl<'a, const W: usize> ExactSolverCore<'a, W> {
    fn record_candidate(&mut self, val: u32, mask: BitSet<W>) {
        let cap_k = self.top_k.min(MAX_TOP_K);
        if cap_k <= 1 || val == 0 {
            return;
        }

        let mut ids = Vec::new();
        for (s_idx, it) in self.sorted_items.iter().enumerate() {
            if mask.has(s_idx) {
                ids.push(it.original_id);
            }
        }
        ids.sort_unstable();

        if !self.candidates.iter().any(|c| c.selected_item_ids == ids) {
            self.candidates.push(CandidateSolution {
                value: val,
                selected_item_ids: ids,
            });
            self.candidates.sort_by_key(|a| Reverse(a.value));
            if self.candidates.len() > cap_k {
                self.candidates.truncate(cap_k);
            }
        }
    }

    fn new(items: &[Item], capacities: &'a [u32], rules: &[BusinessRule]) -> Self {
        let n = items.len().min(BitSet::<W>::CAPACITY);

        let mut internal_items: Vec<InternalItem> = items
            .iter()
            .take(n)
            .map(|it| {
                let mut surrogate_w = 0.0;
                for (d, &cap) in capacities.iter().enumerate() {
                    let w = it.weights.get(d).copied().unwrap_or(0);
                    if cap > 0 {
                        surrogate_w += f64::from(w) / f64::from(cap);
                    }
                }
                if surrogate_w == 0.0 {
                    surrogate_w = 1e-9;
                }
                let density = f64::from(it.value) / surrogate_w;
                InternalItem {
                    original_id: it.id,
                    weights: it.weights.clone(),
                    value: it.value,
                    surrogate_weight: surrogate_w,
                    density,
                }
            })
            .collect();

        internal_items.sort_by(|a, b| b.density.partial_cmp(&a.density).unwrap_or(Ordering::Equal));

        let mut orig_to_sorted = BTreeMap::new();
        for (sorted_idx, it) in internal_items.iter().enumerate() {
            orig_to_sorted.insert(it.original_id, sorted_idx);
        }

        let mut conflicts = vec![BitSet::<W>::new(); n];
        let mut requires = vec![BitSet::<W>::new(); n];

        for rule in rules {
            match rule {
                BusinessRule::Requires { from_id, to_id } => {
                    if let (Some(&s_from), Some(&s_to)) =
                        (orig_to_sorted.get(from_id), orig_to_sorted.get(to_id))
                    {
                        requires[s_from].set(s_to);
                    }
                }
                BusinessRule::Conflicts { a_id, b_id } => {
                    if let (Some(&s_a), Some(&s_b)) =
                        (orig_to_sorted.get(a_id), orig_to_sorted.get(b_id))
                    {
                        conflicts[s_a].set(s_b);
                        conflicts[s_b].set(s_a);
                    }
                }
            }
        }

        let mut requires_closure = requires;
        for k in 0..n {
            for i in 0..n {
                if requires_closure[i].has(k) {
                    let k_closure = requires_closure[k];
                    requires_closure[i].union_with(&k_closure);
                }
            }
        }

        let mut core = Self {
            capacities,
            sorted_items: internal_items,
            conflicts,
            requires_closure,
            best_value: 0,
            best_mask: BitSet::<W>::new(),
            explored_nodes: 0,
            time_limit_ms: None,
            clock: None,
            start_micros: 0,
            timed_out: false,
            orig_to_sorted,
            max_nodes: None,
            root_upper_bound: 0.0,
            top_k: 1,
            candidates: Vec::new(),
        };

        let num_dims = capacities.len().min(MAX_CAPACITY_DIMS);
        let initial_weights = [0u64; MAX_CAPACITY_DIMS];
        core.root_upper_bound = core.compute_fractional_bound(0, &initial_weights[..num_dims], 0);
        core
    }

    fn set_initial_solution(&mut self, initial_val: u32, selected_orig_ids: &[usize]) {
        if initial_val > self.best_value {
            self.best_value = initial_val;
            self.best_mask = BitSet::<W>::new();
            for &orig_id in selected_orig_ids {
                if let Some(&s_idx) = self.orig_to_sorted.get(&orig_id) {
                    self.best_mask.set(s_idx);
                }
            }
        }
    }

    fn solve(&mut self) -> (u32, Vec<usize>, u64, SolutionStatus) {
        let num_dims = self.capacities.len().min(MAX_CAPACITY_DIMS);
        let current_weights = [0u64; MAX_CAPACITY_DIMS];
        self.explored_nodes = 0;
        self.timed_out = false;
        if let Some(clk) = self.clock {
            self.start_micros = clk.now_micros();
        }

        self.dfs(0, BitSet::<W>::new(), 0, &current_weights[..num_dims]);

        let mut selected_ids = Vec::new();
        for (sorted_idx, it) in self.sorted_items.iter().enumerate() {
            if self.best_mask.has(sorted_idx) {
                selected_ids.push(it.original_id);
            }
        }

        let cap_k = self.top_k.min(MAX_TOP_K);
        if cap_k > 1 && self.best_value > 0 {
            let mut best_ids = selected_ids.clone();
            best_ids.sort_unstable();
            if !self
                .candidates
                .iter()
                .any(|c| c.selected_item_ids == best_ids)
            {
                self.candidates.push(CandidateSolution {
                    value: self.best_value,
                    selected_item_ids: best_ids,
                });
                self.candidates.sort_by_key(|a| Reverse(a.value));
                if self.candidates.len() > cap_k {
                    self.candidates.truncate(cap_k);
                }
            }
        }

        let status = if self.timed_out {
            SolutionStatus::Approximate
        } else {
            SolutionStatus::Optimal
        };

        (self.best_value, selected_ids, self.explored_nodes, status)
    }

    fn upper_bound(&self) -> f64 {
        if self.timed_out {
            self.root_upper_bound.max(f64::from(self.best_value))
        } else {
            f64::from(self.best_value)
        }
    }

    fn optimality_gap(&self) -> f64 {
        if !self.timed_out {
            0.0
        } else if self.best_value > 0 {
            let ub = self.upper_bound();
            (ub - f64::from(self.best_value)) / f64::from(self.best_value)
        } else {
            1.0
        }
    }

    fn compute_fractional_bound(
        &self,
        start_idx: usize,
        current_weights: &[u64],
        current_val: u32,
    ) -> f64 {
        let mut min_remaining_ratio = 1.0f64;
        for (d, &cap) in self.capacities.iter().enumerate() {
            let used = current_weights.get(d).copied().unwrap_or(0);
            if cap > 0 {
                let rem = u32::try_from(u64::from(cap).saturating_sub(used)).unwrap_or(0);
                let ratio = f64::from(rem) / f64::from(cap);
                if ratio < min_remaining_ratio {
                    min_remaining_ratio = ratio;
                }
            }
        }

        if min_remaining_ratio <= 0.0 {
            return f64::from(current_val);
        }

        let mut bound = f64::from(current_val);
        let cap_len = u32::try_from(self.capacities.len()).unwrap_or(u32::MAX);
        let mut rem_budget = min_remaining_ratio * f64::from(cap_len);

        for it in &self.sorted_items[start_idx..] {
            if it.surrogate_weight <= rem_budget {
                bound += f64::from(it.value);
                rem_budget -= it.surrogate_weight;
            } else {
                bound += f64::from(it.value) * (rem_budget / it.surrogate_weight);
                break;
            }
        }

        bound
    }

    fn dfs(
        &mut self,
        idx: usize,
        current_mask: BitSet<W>,
        current_val: u32,
        current_weights: &[u64],
    ) {
        if let Some(limit) = self.max_nodes
            && self.explored_nodes >= limit
        {
            self.timed_out = true;
            return;
        }

        if self.explored_nodes.trailing_zeros() >= 12
            && let (Some(clk), Some(limit_ms)) = (self.clock, self.time_limit_ms)
        {
            let current_ms = crate::clock::elapsed_ms(self.start_micros, clk.now_micros());
            if current_ms >= limit_ms {
                self.timed_out = true;
                return;
            }
        }

        self.explored_nodes += 1;

        let cap_k = self.top_k.min(MAX_TOP_K);
        let cutoff = if cap_k <= 1 {
            f64::from(self.best_value)
        } else if self.candidates.len() >= cap_k {
            self.candidates.last().map_or(0.0, |c| f64::from(c.value))
        } else {
            0.0
        };

        let upper_bound = self.compute_fractional_bound(idx, current_weights, current_val);
        if upper_bound <= cutoff {
            return;
        }

        if idx == self.sorted_items.len() {
            if current_val > self.best_value {
                self.best_value = current_val;
                self.best_mask = current_mask;
            }
            if cap_k > 1 && current_val > 0 {
                self.record_candidate(current_val, current_mask);
            }
            return;
        }

        if current_mask.has(idx) {
            self.dfs(idx + 1, current_mask, current_val, current_weights);
            return;
        }

        let mut bundle_mask = self.requires_closure[idx];
        bundle_mask.set(idx);

        let mut conflict_free = true;
        for (i, conflict) in self
            .conflicts
            .iter()
            .enumerate()
            .take(self.sorted_items.len())
        {
            if bundle_mask.has(i) {
                if !current_mask.is_disjoint(conflict) {
                    conflict_free = false;
                    break;
                }
                if !bundle_mask.is_disjoint(conflict) {
                    conflict_free = false;
                    break;
                }
            }
        }

        let num_dims = self.capacities.len().min(MAX_CAPACITY_DIMS);
        let mut next_weights = [0u64; MAX_CAPACITY_DIMS];
        let mut bundle_added_val = 0u32;
        let mut fits = conflict_free;

        if fits {
            for (d, w) in next_weights.iter_mut().enumerate().take(num_dims) {
                *w = current_weights.get(d).copied().unwrap_or(0);
            }

            for i in 0..self.sorted_items.len() {
                if bundle_mask.has(i) && !current_mask.has(i) {
                    bundle_added_val = bundle_added_val.saturating_add(self.sorted_items[i].value);
                    for (d, w) in next_weights.iter_mut().enumerate().take(num_dims) {
                        let item_w = self.sorted_items[i].weights.get(d).copied().unwrap_or(0);
                        *w += u64::from(item_w);
                        if *w > u64::from(self.capacities[d]) {
                            fits = false;
                            break;
                        }
                    }
                    if !fits {
                        break;
                    }
                }
            }
        }

        if fits {
            let mut next_mask = current_mask;
            next_mask.union_with(&bundle_mask);
            self.dfs(
                idx + 1,
                next_mask,
                current_val.saturating_add(bundle_added_val),
                &next_weights[..num_dims],
            );
        }

        self.dfs(idx + 1, current_mask, current_val, current_weights);
    }
}

pub struct BranchAndBoundSolver<'a> {
    items: &'a [Item],
    capacities: &'a [u32],
    rules: &'a [BusinessRule],
    initial_val: u32,
    initial_selected_ids: Vec<usize>,
    time_limit_ms: Option<f64>,
    max_nodes: Option<u64>,
    clock: Option<&'a dyn Clock>,
    pub explored_nodes: u64,
    upper_bound: f64,
    optimality_gap: f64,
    pub top_k: usize,
    candidates: Vec<CandidateSolution>,
}

impl<'a> BranchAndBoundSolver<'a> {
    #[must_use]
    pub fn new(items: &'a [Item], capacities: &'a [u32], rules: &'a [BusinessRule]) -> Self {
        Self {
            items,
            capacities,
            rules,
            initial_val: 0,
            initial_selected_ids: Vec::new(),
            time_limit_ms: None,
            max_nodes: None,
            clock: None,
            explored_nodes: 0,
            upper_bound: 0.0,
            optimality_gap: 0.0,
            top_k: 1,
            candidates: Vec::new(),
        }
    }

    pub fn set_clock(&mut self, clock: &'a dyn Clock) {
        self.clock = Some(clock);
    }

    pub fn set_top_k(&mut self, k: usize) {
        self.top_k = k;
    }

    pub fn set_initial_solution(&mut self, initial_val: u32, selected_orig_ids: &[usize]) {
        self.initial_val = initial_val;
        self.initial_selected_ids = selected_orig_ids.to_vec();
    }

    pub fn set_time_limit_ms(&mut self, limit_ms: f64) {
        self.time_limit_ms = Some(limit_ms);
    }

    pub fn set_max_nodes(&mut self, limit: u64) {
        self.max_nodes = Some(limit);
    }

    /// Executes the Branch-and-Bound solver after validating input ceilings.
    ///
    /// # Errors
    ///
    /// Returns [`OptimizeError::InvalidInput`] if problem bounds, `top_k`, or `time_limit_ms` exceed safety limits.
    pub fn solve(&mut self) -> Result<(u32, Vec<usize>, u64, SolutionStatus), OptimizeError> {
        validate_problem_input(self.items, self.capacities, self.rules)?;

        if self.top_k > MAX_TOP_K {
            return Err(OptimizeError::InvalidInput {
                message: format!(
                    "top_k ({}) exceeds the maximum supported limit of {MAX_TOP_K}.",
                    self.top_k
                ),
            });
        }

        if let Some(limit_ms) = self.time_limit_ms
            && (!limit_ms.is_finite() || limit_ms < 0.0)
        {
            return Err(OptimizeError::InvalidInput {
                message: format!(
                    "time_limit_ms ({limit_ms}) must be a finite, non-negative number."
                ),
            });
        }

        let n = self.items.len();
        let (val, selected, nodes, status) = if n <= 256 {
            self.solve_fixed::<4>()
        } else if n <= 512 {
            self.solve_fixed::<8>()
        } else if n <= 1024 {
            self.solve_fixed::<16>()
        } else {
            self.solve_fixed::<32>()
        };
        self.explored_nodes = nodes;
        Ok((val, selected, nodes, status))
    }

    fn solve_fixed<const W: usize>(&mut self) -> (u32, Vec<usize>, u64, SolutionStatus) {
        let mut core = ExactSolverCore::<W>::new(self.items, self.capacities, self.rules);
        core.top_k = self.top_k.min(MAX_TOP_K);
        core.clock = self.clock;
        if let Some(limit) = self.time_limit_ms {
            core.time_limit_ms = Some(limit);
        }
        if let Some(nodes) = self.max_nodes {
            core.max_nodes = Some(nodes);
        }
        if self.initial_val > 0 {
            core.set_initial_solution(self.initial_val, &self.initial_selected_ids);
        }
        let res = core.solve();
        self.upper_bound = core.upper_bound();
        self.optimality_gap = core.optimality_gap();
        self.candidates = core.candidates;
        res
    }

    #[must_use]
    pub fn upper_bound(&self) -> f64 {
        self.upper_bound
    }

    #[must_use]
    pub fn optimality_gap(&self) -> f64 {
        self.optimality_gap
    }

    pub fn take_candidates(&mut self) -> Vec<CandidateSolution> {
        mem::take(&mut self.candidates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_and_bound_conflict_rule() {
        let items = vec![
            Item {
                id: 0,
                name: "A".into(),
                weights: vec![10, 10],
                value: 50,
            },
            Item {
                id: 1,
                name: "B".into(),
                weights: vec![10, 10],
                value: 60,
            },
        ];
        let capacities = vec![20, 20];
        let rules = vec![BusinessRule::Conflicts { a_id: 0, b_id: 1 }];

        let mut solver = BranchAndBoundSolver::new(&items, &capacities, &rules);
        let (val, selected, _, status) = solver.solve().expect("solve succeeds");

        assert_eq!(val, 60);
        assert_eq!(selected, vec![1]);
        assert_eq!(status, SolutionStatus::Optimal);
    }

    #[test]
    fn test_fractional_bound_prunes_effectively() {
        let mut items = Vec::new();
        for i in 0u32..20 {
            items.push(Item {
                id: usize::try_from(i).unwrap_or(0),
                name: format!("Item_{i}"),
                weights: vec![10],
                value: (i + 1) * 10,
            });
        }
        let capacities = vec![30];
        let rules = vec![];

        let mut solver = BranchAndBoundSolver::new(&items, &capacities, &rules);
        let (val, selected, nodes, status) = solver.solve().expect("solve succeeds");

        assert_eq!(val, 570);
        assert_eq!(selected.len(), 3);
        assert!(
            nodes < 1000,
            "Explored nodes should be well under 1000, was {nodes}"
        );
        assert_eq!(status, SolutionStatus::Optimal);
    }

    #[test]
    fn test_bnb_rejects_more_than_8_dimensions() {
        let items = vec![Item {
            id: 0,
            name: "A".into(),
            weights: vec![1; 9],
            value: 10,
        }];
        let capacities = vec![10; 9];
        let mut solver = BranchAndBoundSolver::new(&items, &capacities, &[]);
        let res = solver.solve();
        assert!(matches!(res, Err(OptimizeError::InvalidInput { .. })));
    }

    #[test]
    fn test_bnb_weight_u32_overflow_does_not_bypass_capacity() {
        let items = vec![
            Item {
                id: 0,
                name: "HeavyA".into(),
                weights: vec![u32::MAX - 10],
                value: 100,
            },
            Item {
                id: 1,
                name: "HeavyB".into(),
                weights: vec![20],
                value: 50,
            },
        ];
        let capacities = vec![u32::MAX];
        let mut solver = BranchAndBoundSolver::new(&items, &capacities, &[]);
        let (val, selected, _, _) = solver.solve().expect("solve succeeds");

        assert_eq!(val, 100);
        assert_eq!(selected, vec![0]);
    }

    #[test]
    fn test_bnb_rejects_excess_top_k() {
        let items = vec![Item {
            id: 0,
            name: "A".into(),
            weights: vec![1],
            value: 10,
        }];
        let mut solver = BranchAndBoundSolver::new(&items, &[10], &[]);
        solver.set_top_k(MAX_TOP_K + 1);
        let res = solver.solve();
        assert!(matches!(res, Err(OptimizeError::InvalidInput { .. })));
    }
}
