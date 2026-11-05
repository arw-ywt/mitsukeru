// lib.rs

pub mod model;
pub mod dp_branchless;
pub mod sat_compiler;
pub mod branch_and_bound;
pub mod analyzer;

pub use model::*;
pub use dp_branchless::Flat2dDpSolver;
pub use sat_compiler::{CompilerConfig, SatCompiler};
pub use branch_and_bound::BranchAndBoundSolver;
pub use analyzer::InfeasibilityAnalyzer;

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

pub fn solve_knapsack_branchless_dp_core(input: &KnapsackInput) -> Result<DpResult, String> {
    if input.capacities.len() < 2 {
        return Err("At least 2 capacities required for 2D DP".to_string());
    }

    let c1 = input.capacities[0] as usize;
    let c2 = input.capacities[1] as usize;
    let n = input.items.len();
    let grid_size = (c1 + 1) * (c2 + 1);
    let total_bytes = (grid_size * 4) + (n * grid_size * 1);
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

pub fn compile_to_3sat_core(input: &KnapsackInput) -> SatReductionResult {
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

pub fn solve_knapsack_branch_and_bound_core(input: &KnapsackInput) -> MipResult {
    let timer = Timer::start();

    let mut solver = BranchAndBoundSolver::new(&input.items, &input.capacities, &input.rules);
    let (best_value, selected_item_ids, explored_nodes) = solver.solve();

    let execution_time_ms = timer.elapsed_ms();

    MipResult {
        best_value,
        selected_item_ids,
        execution_time_ms,
        explored_nodes,
    }
}

pub fn analyze_infeasibility_core(input: &KnapsackInput, mip_best_value: u32) -> ConflictReport {
    let analyzer = InfeasibilityAnalyzer::new(
        &input.items,
        &input.capacities,
        &input.rules,
        input.target_value,
        mip_best_value,
    );
    analyzer.analyze()
}

#[wasm_bindgen]
pub fn solve_knapsack_branchless_dp(val: JsValue) -> Result<JsValue, JsValue> {
    let input: KnapsackInput = serde_wasm_bindgen::from_value(val)?;
    let res = solve_knapsack_branchless_dp_core(&input)
        .map_err(|e| JsValue::from_str(&e))?;
    Ok(serde_wasm_bindgen::to_value(&res)?)
}

#[wasm_bindgen]
pub fn compile_to_3sat(val: JsValue) -> Result<JsValue, JsValue> {
    let input: KnapsackInput = serde_wasm_bindgen::from_value(val)?;
    let res = compile_to_3sat_core(&input);
    Ok(serde_wasm_bindgen::to_value(&res)?)
}

#[wasm_bindgen]
pub fn solve_knapsack_branch_and_bound(val: JsValue) -> Result<JsValue, JsValue> {
    let input: KnapsackInput = serde_wasm_bindgen::from_value(val)?;
    let res = solve_knapsack_branch_and_bound_core(&input);
    Ok(serde_wasm_bindgen::to_value(&res)?)
}

#[wasm_bindgen]
pub fn analyze_infeasibility(val: JsValue, mip_best_value: u32) -> Result<JsValue, JsValue> {
    let input: KnapsackInput = serde_wasm_bindgen::from_value(val)?;
    let res = analyze_infeasibility_core(&input, mip_best_value);
    Ok(serde_wasm_bindgen::to_value(&res)?)
}