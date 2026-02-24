//! Hand-written parser for SONiC SWSS log format.
//!
//! Format: `YYYY-MM-DD.HH:MM:SS.ffffff|<content>`
//!
//! Content patterns:
//! - Pure message: `recording started`
//! - TABLE:Key|OP|kv: `SWITCH_TABLE:switch|SET|k:v|k:v`
//! - TABLE|SubKey|OP|kv: `FLEX_COUNTER_TABLE|PG_DROP|SET|k:v`
//! - TABLE:Key|OP (no kv): `ROUTE_TABLE:fd00::/80|DEL`

#[cfg(test)]
#[path = "swss_parser_tests.rs"]
mod swss_parser_tests;

use crate::record::LogRecord;
use crate::traits::LogParser;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use std::sync::Arc;

/// Parser for SONiC SWSS log format.
#[derive(Debug, Default)]
pub struct SwssParser;

impl SwssParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_shared(
        raw: &str,
        source: &Arc<str>,
        loader_id: &Arc<str>,
        id: u64,
    ) -> Option<LogRecord> {
        Self::parse_inner(raw, source, loader_id, id)
    }

    fn parse_inner(
        raw: &str,
        source: &Arc<str>,
        loader_id: &Arc<str>,
        id: u64,
    ) -> Option<LogRecord> {
        let b = raw.as_bytes();

        // Minimum: "YYYY-MM-DD.HH:MM:SS.f|x" = 22 chars + content
        if b.len() < 22 {
            return None;
        }

        // Parse timestamp: YYYY-MM-DD.HH:MM:SS.ffffff
        // Positions: 0123456789012345678901234567
        //            YYYY-MM-DD.HH:MM:SS.ffffff
        if b[4] != b'-' || b[7] != b'-' || b[10] != b'.' || b[13] != b':' || b[16] != b':' {
            return None;
        }

        let year = parse_u32(&b[0..4])? as i32;
        let month = parse_u32(&b[5..7])?;
        let day = parse_u32(&b[8..10])?;
        let hour = parse_u32(&b[11..13])?;
        let min = parse_u32(&b[14..16])?;
        let sec = parse_u32(&b[17..19])?;

        // Parse fractional seconds (after the '.' at position 19)
        if b[19] != b'.' {
            return None;
        }

        // Find the pipe separator after timestamp
        let pipe_pos = memchr::memchr(b'|', &b[20..])?;
        let frac_end = 20 + pipe_pos;

        // Parse microseconds from fractional part
        let frac_str = std::str::from_utf8(&b[20..frac_end]).ok()?;
        let micros = parse_fractional_micros(frac_str);

        let date = NaiveDate::from_ymd_opt(year, month, day)?;
        let time = NaiveTime::from_hms_micro_opt(hour, min, sec, micros)?;
        let naive = NaiveDateTime::new(date, time);
        let timestamp: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);

        let content_start = frac_end + 1; // skip the '|'
        if content_start >= b.len() {
            // Empty content after timestamp
            return Some(LogRecord {
                id,
                timestamp,
                level: None,
                source: Arc::clone(source),
                pid: None,
                tid: None,
                component_name: None,
                process_name: None,
                hostname: None,
                container: None,
                context: None,
                function: None,
                message: String::new(),
                raw: raw.to_string(),
                metadata: None,
                loader_id: Arc::clone(loader_id),
                expanded: None,
            });
        }

        let content = &raw[content_start..];

        // Try to parse structured content: TABLE[:Key]|OP|kv...
        // First, check if there's a pipe in the content
        let (component, context, function, message) = parse_content(content);

        Some(LogRecord {
            id,
            timestamp,
            level: None,
            source: Arc::clone(source),
            pid: None,
            tid: None,
            component_name: component,
            process_name: None,
            hostname: None,
            container: None,
            context,
            function,
            message,
            raw: raw.to_string(),
            metadata: None,
            loader_id: Arc::clone(loader_id),
            expanded: None,
        })
    }
}

impl LogParser for SwssParser {
    fn parse(&self, raw: &str, source: &str, loader_id: &str, id: u64) -> Option<LogRecord> {
        let source_arc = Arc::from(source);
        let loader_arc = Arc::from(loader_id);
        Self::parse_inner(raw, &source_arc, &loader_arc, id)
    }

    fn name(&self) -> &str {
        "swss"
    }
}

