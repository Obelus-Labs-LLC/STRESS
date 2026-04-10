use schemars::schema_for;
use stress_ref::types::report::{AggregateSummary, RunRecord};

fn main() {
    println!("=== RunRecord JSON Schema ===");
    let schema = schema_for!(RunRecord);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());

    println!("\n=== AggregateSummary JSON Schema ===");
    let schema = schema_for!(AggregateSummary);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
