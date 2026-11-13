// src/lib.rs
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

use alloc::string::String;

pub mod analyzer;
pub mod bitset;
pub mod branch_and_bound;
pub mod clock;
pub mod dp_branchless;
pub mod heuristics;
pub mod model;
pub mod sat_compiler;
pub mod validation;

pub use analyzer::InfeasibilityAnalyzer;
pub use bitset::BitSet;
pub use branch_and_bound::BranchAndBoundSolver;
pub use clock::Clock;
#[cfg(feature = "std")]
pub use clock::SystemClock;
pub use dp_branchless::Flat2dDpSolver;
pub use heuristics::{GraphHeuristics, VariableDecision};
pub use model::*;
pub use sat_compiler::{CompilerConfig, SatCompileError, SatCompiler};
pub use validation::{
    DEFAULT_MAX_DP_MEMORY_BYTES, DEFAULT_MAX_SAT_CLAUSES, DEFAULT_MAX_SAT_VARS, MAX_CAPACITY_DIMS,
    MAX_SUPPORTED_ITEMS, MAX_SUPPORTED_RULES, MAX_TOP_K, validate_optimizer_config,
    validate_problem_input,
};

pub struct Optimizer<'a> {
    input: &'a ProblemInput,
    config: OptimizerConfig,
    clock: Option<&'a dyn Clock>,
}

impl<'a> Optimizer<'a> {
    #[must_use]
    pub fn new(input: &'a ProblemInput) -> Self {
        Self {
            input,
            config: OptimizerConfig::default(),
            #[cfg(feature = "std")]
            clock: Some(&clock::DEFAULT_CLOCK),
            #[cfg(not(feature = "std"))]
            clock: None,
        }
    }

    #[must_use]
    pub fn new_with_config(input: &'a ProblemInput, config: OptimizerConfig) -> Self {
        Self {
            input,
            config,
            #[cfg(feature = "std")]
            clock: Some(&clock::DEFAULT_CLOCK),
            #[cfg(not(feature = "std"))]
            clock: None,
        }
    }

    #[must_use]
    pub fn with_clock(mut self, clock: &'a dyn Clock) -> Self {
        self.clock = Some(clock);
        self
    }

    #[must_use]
    pub fn preset_fast(mut self) -> Self {
        self.config = OptimizerConfig::preset_fast();
        self
    }

    #[must_use]
    pub fn preset_accurate(mut self) -> Self {
        self.config = OptimizerConfig::preset_accurate();
        self
    }

    #[must_use]
    pub fn preset_exact(mut self) -> Self {
        self.config = OptimizerConfig::preset_exact();
        self
    }

    #[must_use]
    pub fn with_strategy(mut self, strategy: SolverStrategy) -> Self {
        self.config.strategy = strategy;
        self
    }

    #[must_use]
    pub fn with_time_limit_ms(mut self, limit_ms: f64) -> Self {
        self.config.time_limit_ms = Some(limit_ms);
        self
    }

    #[must_use]
    pub fn with_max_nodes(mut self, max_nodes: u64) -> Self {
        self.config.max_nodes = Some(max_nodes);
        self
    }

    #[must_use]
    pub fn with_top_k(mut self, k: usize) -> Self {
        self.config.top_k = Some(k);
        self
    }

