// tests/schema_drift_test.rs
#[test]
fn test_schema_is_up_to_date() {
    let current_schema = schemars::schema_for!(mitsukeru::KnapsackInput);
    let expected_json = serde_json::to_string_pretty(&current_schema).unwrap();
    let committed_json = std::fs::read_to_string("schemas/v1/mitsukeru-schema.json")
        .expect("Schema file must exist. Run `cargo run --bin export_schema`");
    
    assert_eq!(
        expected_json.trim(), 
        committed_json.trim(),
        "Committed schema is outdated! Run `cargo run --bin export_schema` to update."
    );
}