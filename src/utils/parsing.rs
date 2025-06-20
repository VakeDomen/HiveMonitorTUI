use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::models::{AuthKeys, WorkerPings};



pub fn parse_worker_pings(raw: Value) -> WorkerPings {
    let mut out = HashMap::new();

    if let Value::Object(map) = raw {
        for (name, v) in map {
            // Expecting an array of timestamp strings
            let times = match v {
                Value::Array(arr) => arr.iter().filter_map(|val| {
                    val.as_str()
                       .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                       .map(|dt| dt.with_timezone(&Utc))
                }).collect(),
                _ => Vec::new(),
            };
            out.insert(name, times);
        }
    }
    out
}


/// Parses the raw `/key` JSON into a Vec of AuthKey, but if the raw
/// is a number (e.g. an integer count), returns an empty Vec.
pub fn parse_auth_keys(raw: Value) -> AuthKeys {
    match raw {
        Value::Array(arr) => {
            // Delegate to serde for normal array of objects
            serde_json::from_value(Value::Array(arr)).unwrap_or_default()
        }
        other => {
            // Unexpected type: try to coerce or just bail to empty
            // eprintln!("WARN unexpected /key payload: {:?}", other);
            Vec::new()
        }
    }
}