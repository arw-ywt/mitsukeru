// src/heuristics.rs

use crate::bitset::BitSet;
use crate::model::{BusinessRule, Item};

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

pub const EMBED_DIM: usize = 8;
const MESSAGE_PASSING_ROUNDS: usize = 2;
const FIX_HIGH_THRESHOLD: f32 = 0.70;
const FIX_LOW_THRESHOLD: f32 = 0.30;

type HeuristicBitSet = BitSet<16>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableDecision {
    Fixed(bool),
    Free,
}

#[derive(Debug, Clone)]
pub struct DivingPrediction {
    pub decisions: Vec<VariableDecision>,
    pub initial_best_value: u32,
    pub initial_selected_ids: Vec<usize>,
    pub confidence_scores: Vec<f32>,
    pub execution_time_ms: f64,
}

pub struct NeuralDivingEngine;

impl NeuralDivingEngine {
    pub fn predict(
        items: &[Item],
        capacities: &[u32],
        rules: &[BusinessRule],
    ) -> DivingPrediction {
        let timer = Instant::now();
        let n = items.len().min(HeuristicBitSet::CAPACITY);

        if n == 0 {
            return DivingPrediction {
                decisions: Vec::new(),
                initial_best_value: 0,
                initial_selected_ids: Vec::new(),
                confidence_scores: Vec::new(),
                execution_time_ms: 0.0,
            };
        }

        let mut requires_out: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut requires_in: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut conflicts: Vec<Vec<usize>> = vec![Vec::new(); n];

        for rule in rules {
            match *rule {
                BusinessRule::Requires { from_id, to_id } => {
                    if from_id < n && to_id < n {
                        requires_out[from_id].push(to_id);
                        requires_in[to_id].push(from_id);
                    }
                }
                BusinessRule::Conflicts { a_id, b_id } => {
                    if a_id < n && b_id < n {
                        conflicts[a_id].push(b_id);
                        conflicts[b_id].push(a_id);
                    }
                }
            }
        }

        let cap_x = capacities.first().copied().unwrap_or(1).max(1) as f32;
        let cap_y = capacities.get(1).copied().unwrap_or(1).max(1) as f32;
        let mut eff_weights_0 = vec![0.0f32; n];
        let mut eff_weights_1 = vec![0.0f32; n];
        let mut eff_values = vec![0.0f32; n];

        let mut queue = vec![0usize; n];

        for i in 0..n {
            let mut visited = HeuristicBitSet::new();
            let mut q_head = 0;
            let mut q_tail = 0;

            visited.set(i);
            queue[q_tail] = i;
            q_tail += 1;

            let mut tot_w0 = 0.0f32;
            let mut tot_w1 = 0.0f32;
            let mut tot_v = 0.0f32;

            while q_head < q_tail {
                let curr = queue[q_head];
                q_head += 1;

                tot_w0 += items[curr].weights.first().copied().unwrap_or(0) as f32;
                tot_w1 += items[curr].weights.get(1).copied().unwrap_or(0) as f32;
                tot_v += items[curr].value as f32;

                for &req in &requires_out[curr] {
                    if !visited.has(req) {
                        visited.set(req);
                        if q_tail < n {
                            queue[q_tail] = req;
                            q_tail += 1;
                        }
                    }
                }
            }
            eff_weights_0[i] = tot_w0;
            eff_weights_1[i] = tot_w1;
            eff_values[i] = tot_v;
        }

        let max_val = eff_values.iter().copied().fold(1.0f32, f32::max).max(1.0);
        let mut node_embeddings: Vec<[f32; EMBED_DIM]> = Vec::with_capacity(n);

        for i in 0..n {
            let mut emb = [0.0f32; EMBED_DIM];

            let w0 = eff_weights_0[i];
            let w1 = eff_weights_1[i];
            let v = eff_values[i];

            let density = v / (w0 + w1 + 1.0);

            emb[0] = v / max_val;
            emb[1] = (w0 / cap_x).min(1.0);
            emb[2] = (w1 / cap_y).min(1.0);
            emb[3] = (density / 5.0).min(1.0);
            emb[4] = (requires_out[i].len() as f32 / 4.0).min(1.0);
            emb[5] = (requires_in[i].len() as f32 / 4.0).min(1.0);
            emb[6] = (conflicts[i].len() as f32 / 4.0).min(1.0);

            for d in 7..EMBED_DIM {
                emb[d] = (emb[d - 7] * 0.5 + emb[(d * 3) % 7] * 0.5).clamp(-1.0, 1.0);
            }

            node_embeddings.push(emb);
        }

        let mut current_emb = node_embeddings;
        let mut next_emb = vec![[0.0f32; EMBED_DIM]; n];

        for _ in 0..MESSAGE_PASSING_ROUNDS {
            for i in 0..n {
                let mut agg = [0.0f32; EMBED_DIM];

                for &req in &requires_out[i] {
                    for d in 0..EMBED_DIM {
                        agg[d] += current_emb[req][d] * 0.4;
                    }
                }

                for &parent in &requires_in[i] {
                    for d in 0..EMBED_DIM {
                        agg[d] += current_emb[parent][d] * 0.6;
                    }
                }

                for &conf in &conflicts[i] {
                    for d in 0..EMBED_DIM {
                        agg[d] -= current_emb[conf][d] * 0.7;
                    }
                }

                for d in 0..EMBED_DIM {
                    let combined = current_emb[i][d] * 0.8 + agg[d] * 0.2;
                    next_emb[i][d] = if combined > 0.0 { combined.min(2.0) } else { 0.0 };
                }
            }
            std::mem::swap(&mut current_emb, &mut next_emb);
        }

        let mut scores = Vec::with_capacity(n);
        for emb in &current_emb {
            let logit = emb[0] * 1.8 - emb[1] * 1.2 - emb[2] * 1.2 + emb[3] * 1.5 - emb[6] * 1.0;
            let prob = 1.0 / (1.0 + (-logit).exp());
            scores.push(prob);
        }

        let (initial_value, initial_selected) = Self::generate_feasible_solution(
            items,
            capacities,
            &requires_out,
            &conflicts,
            &scores,
        );

        let mut decisions = vec![VariableDecision::Free; n];
        let mut is_selected = vec![false; n];
        for &id in &initial_selected {
            if id < n {
                is_selected[id] = true;
            }
        }

        for i in 0..n {
            if scores[i] >= FIX_HIGH_THRESHOLD && is_selected[i] {
                decisions[i] = VariableDecision::Fixed(true);
            } else if scores[i] <= FIX_LOW_THRESHOLD && !is_selected[i] {
                decisions[i] = VariableDecision::Fixed(false);
            }
        }

        let elapsed_ms = timer.elapsed().as_secs_f64() * 1000.0;

        DivingPrediction {
            decisions,
            initial_best_value: initial_value,
            initial_selected_ids: initial_selected,
            confidence_scores: scores,
            execution_time_ms: elapsed_ms,
        }
    }

