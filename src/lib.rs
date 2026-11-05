mod model;
mod dp_branchless;
mod sat_compiler;
mod branch_and_bound;
mod analyzer;

use wasm_bindgen::prelude::*;
use crate::model::{KnapsackInput, DpResult, SatReductionResult, MipResult};
use crate::dp_branchless::Flat2dDpSolver;
use crate::sat_compiler::SatCompiler;
use crate::branch_and_bound::BranchAndBoundSolver;
use crate::analyzer::InfeasibilityAnalyzer;

#[wasm_bindgen]
pub fn solve_knapsack_branchless_dp(val: JsValue) -> Result<JsValue, JsValue> {
    let input: KnapsackInput = serde_wasm_bindgen::from_value(val)?;
    
    if input.capacities.len() < 2 {
        return Err(JsValue::from_str("At least 2 capacities required for 2D DP"));
    }

    let c1 = input.capacities[0] as usize;
    let c2 = input.capacities[1] as usize;
    let n = input.items.len();

    // 実際にヒープに確保する総バイト数を計算
    // テーブル: (c1+1)*(c2+1)*4 bytes, 履歴: n*(c1+1)*(c2+1)*1 byte
    let grid_size = (c1 + 1) * (c2 + 1);
    let total_bytes = (grid_size * 4) + (n * grid_size * 1);
    let memory_mb = total_bytes as f64 / (1024.0 * 1024.0);

    let mut solver = Flat2dDpSolver::new(input.capacities[0], input.capacities[1]);
    let (best_value, selected_item_ids) = solver.solve(&input.items);

    let res = DpResult {
        best_value,
        selected_item_ids,
        execution_time_ms: 0.0,
        memory_mb,
    };

    Ok(serde_wasm_bindgen::to_value(&res)?)
}

#[wasm_bindgen]
pub fn compile_to_3sat(val: JsValue) -> Result<JsValue, JsValue> {
    let input: KnapsackInput = serde_wasm_bindgen::from_value(val)?;
    let target_val = input.target_value.unwrap_or(0);

    let (dimacs, num_vars, num_clauses, var_map) = SatCompiler::compile(
        &input.items,
        &input.capacities,
        &input.rules,
        target_val,
    );

    let res = SatReductionResult {
        num_variables: num_vars,
        num_clauses,
        dimacs_cnf: dimacs,
        variable_map: var_map,
    };

    Ok(serde_wasm_bindgen::to_value(&res)?)
}

#[wasm_bindgen]
pub fn solve_knapsack_branch_and_bound(val: JsValue) -> Result<JsValue, JsValue> {
    let input: KnapsackInput = serde_wasm_bindgen::from_value(val)?;

    let mut solver = BranchAndBoundSolver::new(&input.items, &input.capacities, &input.rules);
    let (best_value, selected_item_ids, explored_nodes) = solver.solve();

    let res = MipResult {
        best_value,
        selected_item_ids,
        execution_time_ms: 0.0,
        explored_nodes,
    };

    Ok(serde_wasm_bindgen::to_value(&res)?)
}

#[wasm_bindgen]
pub fn analyze_infeasibility(val: JsValue, mip_best_value: u32) -> Result<JsValue, JsValue> {
    let input: KnapsackInput = serde_wasm_bindgen::from_value(val)?;
    let analyzer = InfeasibilityAnalyzer::new(
        &input.items,
        &input.capacities,
        &input.rules,
        input.target_value,
        mip_best_value,
    );
    let report = analyzer.analyze();
    Ok(serde_wasm_bindgen::to_value(&report)?)
}