/// Parse content after the timestamp|.
///
/// Returns (component, context, function, message).
///
/// Patterns:
/// 1. No pipe → pure message: ("recording started")
/// 2. TABLE:Key|OP|kv... → component=TABLE, context=Key, function=OP
/// 3. TABLE|SubKey|OP|kv → component=TABLE, context=SubKey, function=OP
/// 4. TABLE:Key|OP (no kv) → function=OP, empty message
fn parse_content(content: &str) -> (Option<String>, Option<String>, Option<String>, String) {
    // Find the first pipe in content
    let first_pipe = match content.find('|') {
        Some(pos) => pos,
        None => {
            // Pure message line — no structured content
            return (None, None, None, content.to_string());
        }
    };

    let first_segment = &content[..first_pipe];
    let rest = &content[first_pipe + 1..];

    // Check if first segment contains ':' → TABLE:Key format
    // Split only at first ':'
    let (table, key) = if let Some(colon_pos) = first_segment.find(':') {
        (
            &first_segment[..colon_pos],
            Some(&first_segment[colon_pos + 1..]),
        )
    } else {
        (first_segment, None)
    };

    // Now parse rest: could be OP|kv..., or SubKey|OP|kv...
    // If we have a key from ':', rest starts with OP
    // If no key, rest starts with SubKey|OP or just OP
    if key.is_some() {
        // TABLE:Key|<rest> — rest is OP[|kv...]
        let (op, kv) = split_op_and_kv(rest);
        (
            Some(table.to_string()),
            key.map(|k| k.to_string()),
            Some(op.to_string()),
            kv.unwrap_or_default(),
        )
    } else {
        // TABLE|<rest> — rest is SubKey|OP|kv... or OP|kv...
        // Check if first part of rest is SET/DEL (known ops)
        let (first_rest, after_first) = match rest.find('|') {
            Some(pos) => (&rest[..pos], Some(&rest[pos + 1..])),
            None => {
                // TABLE|something — 'something' could be OP or SubKey
                // If it's a known op, it's TABLE|OP
                if is_known_op(rest) {
                    return (
                        Some(table.to_string()),
                        None,
                        Some(rest.to_string()),
                        String::new(),
                    );
                }
                // Otherwise it's a pure message that happens to have a pipe
                return (None, None, None, content.to_string());
            }
        };

        if is_known_op(first_rest) {
            // TABLE|OP|kv...
            (
                Some(table.to_string()),
                None,
                Some(first_rest.to_string()),
                after_first.unwrap_or("").to_string(),
            )
        } else {
            // TABLE|SubKey|OP[|kv...]
            let (op, kv) = match after_first {
                Some(rest2) => split_op_and_kv(rest2),
                None => {
                    // TABLE|SubKey with no more content — treat as message
                    return (None, None, None, content.to_string());
                }
            };
            (
                Some(table.to_string()),
                Some(first_rest.to_string()),
                Some(op.to_string()),
                kv.unwrap_or_default(),
            )
        }
    }
}

/// Split "OP|kv1|kv2..." into (OP, Some("kv1|kv2...")) or (OP, None).
fn split_op_and_kv(s: &str) -> (&str, Option<String>) {
    match s.find('|') {
        Some(pos) => (&s[..pos], Some(s[pos + 1..].to_string())),
        None => (s, None),
    }
}

/// Check if a string is a known SWSS operation.
fn is_known_op(s: &str) -> bool {
    matches!(
        s,
        "SET" | "DEL" | "HSET" | "HDEL" | "GETRESPONSE" | "PLANNINGRESPONSE"
    )
}

/// Parse ASCII digits into u32.
fn parse_u32(bytes: &[u8]) -> Option<u32> {
    let mut result: u32 = 0;
    for &b in bytes {
        if !b.is_ascii_digit() {
            return None;
        }
        result = result * 10 + (b - b'0') as u32;
    }
    Some(result)
}

/// Parse fractional seconds string to microseconds.
fn parse_fractional_micros(s: &str) -> u32 {
    let mut micros: u32 = 0;
    let mut digits = 0;
    for b in s.bytes() {
        if digits >= 6 || !b.is_ascii_digit() {
            break;
        }
        micros = micros * 10 + (b - b'0') as u32;
        digits += 1;
    }
    if digits == 0 {
        return 0;
    }
    // Pad to 6 digits
    for _ in digits..6 {
        micros *= 10;
    }
    micros
}
