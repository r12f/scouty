//! JSON log parser — parses NDJSON/JSON Lines log entries.
//!
//! Auto-detects lines starting with `{`, maps well-known fields to LogRecord,
//! and populates `expanded` with the full JSON tree.

#[cfg(test)]
#[path = "json_parser_tests.rs"]
mod json_parser_tests;

use crate::record::{ExpandedField, ExpandedValue, LogLevel, LogRecord};
use crate::traits::LogParser;
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Parser for JSON log lines (one JSON object per line).
#[derive(Debug, Default)]
pub struct JsonParser;

impl JsonParser {
    pub fn new() -> Self {
        Self
    }
}

impl LogParser for JsonParser {
    fn parse(&self, raw: &str, source: &str, loader_id: &str, id: u64) -> Option<LogRecord> {
        let trimmed = raw.trim();
        if !trimmed.starts_with('{') {
            return None;
        }

        let obj: serde_json::Map<String, Value> = serde_json::from_str(trimmed).ok()?;

        let source_arc = Arc::from(source);
        let loader_arc = Arc::from(loader_id);

        let mut timestamp = Utc::now();
        let mut level: Option<LogLevel> = None;
        let mut message = String::new();
        let mut hostname: Option<String> = None;
        let mut component_name: Option<String> = None;
        let mut pid: Option<u32> = None;
        let mut tid: Option<u32> = None;
        let mut metadata: HashMap<String, String> = HashMap::new();
        let mut mapped_keys: Vec<String> = Vec::new();

        for (key, value) in &obj {
            let lower = key.to_ascii_lowercase();
            match lower.as_str() {
                "timestamp" | "time" | "ts" | "@timestamp" => {
                    if let Some(ts) = parse_timestamp(value) {
                        timestamp = ts;
                        mapped_keys.push(key.clone());
                    }
                }
                "level" | "severity" | "loglevel" => {
                    if let Some(s) = value.as_str() {
                        level = LogLevel::from_str_loose(s);
                        mapped_keys.push(key.clone());
                    }
                }
                "message" | "msg" | "log" => {
                    if let Some(s) = value.as_str() {
                        message = s.to_string();
                    } else {
                        message = value.to_string();
                    }
                    mapped_keys.push(key.clone());
                }
                "hostname" | "host" => {
                    hostname = Some(value_to_string(value));
                    mapped_keys.push(key.clone());
                }
                "service" | "component" | "logger" | "name" => {
                    component_name = Some(value_to_string(value));
                    mapped_keys.push(key.clone());
                }
                "pid" => {
                    pid = value
                        .as_u64()
                        .map(|v| v as u32)
                        .or_else(|| value.as_str().and_then(|s| s.parse().ok()));
                    mapped_keys.push(key.clone());
                }
                "tid" | "thread" => {
                    tid = value
                        .as_u64()
                        .map(|v| v as u32)
                        .or_else(|| value.as_str().and_then(|s| s.parse().ok()));
                    mapped_keys.push(key.clone());
                }
                _ => {
                    metadata.insert(key.clone(), value_to_string(value));
                }
            }
        }

        // Build expanded tree excluding already-mapped well-known fields
        let expanded_pairs: Vec<(String, ExpandedValue)> = obj
            .iter()
            .filter(|(k, _)| !mapped_keys.contains(k))
            .map(|(k, v)| (k.clone(), json_to_expanded(v, 0)))
            .collect();

        let expanded = if expanded_pairs.is_empty() {
            None
        } else {
            Some(vec![ExpandedField {
                label: "Payload".to_string(),
                value: ExpandedValue::KeyValue(expanded_pairs),
            }])
        };

        Some(LogRecord {
            id,
            timestamp,
            level,
            source: source_arc,
            pid,
            tid,
            component_name,
            process_name: None,
            hostname,
            container: None,
            context: None,
            function: None,
            message,
            raw: raw.to_string(),
            metadata: if metadata.is_empty() {
                None
            } else {
                Some(metadata)
            },
            loader_id: loader_arc,
            expanded,
        })
    }

    fn name(&self) -> &str {
        "json"
    }
}

/// Convert a JSON value to a string for metadata.
fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

/// Parse a timestamp from a JSON value (string or number).
fn parse_timestamp(v: &Value) -> Option<DateTime<Utc>> {
    match v {
        Value::String(s) => {
            // Try RFC 3339 / ISO 8601
            if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
                return Some(dt.with_timezone(&Utc));
            }
            // Try "YYYY-MM-DD HH:MM:SS" with optional fractional seconds
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f") {
                return Some(dt.and_utc());
            }
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
                return Some(dt.and_utc());
            }
            None
        }
        Value::Number(n) => {
            // Unix timestamp (seconds or milliseconds)
            let ts = n.as_f64()?;
            if ts > 1e12 {
                // Likely milliseconds
                DateTime::from_timestamp_millis(ts as i64)
            } else {
                DateTime::from_timestamp(ts as i64, ((ts.fract()) * 1_000_000_000.0) as u32)
            }
        }
        _ => None,
    }
}

/// Recursively convert a JSON value to ExpandedValue.
/// Depth limited to 10 levels.
fn json_to_expanded(v: &Value, depth: usize) -> ExpandedValue {
    if depth > 10 {
        return ExpandedValue::Text("...".to_string());
    }
    match v {
        Value::Object(map) => {
            let pairs = map
                .iter()
                .map(|(k, val)| (k.clone(), json_to_expanded(val, depth + 1)))
                .collect();
            ExpandedValue::KeyValue(pairs)
        }
        Value::Array(arr) => {
            let items = arr
                .iter()
                .map(|val| json_to_expanded(val, depth + 1))
                .collect();
            ExpandedValue::List(items)
        }
        Value::String(s) => ExpandedValue::Text(s.clone()),
        Value::Number(n) => ExpandedValue::Text(n.to_string()),
        Value::Bool(b) => ExpandedValue::Text(b.to_string()),
        Value::Null => ExpandedValue::Text("null".to_string()),
    }
}

/// Quick check: does this line look like a JSON log line?
pub fn looks_like_json(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('{') && trimmed.ends_with('}')
}
