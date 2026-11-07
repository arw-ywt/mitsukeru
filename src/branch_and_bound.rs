// branch_and_bound.rs

use crate::model::{BusinessRule, Item, SolutionStatus};

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
struct BitSet256 {
    words: [u64; 4],
}

impl BitSet256 {
    #[inline(always)]
    fn new() -> Self {
        Self { words: [0; 4] }
    }

    #[inline(always)]
    fn set(&mut self, idx: usize) {
        if idx < 256 {
            self.words[idx / 64] |= 1u64 << (idx % 64);
        }
    }

    #[inline(always)]
    fn has(&self, idx: usize) -> bool {
        if idx < 256 {
            (self.words[idx / 64] & (1u64 << (idx % 64))) != 0
        } else {
            false
        }
    }

    #[inline(always)]
    fn is_disjoint(&self, other: &Self) -> bool {
        ((self.words[0] & other.words[0])
            | (self.words[1] & other.words[1])
            | (self.words[2] & other.words[2])
            | (self.words[3] & other.words[3]))
            == 0
    }

    #[inline(always)]
    fn contains_all(&self, required: &Self) -> bool {
        ((self.words[0] & required.words[0]) == required.words[0])
            && ((self.words[1] & required.words[1]) == required.words[1])
            && ((self.words[2] & required.words[2]) == required.words[2])
            && ((self.words[3] & required.words[3]) == required.words[3])
    }
}

#[derive(Clone, Debug)]
struct InternalItem {
    original_id: usize,
    weights: Vec<u32>,
    value: u32,
    surrogate_weight: f64,
    density: f64,
}

pub struct BranchAndBoundSolver<'a> {
    capacities: &'a [u32],
    sorted_items: Vec<InternalItem>,
    conflicts: Vec<BitSet256>,
    requires: Vec<BitSet256>,
    requires_closure: Vec<BitSet256>,
    best_value: u32,
    best_mask: BitSet256,
    pub explored_nodes: u64,
    max_nodes: Option<u64>,
    orig_to_sorted: std::collections::HashMap<usize, usize>,
    time_limit_ms: Option<f64>,
    start_time: Option<Instant>,
    timed_out: bool,
}

impl<'a> BranchAndBoundSolver<'a> {
    
    pub fn new(items: &[Item], capacities: &'a [u32], rules: &[BusinessRule]) -> Self {
        Self::new_internal(items, capacities, rules, None)
    }
    
    pub fn set_max_nodes(&mut self, limit: u64) {
        self.max_nodes = Some(limit);
    }

    pub fn new_with_priorities(
        items: &[Item],
        capacities: &'a [u32],
        rules: &[BusinessRule],
        _priorities: &[f32],
    ) -> Self {
        Self::new(items, capacities, rules)
    }
    
