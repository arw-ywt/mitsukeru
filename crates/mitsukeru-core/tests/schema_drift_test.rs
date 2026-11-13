// tests/schema_drift_test.rs
#![cfg(feature = "schemars")]
use mitsukeru::{OptimizeError, OptimizeRequest, ProblemInput, Solution};
use schemars::schema_for;
use std::fs;
use std::path::Path;

#[test]
fn generate_or_verify_schemas() -> Result<(), Box<dyn core::error::Error>> {
    let schema_input = schema_for!(ProblemInput);
    let schema_solution = schema_for!(Solution);
    let schema_request = schema_for!(OptimizeRequest);
    let schema_error = schema_for!(OptimizeError);
    let root_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../schemas");
    fs::create_dir_all(&root_dir)?;

    fs::write(
        root_dir.join("problem-input.schema.json"),
        serde_json::to_string_pretty(&schema_input)?,
    )?;

    fs::write(
        root_dir.join("solution.schema.json"),
        serde_json::to_string_pretty(&schema_solution)?,
    )?;

    fs::write(
        root_dir.join("optimize-request.schema.json"),
        serde_json::to_string_pretty(&schema_request)?,
    )?;

    fs::write(
        root_dir.join("optimize-error.schema.json"),
        serde_json::to_string_pretty(&schema_error)?,
    )?;

    Ok(())
}