    /// Optimizes the problem instance and calculates the optimal solution.
    ///
    /// # Errors
    ///
    /// Returns an [`OptimizeError`] in the following cases:
    /// - If input sizes, dimensions, rules, `top_k`, `time_limit_ms`, or value sums violate supported ceilings ([`OptimizeError::InvalidInput`]).
    /// - If `time_limit_ms` is set without an available `Clock` in `no_std` mode ([`OptimizeError::InvalidInput`]).
    /// - If the problem is determined to be infeasible ([`OptimizeError::Infeasible`]).
    pub fn solve(&self) -> Result<Solution, OptimizeError> {
        validate_problem_input(&self.input.items, &self.input.capacities, &self.input.rules)?;
        validate_optimizer_config(&self.config)?;

        if self.config.time_limit_ms.is_some() && self.clock.is_none() {
            return Err(OptimizeError::InvalidInput {
                message: String::from(
                    "time_limit_ms was specified, but no Clock is available in this environment. In no_std mode, provide a hardware Clock via `.with_clock()` or use node limits via `.with_max_nodes()`.",
                ),
            });
        }

        let start_micros = self.clock.map(Clock::now_micros);
        let item_count = self.input.items.len();
        let rule_count = self.input.rules.len();

        let (use_heuristic, strategy_used) = match self.config.strategy {
            SolverStrategy::Pure => (false, SolverStrategy::Pure),
            SolverStrategy::Heuristic => (true, SolverStrategy::Heuristic),
            SolverStrategy::Mixed => {
                let heuristic_recommended = rule_count > 0
                    && item_count >= 20
                    && (item_count <= 64 || rule_count * 4 >= item_count);
                if heuristic_recommended {
                    (true, SolverStrategy::Heuristic)
                } else {
                    (false, SolverStrategy::Pure)
                }
            }
        };

        let mut solver =
            BranchAndBoundSolver::new(&self.input.items, &self.input.capacities, &self.input.rules);

        if let Some(clk) = self.clock {
            solver.set_clock(clk);
        }

        if let Some(k) = self.config.top_k {
            solver.set_top_k(k);
        }

        if use_heuristic {
            let prediction = GraphHeuristics::predict(
                &self.input.items,
                &self.input.capacities,
                &self.input.rules,
            )?;
            solver.set_initial_solution(
                prediction.initial_best_value,
                &prediction.initial_selected_ids,
            );
        }

        if let Some(limit) = self.config.time_limit_ms {
            solver.set_time_limit_ms(limit);
        }
        if let Some(max_nodes) = self.config.max_nodes {
            solver.set_max_nodes(max_nodes);
        }

        let (best_value, mut selected_ids, explored_nodes, status) = solver.solve()?;
        selected_ids.sort_unstable();

        let upper_bound = solver.upper_bound();
        let optimality_gap = solver.optimality_gap();
        let top_candidates = if self.config.top_k.is_some() {
            Some(solver.take_candidates())
        } else {
            None
        };

        let analyzer = InfeasibilityAnalyzer::new(
            &self.input.items,
            &self.input.capacities,
            &self.input.rules,
            self.input.target_value,
            best_value,
        );
        let report = analyzer.analyze()?;

        if !report.is_feasible {
            return Err(OptimizeError::Infeasible { report });
        }

        let execution_time_ms = if let (Some(clk), Some(start)) = (self.clock, start_micros) {
            clock::elapsed_ms(start, clk.now_micros())
        } else {
            0.0
        };

        Ok(Solution {
            best_value,
            selected_item_ids: selected_ids,
            execution_time_ms,
            explored_nodes,
            status,
            strategy_used: Some(strategy_used),
            top_candidates,
            upper_bound,
            optimality_gap,
        })
    }
}

/// Solves the problem exactly using 2D dynamic programming (Flat 2D DP).
///
/// # Errors
///
/// Returns [`OptimizeError::InvalidInput`] if fewer than 2 capacities are provided,
/// input ceilings are violated, or the DP memory budget is exceeded.
pub fn solve_branchless_dp_core(input: &ProblemInput) -> Result<DpResult, OptimizeError> {
    const BYTES_PER_MB: usize = 1024 * 1024;
    const BYTES_PER_MB_F64: f64 = 1024.0 * 1024.0;

    validate_problem_input(&input.items, &input.capacities, &input.rules)?;

    if input.capacities.len() < 2 {
        return Err(OptimizeError::InvalidInput {
            message: String::from("At least 2 capacities required for 2D DP"),
        });
    }

    #[cfg(feature = "std")]
    let clock: Option<&dyn Clock> = Some(&clock::DEFAULT_CLOCK);
    #[cfg(not(feature = "std"))]
    let clock: Option<&dyn Clock> = None;

    let start_micros = clock.map(Clock::now_micros);

    let mut solver = Flat2dDpSolver::new(input.capacities[0], input.capacities[1])?;
    let (best_value, selected_item_ids) = solver.solve(&input.items)?;
    let (_, _, total_bytes) = solver.allocated_bytes();

    let mb_whole = u32::try_from(total_bytes / BYTES_PER_MB).unwrap_or(u32::MAX);
    let mb_rem = u32::try_from(total_bytes % BYTES_PER_MB).unwrap_or(0);
    let memory_mb = f64::from(mb_whole) + (f64::from(mb_rem) / BYTES_PER_MB_F64);

    let execution_time_ms = if let (Some(clk), Some(start)) = (clock, start_micros) {
        clock::elapsed_ms(start, clk.now_micros())
    } else {
        0.0
    };

    Ok(DpResult {
        best_value,
        selected_item_ids,
        execution_time_ms,
        memory_mb,
    })
}

/// Reduces the combinatorial problem into 3-SAT (DIMACS CNF format).
///
/// # Errors
///
/// Returns [`SatCompileError`] if input/circuit limits are exceeded, a business rule contains an out-of-bounds item ID, or variable allocation overflows.
pub fn compile_to_3sat_core(input: &ProblemInput) -> Result<SatReductionResult, SatCompileError> {
    let target_val = input.target_value.unwrap_or(0);
    let config = CompilerConfig::default();

    let (dimacs, num_vars, num_clauses, var_map) = SatCompiler::compile(
        &input.items,
        &input.capacities,
        &input.rules,
        target_val,
        &config,
    )?;

    Ok(SatReductionResult {
        num_variables: num_vars,
        num_clauses,
        dimacs_cnf: dimacs,
        variable_map: var_map,
    })
}

