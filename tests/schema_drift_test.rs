// tests/schema_drift_test.rs

use mitsukeru::ProblemInput;
use mitsukeru::schemars::schema_for;
use std::fs;

#[test]
fn generate_and_verify_schema() {
    let current_schema = schema_for!(ProblemInput);
    let expected_json = serde_json::to_string_pretty(&current_schema).unwrap();
    fs::create_dir_all("schemas/v1").unwrap();
    let schema_path = "schemas/v1/mitsukeru-schema.json";
    let _ = fs::write(schema_path, &expected_json);
    let committed_json = fs::read_to_string(schema_path)
        .expect("Schema file missing.");

    assert_eq!(
        expected_json.trim(),
        committed_json.trim(),
        "Schema mismatch!"
    );
}