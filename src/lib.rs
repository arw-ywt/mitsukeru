// src/lib.rs

pub mod model;
pub mod bitset;
pub mod dp_branchless;
pub mod sat_compiler;
pub mod branch_and_bound;
pub mod analyzer;
pub mod heuristics;

pub use model::*;
pub use bitset::BitSet;
pub use branch_and_bound::BranchAndBoundSolver;
pub use analyzer::InfeasibilityAnalyzer;
pub use sat_compiler::{CompilerConfig, SatCompiler};
pub use dp_branchless::Flat2dDpSolver;
pub use heuristics::{GraphHeuristics, VariableDecision};

pub const MAX_SUPPORTED_ITEMS: usize = 2048;

use wasm_bindgen::prelude::*;
use web_time::Instant;

struct Timer {
    start: Instant,
}

impl Timer {
    fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
}

pub struct Optimizer<'a> {
    input: &'a ProblemInput,
    config: OptimizerConfig,
}

impl<'a> Optimizer<'a> {
    /// デフォルト構成（preset_fast: 2ms制限）で作成
    pub fn new(input: &'a ProblemInput) -> Self {
        Self { 
            input,
            config: OptimizerConfig::default(),
        }
    }

    /// 設定を指定して作成
    pub fn new_with_config(input: &'a ProblemInput, config: OptimizerConfig) -> Self {
        Self { input, config }
    }

    /// 【プリセット】Web同期 / 低遅延向け（2.0ms / 100,000 nodes）
    pub fn preset_fast(mut self) -> Self {
        self.config = OptimizerConfig::preset_fast();
        self
    }

    /// 【プリセット】バッチ処理・高品質追求向け（100ms / 1,000,000 nodes）
    pub fn preset_accurate(mut self) -> Self {
        self.config = OptimizerConfig::preset_accurate();
        self
    }

    /// 【プリセット】厳密解追求向け（時間無制限）
    pub fn preset_exact(mut self) -> Self {
        self.config = OptimizerConfig::preset_exact();
        self
    }

    pub fn with_strategy(mut self, strategy: SolverStrategy) -> Self {
        self.config.strategy = strategy;
        self
    }

    pub fn with_time_limit_ms(mut self, limit_ms: f64) -> Self {
        self.config.time_limit_ms = Some(limit_ms);
        self
    }

    pub fn with_max_nodes(mut self, max_nodes: u64) -> Self {
        self.config.max_nodes = Some(max_nodes);
        self
    }

    pub fn with_top_k(mut self, k: usize) -> Self {
        self.config.top_k = Some(k);
        self
    }
    
