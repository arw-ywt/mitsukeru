use mitsukeru::{
    GraphHeuristics, OptimizeRequest, Optimizer, OptimizerConfig, ProblemInput,
    analyze_infeasibility_core, compile_to_3sat_core, solve_branch_and_bound_core,
    solve_branchless_dp_core,
};
use wasm_bindgen::prelude::*;

/// Performs optimization based on the input data and settings.
///
/// # Errors
///
/// Returns a [`JsValue`] error if serialization or deserialization of the JS object fails, or if the optimization process fails.
#[wasm_bindgen]
pub fn optimize(input_value: JsValue, config_value: Option<JsValue>) -> Result<JsValue, JsValue> {
    let (problem, config) = if let Some(cfg_js) =
        config_value.filter(|v| !v.is_undefined() && !v.is_null())
    {
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

/// Invokes the 2D dynamic programming core solver.
///
/// # Errors
///
/// Returns a [`JsValue`] if an error occurs, such as a failure to deserialize a JS object or an insufficient capacity constraint.
#[wasm_bindgen]
pub fn solve_branchless_dp(input_value: JsValue) -> Result<JsValue, JsValue> {
    let input: ProblemInput = serde_wasm_bindgen::from_value(input_value)?;
    let res = solve_branchless_dp_core(&input).map_err(|e| JsValue::from_str(&e))?;
    Ok(serde_wasm_bindgen::to_value(&res)?)
}

/// Reduces the input problem to 3-SAT (in DIMACS CNF format).
///
/// # Errors
///
/// Returns a [`JsValue`] if the serialization or deserialization of a JS object fails.
#[wasm_bindgen]
pub fn compile_to_3sat(input_value: JsValue) -> Result<JsValue, JsValue> {
    let input: ProblemInput = serde_wasm_bindgen::from_value(input_value)?;
    let res = compile_to_3sat_core(&input);
    Ok(serde_wasm_bindgen::to_value(&res)?)
}

/// Performs a search using the Branch and Bound method.
///
/// # Errors
///
/// Returns a [`JsValue`] if the serialization or deserialization of a JS object fails.
#[wasm_bindgen]
pub fn solve_branch_and_bound(input_value: JsValue) -> Result<JsValue, JsValue> {
    let input: ProblemInput = serde_wasm_bindgen::from_value(input_value)?;
    let res = solve_branch_and_bound_core(&input);
    Ok(serde_wasm_bindgen::to_value(&res)?)
}

/// Performs static analysis to identify constraint conflicts or causes of infeasibility.
///
/// # Errors
///
/// Returns a [`JsValue`] if the serialization or deserialization of a JS object fails.
#[wasm_bindgen]
pub fn analyze_infeasibility(
    input_value: JsValue,
    bnb_best_value: u32,
) -> Result<JsValue, JsValue> {
    let input: ProblemInput = serde_wasm_bindgen::from_value(input_value)?;
    let res = analyze_infeasibility_core(&input, bnb_best_value);
    Ok(serde_wasm_bindgen::to_value(&res)?)
}

/// Performs initial solution prediction using graph heuristics.
///
/// # Errors
///
/// Returns a [`JsValue`] if serialization or deserialization of a JS object fails.
#[wasm_bindgen]
pub fn predict_heuristics(input_value: JsValue) -> Result<JsValue, JsValue> {
    let input: ProblemInput = serde_wasm_bindgen::from_value(input_value)?;
    let prediction = GraphHeuristics::predict(&input.items, &input.capacities, &input.rules);
    Ok(serde_wasm_bindgen::to_value(
        &prediction.initial_selected_ids,
    )?)
}
