// src/bin/export_schema.rs

use mitsukeru::KnapsackInput;
use schemars::schema_for;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let schema = schema_for!(KnapsackInput);
    let schema_json = serde_json::to_string_pretty(&schema)?;
    let output_dir = Path::new("schemas/v1");
    
    create_dir_all(output_dir)?;

    let output_path = output_dir.join("mitsukeru-schema.json");
    let mut file = File::create(&output_path)?;
    file.write_all(schema_json.as_bytes())?;

    println!("--------------------------------------------------");
    println!(" Successfully generated JSON Schema!");
    println!(" Path: {}", output_path.display());
    println!("--------------------------------------------------");

    Ok(())
}