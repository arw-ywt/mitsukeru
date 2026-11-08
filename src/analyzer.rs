// src/analyzer.rs

use crate::bitset::BitSet;
use crate::model::{
    BusinessRule, CapacityConflict, ChainConflict, ConflictReport, Item, TargetConflict,
};
use std::collections::VecDeque;

pub struct InfeasibilityAnalyzer<'a> {
    items: &'a [Item],
    capacities: &'a [u32],
    rules: &'a [BusinessRule],
    target_value: Option<u32>,
    bnb_best_value: u32,
}

impl<'a> InfeasibilityAnalyzer<'a> {
    pub fn new(
        items: &'a [Item],
        capacities: &'a [u32],
        rules: &'a [BusinessRule],
        target_value: Option<u32>,
        bnb_best_value: u32,
    ) -> Self {
        Self {
            items,
            capacities,
            rules,
            target_value,
            bnb_best_value,
        }
    }

    pub fn analyze(&self) -> ConflictReport {
        let n = self.items.len();
        if n <= 256 {
            self.analyze_core::<4>()
        } else if n <= 512 {
            self.analyze_core::<8>()
        } else if n <= 1024 {
            self.analyze_core::<16>()
        } else {
            self.analyze_core::<32>()
        }
    }

    fn analyze_core<const W: usize>(&self) -> ConflictReport {
        let n = self.items.len();
        let n_clamped = n.min(BitSet::<W>::CAPACITY);

        let mut adj_requires = vec![Vec::new(); n];
        let mut conflicts = vec![BitSet::<W>::new(); n];
        let mut closure = vec![BitSet::<W>::new(); n];

        for (i, w) in closure.iter_mut().enumerate().take(n) {
            w.set(i);
        }

        for rule in self.rules {
            match rule {
                BusinessRule::Requires { from_id, to_id } => {
                    if *from_id < n && *to_id < n {
                        adj_requires[*from_id].push(*to_id);
                        closure[*from_id].set(*to_id);
                    }
                }
                BusinessRule::Conflicts { a_id, b_id } => {
                    if *a_id < n && *b_id < n {
                        conflicts[*a_id].set(*b_id);
                        conflicts[*b_id].set(*a_id);
                    }
                }
            }
        }

        for k in 0..n_clamped {
            for i in 0..n_clamped {
                if closure[i].has(k) {
                    let k_closure = closure[k];
                    closure[i].union_with(&k_closure);
                }
            }
        }

        let mut logical_chain_conflicts = Vec::new();
        let mut capacity_conflicts = Vec::new();
        let num_dims = self.capacities.len();

        for root in 0..n {
            let conflict_bits = closure[root].intersect(&conflicts[root]);
            if !conflict_bits.is_empty() {
                let mut visited_bfs = BitSet::<W>::new();
                let mut queue = VecDeque::new();
                let mut path_parent = vec![None; n];

                visited_bfs.set(root);
                queue.push_back(root);

                while let Some(curr) = queue.pop_front() {
                    for &next in &adj_requires[curr] {
                        if !visited_bfs.has(next) {
                            visited_bfs.set(next);
                            path_parent[next] = Some(curr);
                            queue.push_back(next);

                            if conflicts[root].has(next) {
                                let mut path = vec![next];
                                let mut p = Some(curr);
                                while let Some(node) = p {
                                    path.push(node);
                                    p = path_parent[node];
                                }
                                path.reverse();

                                let path_names: Vec<String> = path
                                    .iter()
                                    .map(|&id| self.items[id].name.clone())
                                    .collect();

                                let msg = format!(
                                    "Chain Conflict: [{}] requires '{}', but it conflicts with root '{}'",
                                    path_names.join(" -> "),
                                    self.items[next].name,
                                    self.items[root].name
                                );

                                logical_chain_conflicts.push(ChainConflict {
                                    root_id: root,
                                    conflicting_id: next,
                                    path_ids: path,
                                    message: msg,
                                });
                            }
                        }
                    }
                }
            }

            let mut bundle_weights = [0u32; 8];
            let mut overflowed_dim = None;

            if num_dims <= 8 {
                closure[root].for_each_one(|req_id| {
                    if req_id < n {
                        let it_w = &self.items[req_id].weights;
                        for (d, w) in bundle_weights.iter_mut().enumerate().take(num_dims) {
                            *w += it_w.get(d).copied().unwrap_or(0);
                        }
                    }
                });

                for (d, w) in bundle_weights.iter().enumerate().take(num_dims) {
                    if *w > self.capacities[d] {
                        overflowed_dim = Some(d);
                        break;
                    }
                }
            } else {
                let mut dyn_weights = vec![0u32; num_dims];
                closure[root].for_each_one(|req_id| {
                    if req_id < n {
                        let it_w = &self.items[req_id].weights;
                        for (d, w) in dyn_weights.iter_mut().enumerate().take(num_dims) {
                            *w += it_w.get(d).copied().unwrap_or(0);
                        }
                    }
                });
                for (d, w) in dyn_weights.iter().enumerate().take(num_dims) {
                    if *w > self.capacities[d] {
                        overflowed_dim = Some(d);
                        bundle_weights[0] = dyn_weights[d];
                        break;
                    }
                }
            }

            if let Some(d) = overflowed_dim {
                let mut bundle_ids = Vec::with_capacity(closure[root].count_ones());
                closure[root].for_each_one(|id| {
                    if id < n {
                        bundle_ids.push(id);
                    }
                });

                let consumed = if num_dims <= 8 {
                    bundle_weights[d]
                } else {
                    bundle_weights[0]
                };

                let msg = match bundle_ids.len() {
                    1 => format!(
                        "Single Capacity Overflow: '{}' consumes {} for constraint {}, exceeding limit ({})",
                        self.items[root].name,
                        consumed,
                        d + 1,
                        self.capacities[d]
                    ),
                    _ => format!(
                        "Bundle Capacity Overflow: Selecting '{}' requires {} items with total constraint {} consumption ({}), exceeding limit ({})",
                        self.items[root].name,
                        bundle_ids.len(),
                        d + 1,
                        consumed,
                        self.capacities[d]
                    ),
                };

                capacity_conflicts.push(CapacityConflict {
                    root_id: root,
                    dimension: d,
                    capacity_limit: self.capacities[d],
                    consumed_weight: consumed,
                    bundle_ids,
                    message: msg,
                });
            }
        }

        let mut target_conflict = None;
        if let Some(target) = self.target_value 
            && self.bnb_best_value < target {
                let msg = format!(
                    "Unachievable Target: Target value ({} pt) cannot be reached; maximum achievable score under constraints is ({} pt)",
                    target, self.bnb_best_value
                );
                target_conflict = Some(TargetConflict {
                    target_value: target,
                    achievable_value: self.bnb_best_value,
                    message: msg,
                });
            }

        let is_feasible = logical_chain_conflicts.is_empty()
            && capacity_conflicts.is_empty()
            && target_conflict.is_none();

        ConflictReport {
            is_feasible,
            logical_chain_conflicts,
            capacity_conflicts,
            target_conflict,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_conflict() {
        let items = vec![
            Item { id: 0, name: "A".into(), weights: vec![10], value: 10 },
            Item { id: 1, name: "B".into(), weights: vec![10], value: 10 },
            Item { id: 2, name: "C".into(), weights: vec![10], value: 10 },
        ];
        let rules = vec![
            BusinessRule::Requires { from_id: 0, to_id: 1 },
            BusinessRule::Requires { from_id: 1, to_id: 2 },
            BusinessRule::Conflicts { a_id: 0, b_id: 2 },
        ];

        let analyzer = InfeasibilityAnalyzer::new(&items, &[50], &rules, None, 10);
        let report = analyzer.analyze();
        assert!(!report.is_feasible);
        assert_eq!(report.logical_chain_conflicts.len(), 1);

        let conflict = &report.logical_chain_conflicts[0];
        assert_eq!(conflict.root_id, 0);
        assert_eq!(conflict.conflicting_id, 2);
        assert_eq!(conflict.path_ids, vec![0, 1, 2]);
        assert!(conflict.message.contains("A -> B -> C"));
    }
}