    pub fn solve(&self) -> Result<Solution, OptimizeError> {
        let timer = Timer::start();
        let item_count = self.input.items.len();
        let rule_count = self.input.rules.len();

        if item_count > MAX_SUPPORTED_ITEMS {
            return Err(OptimizeError::InvalidInput {
                message: format!(
                    "Item count ({}) exceeds the maximum supported limit of {} items. Please pre-filter candidates.",
                    item_count, MAX_SUPPORTED_ITEMS
                ),
            });
        }

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

        let mut solver = BranchAndBoundSolver::new(
            &self.input.items,
            &self.input.capacities,
            &self.input.rules,
        );

        if let Some(k) = self.config.top_k {
            solver.set_top_k(k);
        }

        if use_heuristic {
            let prediction = GraphHeuristics::predict(
                &self.input.items,
                &self.input.capacities,
                &self.input.rules,
            );
            solver.set_initial_solution(prediction.initial_best_value, &prediction.initial_selected_ids);
        }

        if let Some(limit) = self.config.time_limit_ms {
            solver.set_time_limit_ms(limit);
        }
        if let Some(max_nodes) = self.config.max_nodes {
            solver.set_max_nodes(max_nodes);
        }

        let (best_value, mut selected_ids, explored_nodes, status) = solver.solve();
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
        let report = analyzer.analyze();

        if !report.is_feasible {
            return Err(OptimizeError::Infeasible { report });
        }

        let execution_time_ms = timer.elapsed_ms();

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

pub fn solve_branchless_dp_core(input: &ProblemInput) -> Result<DpResult, String> {
    if input.capacities.len() < 2 {
        return Err("At least 2 capacities required for 2D DP".to_string());
    }

    let c1 = input.capacities[0] as usize;
    let c2 = input.capacities[1] as usize;
    let n = input.items.len();
    let grid_size = (c1 + 1) * (c2 + 1);
    let total_bytes = (grid_size * std::mem::size_of::<u32>()) 
    + (n * grid_size * std::mem::size_of::<u8>());
    let memory_mb = total_bytes as f64 / (1024.0 * 1024.0);

    let timer = Timer::start();

    let mut solver = Flat2dDpSolver::new(input.capacities[0], input.capacities[1]);
    let (best_value, selected_item_ids) = solver.solve(&input.items);

    let execution_time_ms = timer.elapsed_ms();

    Ok(DpResult {
        best_value,
        selected_item_ids,
        execution_time_ms,
        memory_mb,
    })
}

pub fn compile_to_3sat_core(input: &ProblemInput) -> SatReductionResult {
    let target_val = input.target_value.unwrap_or(0);
    let config = CompilerConfig::default();

    let (dimacs, num_vars, num_clauses, var_map) = SatCompiler::compile(
        &input.items,
        &input.capacities,
        &input.rules,
        target_val,
        &config,
    );

    SatReductionResult {
        num_variables: num_vars,
        num_clauses,
        dimacs_cnf: dimacs,
        variable_map: var_map,
    }
}

pub fn solve_branch_and_bound_core(input: &ProblemInput) -> BnbResult {
    let timer = Timer::start();

    let mut solver = BranchAndBoundSolver::new(&input.items, &input.capacities, &input.rules);
    
    let time_limit = if cfg!(debug_assertions) { 50.0 } else { 2.0 };
    solver.set_time_limit_ms(time_limit);

    let max_nodes = 100_000;
    solver.set_max_nodes(max_nodes);

    let (best_value, selected_item_ids, explored_nodes, status, ..) = solver.solve();

    let execution_time_ms = timer.elapsed_ms();

    BnbResult {
        best_value,
        selected_item_ids,
        execution_time_ms,
        explored_nodes,
        status,
    }
}

pub fn analyze_infeasibility_core(input: &ProblemInput, bnb_best_value: u32) -> ConflictReport {
    let analyzer = InfeasibilityAnalyzer::new(
        &input.items,
        &input.capacities,
        &input.rules,
        input.target_value,
        bnb_best_value,
    );
    analyzer.analyze()
}



#[wasm_bindgen]
pub fn optimize(input_value: JsValue, config_value: Option<JsValue>) -> Result<JsValue, JsValue> {
    let (problem, config) = if let Some(cfg_js) = config_value.filter(|v| !v.is_undefined() && !v.is_null()) {
        let problem: ProblemInput = serde_wasm_bindgen::from_value(input_value)?;
        let config: OptimizerConfig = serde_wasm_bindgen::from_value(cfg_js)?;
        (problem, config)
    } else if let Ok(req) = serde_wasm_bindgen::from_value::<OptimizeRequest>(input_value.clone()) {
        (req.problem, req.config.unwrap_or_default())
    } else {
        let problem: ProblemInput = serde_wasm_bindgen::from_value(input_value)?;
        (problem, OptimizerConfig::default())
    };

    let optimizer = Optimizer::new_with_config(&problem, config);
    match optimizer.solve() {
        Ok(solution) => Ok(serde_wasm_bindgen::to_value(&solution)?),
        Err(err) => {
            let err_obj = serde_wasm_bindgen::to_value(&err)?;
            Err(err_obj)
        }
    }
}

#[wasm_bindgen]
pub fn solve_branchless_dp(input_value: JsValue) -> Result<JsValue, JsValue> {
    let input: ProblemInput = serde_wasm_bindgen::from_value(input_value)?;
    let res = solve_branchless_dp_core(&input)
        .map_err(|e| JsValue::from_str(&e))?;
    Ok(serde_wasm_bindgen::to_value(&res)?)
}

#[wasm_bindgen]
pub fn compile_to_3sat(input_value: JsValue) -> Result<JsValue, JsValue> {
    let input: ProblemInput = serde_wasm_bindgen::from_value(input_value)?;
    let res = compile_to_3sat_core(&input);
    Ok(serde_wasm_bindgen::to_value(&res)?)
}

#[wasm_bindgen]
pub fn solve_branch_and_bound(input_value: JsValue) -> Result<JsValue, JsValue> {
    let input: ProblemInput = serde_wasm_bindgen::from_value(input_value)?;
    let res = solve_branch_and_bound_core(&input);
    Ok(serde_wasm_bindgen::to_value(&res)?)
}

#[wasm_bindgen]
pub fn analyze_infeasibility(input_value: JsValue, bnb_best_value: u32) -> Result<JsValue, JsValue> {
    let input: ProblemInput = serde_wasm_bindgen::from_value(input_value)?;
    let res = analyze_infeasibility_core(&input, bnb_best_value);
    Ok(serde_wasm_bindgen::to_value(&res)?)
}

#[wasm_bindgen]
pub fn predict_heuristics(input_value: JsValue) -> Result<JsValue, JsValue> {
    let input: ProblemInput = serde_wasm_bindgen::from_value(input_value)?;
    let prediction = GraphHeuristics::predict(&input.items, &input.capacities, &input.rules);
    Ok(serde_wasm_bindgen::to_value(&prediction.initial_selected_ids)?)
}

pub use schemars;
pub use serde_json;


#[cfg(test)]
mod tests {
use super::*;

#[test]
fn test_optimizer_high_level_api() {
    let input = ProblemInput {
        items: vec![
            Item { id: 0, name: "A".into(), weights: vec![10], value: 100 },
            Item { id: 1, name: "B".into(), weights: vec![10], value: 200 },
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
    fn test_optimizer_infeasible_conflict() {
        let input = ProblemInput {
            items: vec![
                Item { id: 0, name: "A".into(), weights: vec![10], value: 100 },
                Item { id: 1, name: "B".into(), weights: vec![10], value: 200 },
            ],
            capacities: vec![50],
            rules: vec![
                BusinessRule::Requires { from_id: 0, to_id: 1 },
                BusinessRule::Conflicts { a_id: 0, b_id: 1 },
            ],
            target_value: None,
        };

        let optimizer = Optimizer::new(&input);
        match optimizer.solve() {
            Err(OptimizeError::Infeasible { report }) => {
                assert!(!report.is_feasible);
                assert!(!report.logical_chain_conflicts.is_empty());
            }
            _ => panic!("Expected infeasible error"),
        }
    }
}