    fn generate_feasible_solution(
        items: &[Item],
        capacities: &[u32],
        requires_out: &[Vec<usize>],
        conflicts: &[Vec<usize>],
        scores: &[f32],
    ) -> (u32, Vec<usize>) {
        let n = items.len().min(HeuristicBitSet::CAPACITY);
        let num_dims = capacities.len();

        let mut order: Vec<usize> = (0..n).collect();
        order.sort_unstable_by(|&a, &b| {
            scores[b].partial_cmp(&scores[a]).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut selected_mask = HeuristicBitSet::new();
        let mut total_weights = [0u32; 8];
        let mut current_value = 0u32;

        let mut bundle_queue = vec![0usize; n];

        for &candidate in &order {
            if selected_mask.has(candidate) {
                continue;
            }

            let mut bundle_mask = HeuristicBitSet::new();
            let mut q_head = 0;
            let mut q_tail = 0;

            bundle_mask.set(candidate);
            bundle_queue[q_tail] = candidate;
            q_tail += 1;

            while q_head < q_tail {
                let curr = bundle_queue[q_head];
                q_head += 1;

                for &req in &requires_out[curr] {
                    if !bundle_mask.has(req) {
                        bundle_mask.set(req);
                        if q_tail < n {
                            bundle_queue[q_tail] = req;
                            q_tail += 1;
                        }
                    }
                }
            }

            let mut has_conflict = false;
            for i in 0..q_tail {
                let item_id = bundle_queue[i];
                for &conf in &conflicts[item_id] {
                    if selected_mask.has(conf) || bundle_mask.has(conf) {
                        has_conflict = true;
                        break;
                    }
                }
                if has_conflict {
                    break;
                }
            }

            if has_conflict {
                continue;
            }

            let mut next_weights = total_weights;
            let mut bundle_value = 0u32;
            let mut fits = true;

            for i in 0..q_tail {
                let item_id = bundle_queue[i];
                if !selected_mask.has(item_id) {
                    bundle_value += items[item_id].value;
                    for (dim, &w) in items[item_id].weights.iter().enumerate() {
                        if dim < num_dims && dim < 8 {
                            next_weights[dim] += w;
                            if next_weights[dim] > capacities[dim] {
                                fits = false;
                                break;
                            }
                        }
                    }
                    if !fits {
                        break;
                    }
                }
            }

            if fits {
                selected_mask.union_with(&bundle_mask);
                total_weights = next_weights;
                current_value += bundle_value;
            }
        }

        let selected_ids = (0..n)
            .filter(|&i| selected_mask.has(i))
            .collect();

        (current_value, selected_ids)
    }
}