//! Validation only: line-delimited JSON -> engine observations, no effects.
use serde_json::{json, Value};
use std::io::{self, BufRead};
use tezbar_custom_wordle_v2::{generate, generate_utf16, Failure};

fn main() {
    for line in io::stdin().lock().lines() {
        let input: Value =
            serde_json::from_str(&line.expect("input read failed")).expect("invalid case");
        let result = if let Some(units) = input.get("utf16").and_then(Value::as_array) {
            let units: Option<Vec<u16>> = units
                .iter()
                .map(|v| v.as_u64().and_then(|n| u16::try_from(n).ok()))
                .collect();
            units.map_or(Err(Failure::MalformedArgument), |v| generate_utf16(&v))
        } else {
            input
                .get("word")
                .and_then(Value::as_str)
                .map_or(Err(Failure::MalformedArgument), generate)
        };
        let output = match result {
            Ok(v) => json!({"url":v.url,"word":v.word}),
            Err(Failure::MalformedArgument) => json!({"error":"malformed"}),
            Err(e) => json!({"validation":e.message()}),
        };
        println!("{output}");
    }
}
