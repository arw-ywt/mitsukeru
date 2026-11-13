// src/analyzer.rs

use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;

use crate::bitset::BitSet;
use crate::model::{
    BusinessRule, CapacityConflict, ChainConflict, ConflictReport, Item, OptimizeError,
    TargetConflict,
};
use crate::validation::{MAX_CAPACITY_DIMS, validate_problem_input};

pub struct InfeasibilityAnalyzer<'a> {
    items: &'a [Item],
    capacities: &'a [u32],
    rules: &'a [BusinessRule],
    target_value: Option<u32>,
    bnb_best_value: u32,
}

impl<'a> InfeasibilityAnalyzer<'a> {
    #[must_use]
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

    /// Analyzes the problem instance for logical chain, capacity, and target conflicts.
    ///
    /// # Errors
    ///
    /// Returns [`OptimizeError::InvalidInput`] if problem bounds exceed safety ceilings.
    pub fn analyze(&self) -> Result<ConflictReport, OptimizeError> {
        validate_problem_input(self.items, self.capacities, self.rules)?;

        let n = self.items.len();
        if n <= 256 {
            Ok(self.analyze_core::<4>())
        } else if n <= 512 {
            Ok(self.analyze_core::<8>())
        } else if n <= 1024 {
            Ok(self.analyze_core::<16>())
        } else {
            Ok(self.analyze_core::<32>())
        }
    }

