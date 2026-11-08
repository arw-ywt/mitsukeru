// tests/schema_drift_test.rs
// cargo test --test schema_drift_test

use mitsukeru::{OptimizeError, OptimizeRequest, ProblemInput, Solution};
use std::fs;

#[test]
fn test_export_json_schema() {
    fs::create_dir_all("schemas").unwrap();

    let schema_input = schemars::schema_for!(ProblemInput);
    fs::write(
        "schemas/problem-input.schema.json",
        serde_json::to_string_pretty(&schema_input).unwrap(),
    ).unwrap();

    let schema_solution = schemars::schema_for!(Solution);
    fs::write(
        "schemas/solution.schema.json",
        serde_json::to_string_pretty(&schema_solution).unwrap(),
    ).unwrap();

    let schema_request = schemars::schema_for!(OptimizeRequest);
    fs::write(
        "schemas/optimize-request.schema.json",
        serde_json::to_string_pretty(&schema_request).unwrap(),
    ).unwrap();

    let schema_error = schemars::schema_for!(OptimizeError);
    fs::write(
        "schemas/optimize-error.schema.json",
        serde_json::to_string_pretty(&schema_error).unwrap(),
    ).unwrap();
}