/// Executes the standalone Branch-and-Bound solver with default fast limits.
///
/// # Errors
///
/// Returns [`OptimizeError::InvalidInput`] if problem bounds exceed safety ceilings.
pub fn solve_branch_and_bound_core(input: &ProblemInput) -> Result<BnbResult, OptimizeError> {
    validate_problem_input(&input.items, &input.capacities, &input.rules)?;

    #[cfg(feature = "std")]
    let clock: Option<&dyn Clock> = Some(&clock::DEFAULT_CLOCK);
    #[cfg(not(feature = "std"))]
    let clock: Option<&dyn Clock> = None;

    let start_micros = clock.map(Clock::now_micros);

    let mut solver = BranchAndBoundSolver::new(&input.items, &input.capacities, &input.rules);

    if let Some(clk) = clock {
        solver.set_clock(clk);
        let time_limit = if cfg!(debug_assertions) { 50.0 } else { 2.0 };
        solver.set_time_limit_ms(time_limit);
    }

    let max_nodes = 100_000;
    solver.set_max_nodes(max_nodes);

    let (best_value, selected_item_ids, explored_nodes, status) = solver.solve()?;

    let execution_time_ms = if let (Some(clk), Some(start)) = (clock, start_micros) {
        clock::elapsed_ms(start, clk.now_micros())
    } else {
        0.0
    };

    Ok(BnbResult {
        best_value,
        selected_item_ids,
        execution_time_ms,
        explored_nodes,
        status,
    })
}

/// Performs static infeasibility analysis on the problem input.
///
/// # Errors
///
/// Returns [`OptimizeError::InvalidInput`] if problem bounds exceed safety ceilings.
pub fn analyze_infeasibility_core(
    input: &ProblemInput,
    bnb_best_value: u32,
) -> Result<ConflictReport, OptimizeError> {
    let analyzer = InfeasibilityAnalyzer::new(
        &input.items,
        &input.capacities,
        &input.rules,
        input.target_value,
        bnb_best_value,
    );
    analyzer.analyze()
}

#[cfg(feature = "schemars")]
pub use schemars;
#[cfg(feature = "serde")]
pub use serde_json;

#[cfg(all(test, feature = "std"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "std")]
    fn test_optimizer_high_level_api() {
        let input = ProblemInput {
            items: vec![
                Item {
                    id: 0,
                    name: "A".into(),
                    weights: vec![10],
                    value: 100,
                },
                Item {
                    id: 1,
                    name: "B".into(),
                    weights: vec![10],
                    value: 200,
                },
            ],
            capacities: vec![15],
            rules: vec![],
            target_value: None,
        };

        let optimizer = Optimizer::new(&input);
        let solution = optimizer.solve().expect("Should find optimal solution");

        assert_eq!(solution.best_value, 200);
        assert_eq!(solution.selected_item_ids, vec![1]);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_optimizer_infeasible_conflict() {
        let input = ProblemInput {
            items: vec![
                Item {
                    id: 0,
                    name: "A".into(),
                    weights: vec![10],
                    value: 100,
                },
                Item {
                    id: 1,
                    name: "B".into(),
                    weights: vec![10],
                    value: 200,
                },
            ],
            capacities: vec![50],
            rules: vec![
                BusinessRule::Requires {
                    from_id: 0,
                    to_id: 1,
                },
                BusinessRule::Conflicts { a_id: 0, b_id: 1 },
            ],
            target_value: None,
        };

        let optimizer = Optimizer::new(&input);
        let res = optimizer.solve();

        assert!(matches!(res, Err(OptimizeError::Infeasible { .. })));

        if let Err(OptimizeError::Infeasible { report }) = res {
            assert!(!report.is_feasible);
            assert!(!report.logical_chain_conflicts.is_empty());
        }
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_core_entrypoints_enforce_limits() {
        let oversized_dp = ProblemInput {
            items: vec![],
            capacities: vec![u32::MAX, u32::MAX],
            rules: vec![],
            target_value: None,
        };
        assert!(matches!(
            solve_branchless_dp_core(&oversized_dp),
            Err(OptimizeError::InvalidInput { .. })
        ));

        let too_many_dims = ProblemInput {
            items: vec![Item {
                id: 0,
                name: "A".into(),
                weights: vec![1; 9],
                value: 10,
            }],
            capacities: vec![10; 9],
            rules: vec![],
            target_value: None,
        };
        assert!(matches!(
            solve_branch_and_bound_core(&too_many_dims),
            Err(OptimizeError::InvalidInput { .. })
        ));
        assert!(matches!(
            analyze_infeasibility_core(&too_many_dims, 0),
            Err(OptimizeError::InvalidInput { .. })
        ));
    }
}