    fn analyze_core<const W: usize>(&self) -> ConflictReport {
        let n = self.items.len();
        let (adj_requires, conflicts, closure) = self.build_rule_structures::<W>();

        let mut logical_chain_conflicts = Vec::new();
        let mut capacity_conflicts = Vec::new();

        for root in 0..n {
            if let Some(chain) =
                self.find_chain_conflict::<W>(root, &closure[root], &conflicts[root], &adj_requires)
            {
                logical_chain_conflicts.extend(chain);
            }
            if let Some(cap) = self.check_capacity_conflict::<W>(root, &closure[root]) {
                capacity_conflicts.push(cap);
            }
        }

        let target_conflict = self.check_target_conflict();
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

    fn build_rule_structures<const W: usize>(
        &self,
    ) -> (Vec<Vec<usize>>, Vec<BitSet<W>>, Vec<BitSet<W>>) {
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

        (adj_requires, conflicts, closure)
    }

    fn find_chain_conflict<const W: usize>(
        &self,
        root: usize,
        root_closure: &BitSet<W>,
        root_conflicts: &BitSet<W>,
        adj_requires: &[Vec<usize>],
    ) -> Option<Vec<ChainConflict>> {
        let n = self.items.len();
        let conflict_bits = root_closure.intersect(root_conflicts);
        if conflict_bits.is_empty() {
            return None;
        }

        let mut conflicts = Vec::new();
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

                    if root_conflicts.has(next) {
                        let mut path = vec![next];
                        let mut p = Some(curr);
                        while let Some(node) = p {
                            path.push(node);
                            p = path_parent[node];
                        }
                        path.reverse();

                        let path_names: Vec<String> =
                            path.iter().map(|&id| self.items[id].name.clone()).collect();

                        let msg = format!(
                            "Chain Conflict: [{}] requires '{}', but it conflicts with root '{}'",
                            path_names.join(" -> "),
                            self.items[next].name,
                            self.items[root].name
                        );

                        conflicts.push(ChainConflict {
                            root_id: root,
                            conflicting_id: next,
                            path_ids: path,
                            message: msg,
                        });
                    }
                }
            }
        }
        Some(conflicts)
    }

    fn check_capacity_conflict<const W: usize>(
        &self,
        root: usize,
        root_closure: &BitSet<W>,
    ) -> Option<CapacityConflict> {
        let n = self.items.len();
        let num_dims = self.capacities.len();
        let mut overflowed_dim = None;
        let mut consumed = 0u64;

        if num_dims <= MAX_CAPACITY_DIMS {
            let mut bundle_weights = [0u64; MAX_CAPACITY_DIMS];
            root_closure.for_each_one(|req_id| {
                if req_id < n {
                    let it_w = &self.items[req_id].weights;
                    for (d, w) in bundle_weights.iter_mut().enumerate().take(num_dims) {
                        *w += u64::from(it_w.get(d).copied().unwrap_or(0));
                    }
                }
            });

            for (d, &w) in bundle_weights.iter().enumerate().take(num_dims) {
                if w > u64::from(self.capacities[d]) {
                    overflowed_dim = Some(d);
                    consumed = w;
                    break;
                }
            }
        } else {
            let mut dyn_weights = vec![0u64; num_dims];
            root_closure.for_each_one(|req_id| {
                if req_id < n {
                    let it_w = &self.items[req_id].weights;
                    for (d, w) in dyn_weights.iter_mut().enumerate().take(num_dims) {
                        *w += u64::from(it_w.get(d).copied().unwrap_or(0));
                    }
                }
            });
            for (d, &w) in dyn_weights.iter().enumerate().take(num_dims) {
                if w > u64::from(self.capacities[d]) {
                    overflowed_dim = Some(d);
                    consumed = w;
                    break;
                }
            }
        }

        overflowed_dim.map(|d| {
            let mut bundle_ids = Vec::with_capacity(root_closure.count_ones());
            root_closure.for_each_one(|id| {
                if id < n {
                    bundle_ids.push(id);
                }
            });

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

            CapacityConflict {
                root_id: root,
                dimension: d,
                capacity_limit: self.capacities[d],
                consumed_weight: consumed,
                bundle_ids,
                message: msg,
            }
        })
    }

    fn check_target_conflict(&self) -> Option<TargetConflict> {
        if let Some(target) = self.target_value
            && self.bnb_best_value < target
        {
            let msg = format!(
                "Unachievable Target: Target value ({} pt) cannot be reached; maximum achievable score under constraints is ({} pt)",
                target, self.bnb_best_value
            );
            Some(TargetConflict {
                target_value: target,
                achievable_value: self.bnb_best_value,
                message: msg,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_conflict() {
        let items = vec![
            Item {
                id: 0,
                name: "A".into(),
                weights: vec![10],
                value: 10,
            },
            Item {
                id: 1,
                name: "B".into(),
                weights: vec![10],
                value: 10,
            },
            Item {
                id: 2,
                name: "C".into(),
                weights: vec![10],
                value: 10,
            },
        ];
        let rules = vec![
            BusinessRule::Requires {
                from_id: 0,
                to_id: 1,
            },
            BusinessRule::Requires {
                from_id: 1,
                to_id: 2,
            },
            BusinessRule::Conflicts { a_id: 0, b_id: 2 },
        ];

        let analyzer = InfeasibilityAnalyzer::new(&items, &[50], &rules, None, 10);
        let report = analyzer.analyze().expect("analysis succeeds");
        assert!(!report.is_feasible);
        assert_eq!(report.logical_chain_conflicts.len(), 1);

        let conflict = &report.logical_chain_conflicts[0];
        assert_eq!(conflict.root_id, 0);
        assert_eq!(conflict.conflicting_id, 2);
        assert_eq!(conflict.path_ids, vec![0, 1, 2]);
        assert!(conflict.message.contains("A -> B -> C"));
    }

    #[test]
    fn test_analyzer_detects_u32_overflow_capacity_conflict() {
        let items = vec![
            Item {
                id: 0,
                name: "A".into(),
                weights: vec![u32::MAX - 10],
                value: 10,
            },
            Item {
                id: 1,
                name: "B".into(),
                weights: vec![20],
                value: 10,
            },
        ];
        let rules = vec![BusinessRule::Requires {
            from_id: 0,
            to_id: 1,
        }];

        let analyzer = InfeasibilityAnalyzer::new(&items, &[u32::MAX], &rules, None, 0);
        let report = analyzer.analyze().expect("analysis succeeds");
        assert!(!report.is_feasible);
        assert_eq!(report.capacity_conflicts.len(), 1);
        assert_eq!(
            report.capacity_conflicts[0].consumed_weight,
            u64::from(u32::MAX) + 10
        );
    }

    #[test]
    fn test_analyzer_rejects_excess_dims() {
        let items = vec![Item {
            id: 0,
            name: "A".into(),
            weights: vec![1; 9],
            value: 10,
        }];
        let analyzer = InfeasibilityAnalyzer::new(&items, &[10; 9], &[], None, 0);
        let res = analyzer.analyze();
        assert!(matches!(res, Err(OptimizeError::InvalidInput { .. })));
    }
}
