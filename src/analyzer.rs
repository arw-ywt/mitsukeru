use crate::model::{BusinessRule, ConflictReport, Item};
use std::collections::{HashSet, VecDeque};

pub struct InfeasibilityAnalyzer<'a> {
    items: &'a [Item],
    capacities: &'a [u32],
    rules: &'a [BusinessRule],
    target_value: Option<u32>,
    mip_best_value: u32,
}

impl<'a> InfeasibilityAnalyzer<'a> {
    pub fn new(
        items: &'a [Item],
        capacities: &'a [u32],
        rules: &'a [BusinessRule],
        target_value: Option<u32>,
        mip_best_value: u32,
    ) -> Self {
        Self {
            items,
            capacities,
            rules,
            target_value,
            mip_best_value,
        }
    }

    pub fn analyze(&self) -> ConflictReport {
        let n = self.items.len();
        let mut adj_requires = vec![Vec::new(); n];
        let mut conflicts = vec![HashSet::new(); n];

        // グラフの構築
        for rule in self.rules {
            match rule {
                BusinessRule::Requires { from_id, to_id } => {
                    if *from_id < n && *to_id < n {
                        adj_requires[*from_id].push(*to_id);
                    }
                }
                BusinessRule::Conflicts { a_id, b_id } => {
                    if *a_id < n && *b_id < n {
                        conflicts[*a_id].insert(*b_id);
                        conflicts[*b_id].insert(*a_id);
                    }
                }
            }
        }

        let mut logical_chain_conflicts = Vec::new();
        let mut capacity_conflicts = Vec::new();

        // 1. 各アイテムを起点とした「依存連鎖の追跡」と「数値パッケージ検査」
        for root in 0..n {
            let mut visited = HashSet::new();
            let mut queue = VecDeque::new();
            let mut path_parent = vec![None; n];

            visited.insert(root);
            queue.push_back(root);

            while let Some(curr) = queue.pop_front() {
                for &next in &adj_requires[curr] {
                    if !visited.contains(&next) {
                        visited.insert(next);
                        path_parent[next] = Some(curr);
                        queue.push_back(next);

                        // 連鎖の先で、起点 root と排他関係にあるものを踏んだか？
                        if conflicts[root].contains(&next) {
                            // 経路パスの復元
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

                            logical_chain_conflicts.push(format!(
                                "連鎖矛盾: 「{}」が必須連鎖しますが、終点「{}」は起点「{}」と排他です（{} は採択不能）",
                                path_names.join(" ➔ "),
                                self.items[next].name,
                                self.items[root].name,
                                self.items[root].name
                            ));
                        }
                    }
                }
            }

            // 数値検査: root を積むと同時に道連れになる全アイテムの合計重量
            let num_dims = self.capacities.len();
            let mut bundle_weights = vec![0u32; num_dims];
            for &req_id in &visited {
                for d in 0..num_dims {
                    bundle_weights[d] += self.items[req_id].weights.get(d).copied().unwrap_or(0);
                }
            }

            for d in 0..num_dims {
                if bundle_weights[d] > self.capacities[d] {
                    if visited.len() == 1 {
                        capacity_conflicts.push(format!(
                            "単体容量超過: 「{}」の制約{}消費量 ({}) が上限 ({}) を超過しています",
                            self.items[root].name,
                            d + 1,
                            bundle_weights[d],
                            self.capacities[d]
                        ));
                    } else {
                        capacity_conflicts.push(format!(
                            "パッケージ容量超過: 「{}」を採用すると必須連鎖する計{}件の合計制約{} ({}) が上限 ({}) を突破します",
                            self.items[root].name,
                            visited.len(),
                            d + 1,
                            bundle_weights[d],
                            self.capacities[d]
                        ));
                    }
                    break;
                }
            }
        }

        // 2. 目標満足度に対する矛盾
        let mut target_conflict = None;
        if let Some(target) = self.target_value {
            if self.mip_best_value < target {
                target_conflict = Some(format!(
                    "目標未達: 目標満足度 ({} pt) に対して、全制約を守った最大達成可能スコアは ({} pt) です",
                    target, self.mip_best_value
                ));
            }
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
        // A -> B -> C かつ A ⊥ C
        let rules = vec![
            BusinessRule::Requires { from_id: 0, to_id: 1 },
            BusinessRule::Requires { from_id: 1, to_id: 2 },
            BusinessRule::Conflicts { a_id: 0, b_id: 2 },
        ];

        let analyzer = InfeasibilityAnalyzer::new(&items, &[50], &rules, None, 10);
        let report = analyzer.analyze();
        assert!(!report.is_feasible);
        assert_eq!(report.logical_chain_conflicts.len(), 1);
        assert!(report.logical_chain_conflicts[0].contains("A ➔ B ➔ C"));
    }
}