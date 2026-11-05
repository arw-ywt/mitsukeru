// branch_and_bound.rs

use crate::model::{BusinessRule, Item};

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
    best_value: u32,
    best_mask: BitSet256,
    pub explored_nodes: u64,
}

impl<'a> BranchAndBoundSolver<'a> {
    pub fn new(items: &[Item], capacities: &'a [u32], rules: &[BusinessRule]) -> Self {
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

        Self {
            capacities,
            sorted_items: internal_items,
            conflicts,
            requires,
            best_value: 0,
            best_mask: BitSet256::new(),
            explored_nodes: 0,
        }
    }

    pub fn solve(&mut self) -> (u32, Vec<usize>, u64) {
        let num_dims = self.capacities.len();
        let current_weights = vec![0u32; num_dims];
        self.best_value = 0;
        self.best_mask = BitSet256::new();
        self.explored_nodes = 0;

        self.dfs(0, BitSet256::new(), 0, &current_weights);

        let mut selected_ids = Vec::new();
        for (sorted_idx, it) in self.sorted_items.iter().enumerate() {
            if self.best_mask.has(sorted_idx) {
                selected_ids.push(it.original_id);
            }
        }

        (self.best_value, selected_ids, self.explored_nodes)
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
        self.explored_nodes += 1;

        let upper_bound = self.compute_fractional_bound(idx, current_weights, current_val);
        if upper_bound <= self.best_value as f64 {
            return;
        }

        if idx == self.sorted_items.len() {
            for i in 0..self.sorted_items.len() {
                if current_mask.has(i) {
                    let req = &self.requires[i];
                    if !current_mask.contains_all(req) {
                        return;
                    }
                }
            }

            if current_val > self.best_value {
                self.best_value = current_val;
                self.best_mask = current_mask;
            }
            return;
        }

        let item = &self.sorted_items[idx];
        let can_conflict = current_mask.is_disjoint(&self.conflicts[idx]);
        let mut can_fit = true;
        let mut next_weights = current_weights.to_vec();

        for d in 0..self.capacities.len() {
            let w = item.weights.get(d).copied().unwrap_or(0);
            if next_weights[d] + w > self.capacities[d] {
                can_fit = false;
                break;
            }
            next_weights[d] += w;
        }

        if can_conflict && can_fit {
            let mut next_mask = current_mask;
            next_mask.set(idx);
            self.dfs(idx + 1, next_mask, current_val + item.value, &next_weights);
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
        let (val, selected, _) = solver.solve();

        assert_eq!(val, 60);
        assert_eq!(selected, vec![1]);
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
        let (val, selected, nodes) = solver.solve();

        assert_eq!(val, 570);
        assert_eq!(selected.len(), 3);
        assert!(nodes < 1000, "Explored nodes should be well under 1000, was {}", nodes);
    }
}