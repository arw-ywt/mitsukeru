use crate::model::{BusinessRule, Item};

pub struct BranchAndBoundSolver<'a> {
    items: &'a [Item],
    capacities: &'a [u32],
    conflicts: Vec<u64>,
    requires: Vec<u64>,
    best_value: u32,
    best_mask: u64,
    pub explored_nodes: u64,
}

impl<'a> BranchAndBoundSolver<'a> {
    pub fn new(items: &'a [Item], capacities: &'a [u32], rules: &[BusinessRule]) -> Self {
        let n = items.len();
        let mut conflicts = vec![0u64; n];
        let mut requires = vec![0u64; n];

        for rule in rules {
            match rule {
                BusinessRule::Requires { from_id, to_id } => {
                    if *from_id < n && *to_id < n {
                        requires[*from_id] |= 1u64 << to_id;
                    }
                }
                BusinessRule::Conflicts { a_id, b_id } => {
                    if *a_id < n && *b_id < n {
                        conflicts[*a_id] |= 1u64 << b_id;
                        conflicts[*b_id] |= 1u64 << a_id;
                    }
                }
            }
        }

        Self {
            items,
            capacities,
            conflicts,
            requires,
            best_value: 0,
            best_mask: 0,
            explored_nodes: 0,
        }
    }

    pub fn solve(&mut self) -> (u32, Vec<usize>, u64) {
        let num_dims = self.capacities.len();
        let current_weights = vec![0u32; num_dims];
        self.best_value = 0;
        self.best_mask = 0;
        self.explored_nodes = 0;

        self.dfs(0, 0, 0, &current_weights);

        let mut selected_ids = Vec::new();
        for i in 0..self.items.len() {
            if (self.best_mask & (1u64 << i)) != 0 {
                selected_ids.push(self.items[i].id);
            }
        }

        (self.best_value, selected_ids, self.explored_nodes)
    }

    fn dfs(&mut self, idx: usize, current_mask: u64, current_val: u32, current_weights: &[u32]) {
        self.explored_nodes += 1;

        let mut remaining_val = 0;
        for i in idx..self.items.len() {
            remaining_val += self.items[i].value;
        }
        if current_val + remaining_val <= self.best_value {
            return;
        }

        if idx == self.items.len() {
            for i in 0..self.items.len() {
                if (current_mask & (1u64 << i)) != 0 {
                    let req = self.requires[i];
                    if (current_mask & req) != req {
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

        let item = &self.items[idx];
        let item_bit = 1u64 << idx;

        let can_conflict = (current_mask & self.conflicts[idx]) == 0;
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
            self.dfs(idx + 1, current_mask | item_bit, current_val + item.value, &next_weights);
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
}