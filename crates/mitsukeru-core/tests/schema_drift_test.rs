// tests/schema_drift_test.rs
// cargo test --test schema_drift_test

use mitsukeru::{OptimizeError, OptimizeRequest, ProblemInput, Solution};
use schemars::schema_for;
use std::fs;

#[test]
fn generate_or_verify_schemas() -> Result<(), Box<dyn core::error::Error>> {
    let schema_input = schema_for!(ProblemInput);
    let schema_solution = schema_for!(Solution);
    let schema_request = schema_for!(OptimizeRequest);
    let schema_error = schema_for!(OptimizeError);

    fs::create_dir_all("schemas")?;

    fs::write(
        "schemas/problem-input.schema.json",
        serde_json::to_string_pretty(&schema_input)?,
    )?;

    fs::write(
        "schemas/solution.schema.json",
        serde_json::to_string_pretty(&schema_solution)?,
    )?;

    fs::write(
        "schemas/optimize-request.schema.json",
        serde_json::to_string_pretty(&schema_request)?,
    )?;

    fs::write(
        "schemas/optimize-error.schema.json",
        serde_json::to_string_pretty(&schema_error)?,
    )?;

    Ok(())
}