    fn new_internal(
        items: &[Item],
        capacities: &'a [u32],
        rules: &[BusinessRule],
        priorities: Option<&[f32]>,
    ) -> Self {
        let n = items.len().min(256);

        let mut internal_items: Vec<InternalItem> = items
            .iter()
            .take(n)
            .map(|it| {
                let mut surrogate_w = 0.0;
                for (d, &cap) in capacities.iter().enumerate() {
                    let w = it.weights.get(d).copied().unwrap_or(0);
                    if cap > 0 {
                        surrogate_w += (w as f64) / (cap as f64);
                    }
                }
                if surrogate_w == 0.0 {
                    surrogate_w = 1e-9;
                }
                let density = (it.value as f64) / surrogate_w;
                InternalItem {
                    original_id: it.id,
                    weights: it.weights.clone(),
                    value: it.value,
                    surrogate_weight: surrogate_w,
                    density,
                }
            })
            .collect();

        internal_items.sort_by(|a, b| {
            b.density
                .partial_cmp(&a.density)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut orig_to_sorted = std::collections::HashMap::new();
        for (sorted_idx, it) in internal_items.iter().enumerate() {
            orig_to_sorted.insert(it.original_id, sorted_idx);
        }

        let mut prefer_take = BitSet256::new();
        for sorted_idx in 0..internal_items.len() {
            prefer_take.set(sorted_idx);
        }

        let mut conflicts = vec![BitSet256::new(); n];
        let mut requires = vec![BitSet256::new(); n];

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

        let mut requires_closure = requires.clone();
        for k in 0..n {
            for i in 0..n {
                if requires_closure[i].has(k) {
                    for word_idx in 0..4 {
                        requires_closure[i].words[word_idx] |= requires_closure[k].words[word_idx];
                    }
                }
            }
        }

        Self {
            capacities,
            sorted_items: internal_items,
            conflicts,
            requires,
            requires_closure,
            best_value: 0,
            best_mask: BitSet256::new(),
            explored_nodes: 0,
            time_limit_ms: None,
            start_time: None,
            timed_out: false,
            orig_to_sorted,
            max_nodes: None,
        }
    }

    pub fn set_initial_solution(&mut self, initial_val: u32, selected_orig_ids: &[usize]) {
        if initial_val > self.best_value {
            self.best_value = initial_val;
            self.best_mask = BitSet256::new();
            for &orig_id in selected_orig_ids {
                if let Some(&s_idx) = self.orig_to_sorted.get(&orig_id) {
                    self.best_mask.set(s_idx);
                }
            }
        }
    }

    pub fn set_time_limit_ms(&mut self, limit_ms: f64) {
        self.time_limit_ms = Some(limit_ms);
    }
    
    pub fn solve(&mut self) -> (u32, Vec<usize>, u64, SolutionStatus) {
        let num_dims = self.capacities.len();
        let current_weights = vec![0u32; num_dims];
        self.explored_nodes = 0;
        self.timed_out = false;
        self.start_time = Some(Instant::now());

        self.dfs(0, BitSet256::new(), 0, &current_weights);

        let mut selected_ids = Vec::new();
        for (sorted_idx, it) in self.sorted_items.iter().enumerate() {
            if self.best_mask.has(sorted_idx) {
                selected_ids.push(it.original_id);
            }
        }

        let status = if !self.timed_out {
            SolutionStatus::Optimal
        } else {
            SolutionStatus::Approximate
        };

        (self.best_value, selected_ids, self.explored_nodes, status)
    }


    #[inline(always)]
    fn compute_fractional_bound(
        &self,
        start_idx: usize,
        current_weights: &[u32],
        current_val: u32,
    ) -> f64 {
        let mut min_remaining_ratio = 1.0f64;
        for (d, &cap) in self.capacities.iter().enumerate() {
            let used = current_weights.get(d).copied().unwrap_or(0);
            if cap > 0 {
                let ratio = ((cap.saturating_sub(used)) as f64) / (cap as f64);
                if ratio < min_remaining_ratio {
                    min_remaining_ratio = ratio;
                }
            }
        }

        if min_remaining_ratio <= 0.0 {
            return current_val as f64;
        }

        let mut bound = current_val as f64;
        let mut rem_budget = min_remaining_ratio * (self.capacities.len() as f64);

        for it in &self.sorted_items[start_idx..] {
            if it.surrogate_weight <= rem_budget {
                bound += it.value as f64;
                rem_budget -= it.surrogate_weight;
            } else {
                bound += (it.value as f64) * (rem_budget / it.surrogate_weight);
                break;
            }
        }

        bound
    }

    fn dfs(
        &mut self,
        idx: usize,
        current_mask: BitSet256,
        current_val: u32,
        current_weights: &[u32],
    ) {
        
        if let Some(limit) = self.max_nodes {
            if self.explored_nodes >= limit {
                self.timed_out = true;
                return;
            }
        }

        if (self.explored_nodes & 4095) == 0 {
            if let (Some(start), Some(limit)) = (self.start_time, self.time_limit_ms) {
                if start.elapsed().as_secs_f64() * 1000.0 >= limit {
                    self.timed_out = true;
                    return;
                }
            }
        }

        if self.timed_out {
            return;
        }

        self.explored_nodes += 1;

        let upper_bound = self.compute_fractional_bound(idx, current_weights, current_val);
        if upper_bound <= self.best_value as f64 {
            return;
        }

        if idx == self.sorted_items.len() {
            if current_val > self.best_value {
                self.best_value = current_val;
                self.best_mask = current_mask;
            }
            return;
        }

        if current_mask.has(idx) {
            self.dfs(idx + 1, current_mask, current_val, current_weights);
            return;
        }

        let req_closure = &self.requires_closure[idx];
        let mut bundle_mask = *req_closure;
        bundle_mask.set(idx);

        let mut conflict_free = true;
        for i in 0..self.sorted_items.len() {
            if bundle_mask.has(i) {
                if !current_mask.is_disjoint(&self.conflicts[i]) {
                    conflict_free = false;
                    break;
                }
                if !bundle_mask.is_disjoint(&self.conflicts[i]) {
                    conflict_free = false;
                    break;
                }
            }
        }

        let num_dims = self.capacities.len();
        let mut next_weights = [0u32; 8];
        let mut bundle_added_val = 0u32;
        let mut fits = conflict_free;

        if fits {
            for d in 0..num_dims {
                next_weights[d] = current_weights.get(d).copied().unwrap_or(0);
            }

            for i in 0..self.sorted_items.len() {
                if bundle_mask.has(i) && !current_mask.has(i) {
                    bundle_added_val += self.sorted_items[i].value;
                    for d in 0..num_dims {
                        let w = self.sorted_items[i].weights.get(d).copied().unwrap_or(0);
                        next_weights[d] += w;
                        if next_weights[d] > self.capacities[d] {
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
            for w_idx in 0..4 {
                next_mask.words[w_idx] |= bundle_mask.words[w_idx];
            }
            self.dfs(
                idx + 1,
                next_mask,
                current_val + bundle_added_val,
                &next_weights[..num_dims],
            );
        }

        self.dfs(idx + 1, current_mask, current_val, current_weights);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_and_bound_conflict_rule() {
        let items = vec![
            Item { id: 0, name: "A".into(), weights: vec![10, 10], value: 50 },
            Item { id: 1, name: "B".into(), weights: vec![10, 10], value: 60 },
        ];
        let capacities = vec![20, 20];
        let rules = vec![BusinessRule::Conflicts { a_id: 0, b_id: 1 }];

        let mut solver = BranchAndBoundSolver::new(&items, &capacities, &rules);
        let (val, selected, _, status) = solver.solve();

        assert_eq!(val, 60);
        assert_eq!(selected, vec![1]);
        assert_eq!(status, SolutionStatus::Optimal);
    }

    #[test]
    fn test_fractional_bound_prunes_effectively() {
        let mut items = Vec::new();
        for i in 0..20 {
            items.push(Item {
                id: i,
                name: format!("Item_{}", i),
                weights: vec![10],
                value: (i as u32 + 1) * 10,
            });
        }
        let capacities = vec![30];
        let rules = vec![];

        let mut solver = BranchAndBoundSolver::new(&items, &capacities, &rules);
        let (val, selected, nodes, status) = solver.solve();

        assert_eq!(val, 570);
        assert_eq!(selected.len(), 3);
        assert!(nodes < 1000, "Explored nodes should be well under 1000, was {}", nodes);
        assert_eq!(status, SolutionStatus::Optimal);
    }
}