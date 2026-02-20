//! Filter expression evaluator — evaluates an Expr against a LogRecord.

use crate::filter::expr::{Expr, Op};
use crate::record::LogRecord;
use regex::Regex;

/// Evaluate an expression against a log record.
pub fn eval(expr: &Expr, record: &LogRecord) -> bool {
    match expr {
        Expr::Comparison { field, op, value } => {
            let field_value = get_field(record, field);
            match field_value {
                Some(fv) => compare(&fv, op, value),
                None => false,
            }
        }
        Expr::And(left, right) => eval(left, record) && eval(right, record),
        Expr::Or(left, right) => eval(left, record) || eval(right, record),
        Expr::Not(inner) => !eval(inner, record),
    }
}

/// Extract a field value from a LogRecord by name.
fn get_field(record: &LogRecord, field: &str) -> Option<String> {
    match field {
        "id" => Some(record.id.to_string()),
        "timestamp" => Some(record.timestamp.to_rfc3339()),
        "level" => record.level.map(|l| l.to_string()),
        "source" => Some(record.source.to_string()),
        "pid" => record.pid.map(|p| p.to_string()),
        "tid" => record.tid.map(|t| t.to_string()),
        "component_name" | "component" => record.component_name.clone(),
        "process_name" | "process" => record.process_name.clone(),
        "message" => Some(record.message.clone()),
        "raw" => Some(record.raw.clone()),
        "loader_id" => Some(record.loader_id.to_string()),
        _ => {
            // Check metadata with "metadata." prefix or direct key
            if let Some(key) = field.strip_prefix("metadata.") {
                record.metadata.as_ref().and_then(|m| m.get(key).cloned())
            } else {
                record.metadata.as_ref().and_then(|m| m.get(field).cloned())
            }
        }
    }
}

/// Compare a field value against a target value using the given operator.
fn compare(field_value: &str, op: &Op, target: &str) -> bool {
    match op {
        Op::Eq => field_value == target,
        Op::Ne => field_value != target,
        Op::Gt => numeric_cmp(field_value, target)
            .map_or(field_value > target, |o| o == std::cmp::Ordering::Greater),
        Op::Ge => numeric_cmp(field_value, target)
            .map_or(field_value >= target, |o| o != std::cmp::Ordering::Less),
        Op::Lt => numeric_cmp(field_value, target)
            .map_or(field_value < target, |o| o == std::cmp::Ordering::Less),
        Op::Le => numeric_cmp(field_value, target)
            .map_or(field_value <= target, |o| o != std::cmp::Ordering::Greater),
        Op::Contains => field_value.contains(target),
        Op::StartsWith => field_value.starts_with(target),
        Op::EndsWith => field_value.ends_with(target),
        Op::Regex => Regex::new(target).is_ok_and(|re| re.is_match(field_value)),
    }
}

/// Try numeric comparison; returns None if either side isn't a number.
fn numeric_cmp(a: &str, b: &str) -> Option<std::cmp::Ordering> {
    let na: f64 = a.parse().ok()?;
    let nb: f64 = b.parse().ok()?;
    na.partial_cmp(&nb)
}

#[cfg(test)]
#[path = "eval_tests.rs"]
mod eval_tests;
