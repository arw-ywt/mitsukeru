// tests/schema_drift_test.rs

use mitsukeru::KnapsackInput;
use mitsukeru::schemars::schema_for;
use mitsukeru::serde_json;
use std::fs;

#[test]
fn test_schema_is_up_to_date() {
    let current_schema = schema_for!(KnapsackInput);
    let expected_json = serde_json::to_string_pretty(&current_schema).unwrap();

    let committed_json = fs::read_to_string("schemas/v1/mitsukeru-schema.json")
        .expect("Schema file missing. Run `cargo run --bin export_schema` to generate it.");

    assert_eq!(
        expected_json.trim(),
        committed_json.trim(),
        "\n=======================================================\n\
         ERROR: The committed JSON schema is out of date!\n\
         You modified `src/model.rs` without updating the schema.\n\
         Please run: `cargo run --bin export_schema`\n\
         ======================================================="
    );
}