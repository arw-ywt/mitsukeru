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

type HeuristicBitSet = BitSet<32>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableDecision {
    Fixed(bool),
    Free,
}

type GraphAdjList = (Vec<Vec<usize>>, Vec<Vec<usize>>, Vec<Vec<usize>>);

#[derive(Debug, Clone)]
pub struct HeuristicPrediction {
    pub decisions: Vec<VariableDecision>,
    pub initial_best_value: u32,
    pub initial_selected_ids: Vec<usize>,
    pub confidence_scores: Vec<f32>,
    pub execution_time_ms: f64,
}

pub struct GraphHeuristics;

struct EmbeddingContext<'a> {
    capacities: &'a [u32],
    eff_w0: &'a [f64],
    eff_w1: &'a [f64],
    eff_v: &'a [f64],
    requires_out: &'a [Vec<usize>],
    requires_in: &'a [Vec<usize>],
    conflicts: &'a [Vec<usize>],
}

impl GraphHeuristics {
    #[must_use]
    pub fn predict(
        items: &[Item],
        capacities: &[u32],
        rules: &[BusinessRule],
    ) -> HeuristicPrediction {
        let timer = Instant::now();
        let n = items.len().min(HeuristicBitSet::CAPACITY);

        if n == 0 {
            return HeuristicPrediction {
                decisions: Vec::new(),
                initial_best_value: 0,
                initial_selected_ids: Vec::new(),
                confidence_scores: Vec::new(),
                execution_time_ms: 0.0,
            };
        }

        let (requires_out, requires_in, conflicts) = Self::build_graphs(n, rules);
        let (eff_w0, eff_w1, eff_v) = Self::compute_effective_weights(items, n, &requires_out);

        let ctx = EmbeddingContext {
            capacities,
            eff_w0: &eff_w0,
            eff_w1: &eff_w1,
            eff_v: &eff_v,
            requires_out: &requires_out,
            requires_in: &requires_in,
            conflicts: &conflicts,
        };

        
        let node_embeddings = Self::build_initial_embeddings(n, &ctx);

        let final_embeddings = Self::run_message_passing(
            n,
            node_embeddings,
            &requires_out,
            &requires_in,
            &conflicts,
        );


        let scores = Self::compute_scores(&final_embeddings);

        let (initial_value, initial_selected) = Self::generate_feasible_solution(
            items,
            capacities,
            &requires_out,
            &conflicts,
            &scores,
        );

        let decisions = Self::make_decisions(n, &scores, &initial_selected);
        let elapsed_ms = timer.elapsed().as_secs_f64() * 1000.0;

        HeuristicPrediction {
            decisions,
            initial_best_value: initial_value,
            initial_selected_ids: initial_selected,
            confidence_scores: scores,
            execution_time_ms: elapsed_ms,
        }
    }

    fn build_graphs(
        n: usize,
        rules: &[BusinessRule],
    ) -> GraphAdjList {
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
        (requires_out, requires_in, conflicts)
    }

    fn compute_effective_weights(
        items: &[Item],
        n: usize,
        requires_out: &[Vec<usize>],
    ) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let mut eff_w0 = vec![0.0f64; n];
        let mut eff_w1 = vec![0.0f64; n];
        let mut eff_v = vec![0.0f64; n];
        let mut queue = vec![0usize; n];

        for i in 0..n {
            let mut visited = HeuristicBitSet::new();
            let mut q_head = 0;
            let mut q_tail = 0;

            visited.set(i);
            queue[q_tail] = i;
            q_tail += 1;

            let mut tot_w0 = 0.0f64;
            let mut tot_w1 = 0.0f64;
            let mut tot_v = 0.0f64;

            while q_head < q_tail {
                let curr = queue[q_head];
                q_head += 1;

                tot_w0 += f64::from(items[curr].weights.first().copied().unwrap_or(0));
                tot_w1 += f64::from(items[curr].weights.get(1).copied().unwrap_or(0));
                tot_v += f64::from(items[curr].value);

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
            eff_w0[i] = tot_w0;
            eff_w1[i] = tot_w1;
            eff_v[i] = tot_v;
        }
        (eff_w0, eff_w1, eff_v)
    }

