// src/heuristics.rs

use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::mem;

use crate::bitset::BitSet;
use crate::clock::Clock;
use crate::model::{BusinessRule, Item, OptimizeError};
use crate::validation::{MAX_CAPACITY_DIMS, validate_problem_input};

pub const EMBED_DIM: usize = 8;
const MESSAGE_PASSING_ROUNDS: usize = 2;
const FIX_HIGH_THRESHOLD: f64 = 0.70;
const FIX_LOW_THRESHOLD: f64 = 0.30;

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
    pub confidence_scores: Vec<f64>,
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
    /// Predicts initial variable decisions and a warm-start feasible solution.
    ///
    /// # Errors
    ///
    /// Returns [`OptimizeError::InvalidInput`] if problem bounds exceed safety ceilings.
    pub fn predict(
        items: &[Item],
        capacities: &[u32],
        rules: &[BusinessRule],
    ) -> Result<HeuristicPrediction, OptimizeError> {
        #[cfg(feature = "std")]
        {
            Self::predict_with_clock(items, capacities, rules, Some(&crate::clock::DEFAULT_CLOCK))
        }
        #[cfg(not(feature = "std"))]
        {
            Self::predict_with_clock(items, capacities, rules, None)
        }
    }

    /// Predicts initial variable decisions using an injected clock.
    ///
    /// # Errors
    ///
    /// Returns [`OptimizeError::InvalidInput`] if problem bounds exceed safety ceilings.
    pub fn predict_with_clock(
        items: &[Item],
        capacities: &[u32],
        rules: &[BusinessRule],
        clock: Option<&dyn Clock>,
    ) -> Result<HeuristicPrediction, OptimizeError> {
        validate_problem_input(items, capacities, rules)?;

        let start_micros = clock.map(Clock::now_micros);
        let n = items.len().min(HeuristicBitSet::CAPACITY);

        if n == 0 {
            return Ok(HeuristicPrediction {
                decisions: Vec::new(),
                initial_best_value: 0,
                initial_selected_ids: Vec::new(),
                confidence_scores: Vec::new(),
                execution_time_ms: 0.0,
            });
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

        let final_embeddings =
            Self::run_message_passing(n, node_embeddings, &requires_out, &requires_in, &conflicts);

        let scores = Self::compute_scores(&final_embeddings);

        let (initial_value, initial_selected) =
            Self::generate_feasible_solution(items, capacities, &requires_out, &conflicts, &scores);

        let decisions = Self::make_decisions(n, &scores, &initial_selected);
        let elapsed_ms = if let (Some(clk), Some(start)) = (clock, start_micros) {
            crate::clock::elapsed_ms(start, clk.now_micros())
        } else {
            0.0
        };

        Ok(HeuristicPrediction {
            decisions,
            initial_best_value: initial_value,
            initial_selected_ids: initial_selected,
            confidence_scores: scores,
            execution_time_ms: elapsed_ms,
        })
    }

    fn build_graphs(n: usize, rules: &[BusinessRule]) -> GraphAdjList {
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

    fn build_initial_embeddings(n: usize, ctx: &EmbeddingContext) -> Vec<[f64; EMBED_DIM]> {
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

            let req_out_cnt =
                f64::from(u32::try_from(ctx.requires_out[i].len()).unwrap_or(u32::MAX));
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
                    next_emb[i][d] = if combined > 0.0 {
                        combined.min(2.0)
                    } else {
                        0.0
                    };
                }
            }
            mem::swap(&mut current_emb, &mut next_emb);
        }
        current_emb
    }

    fn compute_scores(embeddings: &[[f64; EMBED_DIM]]) -> Vec<f64> {
        let mut scores = Vec::with_capacity(embeddings.len());
        for emb in embeddings {
            let logit = emb[0] * 1.8 - emb[1] * 1.2 - emb[2] * 1.2 + emb[3] * 1.5 - emb[6] * 1.0;
            let prob_f64 = sigmoid(logit);
            scores.push(prob_f64);
        }
        scores
    }

    fn make_decisions(
        n: usize,
        scores: &[f64],
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
        scores: &[f64],
    ) -> (u32, Vec<usize>) {
        let n = items.len().min(HeuristicBitSet::CAPACITY);
        let num_dims = capacities.len().min(MAX_CAPACITY_DIMS);

        let mut order: Vec<usize> = (0..n).collect();
        order.sort_unstable_by(|&a, &b| {
            scores[b].partial_cmp(&scores[a]).unwrap_or(Ordering::Equal)
        });

        let mut selected_mask = HeuristicBitSet::new();
        let mut total_weights = [0u64; MAX_CAPACITY_DIMS];
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

            for idx in bundle_queue.iter().take(q_tail) {
                if !selected_mask.has(*idx) {
                    bundle_value = bundle_value.saturating_add(items[*idx].value);
                    for (dim, &w) in items[*idx].weights.iter().enumerate().take(num_dims) {
                        next_weights[dim] += u64::from(w);
                        if next_weights[dim] > u64::from(capacities[dim]) {
                            fits = false;
                            break;
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
                current_value = current_value.saturating_add(bundle_value);
            }
        }

        let selected_ids = (0..n).filter(|&i| selected_mask.has(i)).collect();

        (current_value, selected_ids)
    }
}

#[inline]
fn sigmoid(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        1.0 / (1.0 + (-x).exp())
    }
    #[cfg(all(not(feature = "std"), feature = "libm"))]
    {
        1.0 / (1.0 + libm::exp(-x))
    }
    #[cfg(all(not(feature = "std"), not(feature = "libm")))]
    {
        const LN2: f64 = core::f64::consts::LN_2;
        const INV_LN2: f64 = core::f64::consts::LOG2_E;

        if x.is_nan() {
            return f64::NAN;
        }
        let t = -x;
        if t >= 50.0 {
            return 0.0;
        }
        if t <= -50.0 {
            return 1.0;
        }

        let k_f = t * INV_LN2;
        #[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
        let k = (if k_f >= 0.0 { k_f + 0.5 } else { k_f - 0.5 }) as i32;

        let r = t - f64::from(k) * LN2;

        let exp_r = 1.0
            + r * (1.0
                + r * (0.5 + r * (1.0 / 6.0 + r * (1.0 / 24.0 + r * (1.0 / 120.0 + r / 720.0)))));
        let exp_bits = u64::try_from(k + 1023).unwrap_or(1023) << 52;
        let pow2_k = f64::from_bits(exp_bits);

        1.0 / (1.0 + exp_r * pow2_k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heuristics_rejects_excess_dims() {
        let items = vec![Item {
            id: 0,
            name: "A".into(),
            weights: vec![1; 9],
            value: 10,
        }];
        let res = GraphHeuristics::predict(&items, &[10; 9], &[]);
        assert!(matches!(res, Err(OptimizeError::InvalidInput { .. })));
    }

    #[test]
    fn test_heuristics_weight_u32_overflow_does_not_bypass_capacity() {
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
        let pred = GraphHeuristics::predict(&items, &[u32::MAX], &[]).expect("prediction succeeds");
        assert!(pred.initial_selected_ids.len() <= 1);
    }
}