    fn build_initial_embeddings(
        n: usize,
        ctx: &EmbeddingContext,
    ) -> Vec<[f64; EMBED_DIM]> {
        let cap_x = f64::from(ctx.capacities.first().copied().unwrap_or(1).max(1));
        let cap_y = f64::from(ctx.capacities.get(1).copied().unwrap_or(1).max(1));
        let max_val = ctx.eff_v.iter().copied().fold(1.0f64, f64::max).max(1.0);
        let mut node_embeddings = Vec::with_capacity(n);

        for i in 0..n {
            let mut emb = [0.0f64; EMBED_DIM];

            let w0 = ctx.eff_w0[i];
            let w1 = ctx.eff_w1[i];
            let v = ctx.eff_v[i];
            let density = v / (w0 + w1 + 1.0);

            let req_out_cnt = f64::from(u32::try_from(ctx.requires_out[i].len()).unwrap_or(u32::MAX));
            let req_in_cnt = f64::from(u32::try_from(ctx.requires_in[i].len()).unwrap_or(u32::MAX));
            let conf_cnt = f64::from(u32::try_from(ctx.conflicts[i].len()).unwrap_or(u32::MAX));

            emb[0] = v / max_val;
            emb[1] = (w0 / cap_x).min(1.0);
            emb[2] = (w1 / cap_y).min(1.0);
            emb[3] = (density / 5.0).min(1.0);
            emb[4] = (req_out_cnt / 4.0).min(1.0);
            emb[5] = (req_in_cnt / 4.0).min(1.0);
            emb[6] = (conf_cnt / 4.0).min(1.0);

            for d in 7..EMBED_DIM {
                emb[d] = (emb[d - 7] * 0.5 + emb[(d * 3) % 7] * 0.5).clamp(-1.0, 1.0);
            }

            node_embeddings.push(emb);
        }
        node_embeddings
    }


    fn run_message_passing(
        n: usize,
        mut current_emb: Vec<[f64; EMBED_DIM]>,
        requires_out: &[Vec<usize>],
        requires_in: &[Vec<usize>],
        conflicts: &[Vec<usize>],
    ) -> Vec<[f64; EMBED_DIM]> {
        let mut next_emb = vec![[0.0f64; EMBED_DIM]; n];

        for _ in 0..MESSAGE_PASSING_ROUNDS {
            for i in 0..n {
                let mut agg = [0.0f64; EMBED_DIM];

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
        current_emb
    }

    fn compute_scores(embeddings: &[[f64; EMBED_DIM]]) -> Vec<f32> {
        let mut scores = Vec::with_capacity(embeddings.len());
        for emb in embeddings {
            let logit = emb[0] * 1.8 - emb[1] * 1.2 - emb[2] * 1.2 + emb[3] * 1.5 - emb[6] * 1.0;
            let prob_f64 = 1.0 / (1.0 + (-logit).exp());
            let prob_f32 = prob_f64.to_string().parse::<f32>().unwrap_or(0.5);
            scores.push(prob_f32);
        }
        scores
    }

    fn make_decisions(
        n: usize,
        scores: &[f32],
        initial_selected: &[usize],
    ) -> Vec<VariableDecision> {
        let mut decisions = vec![VariableDecision::Free; n];
        let mut is_selected = vec![false; n];
        for &id in initial_selected {
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
        decisions
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
            for w in bundle_queue.iter().take(q_tail) {
                for &conf in &conflicts[*w] {
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

            for w in bundle_queue.iter().take(q_tail) {
                if !selected_mask.has(*w) {
                    bundle_value += items[*w].value;
                    for (dim, &w) in items[*w].weights.iter().enumerate() {
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