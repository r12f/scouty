//! Hand-written byte-level parser for SONiC sairedis log format.
//!
//! Format: `YYYY-MM-DD.HH:MM:SS.ffffff|<op>|<detail...>`
//!
//! Supports 13 op codes with stateful G/Q context association.

#[cfg(test)]
#[path = "sairedis_parser_tests.rs"]
mod sairedis_parser_tests;

use crate::record::{ExpandedField, ExpandedValue, LogLevel, LogRecord};
use crate::traits::LogParser;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use std::cell::RefCell;
use std::sync::Arc;

/// Known op codes for sairedis logs.
const KNOWN_OPS: &[u8] = b"crsgGpCRSBqQnaA";

/// Parser for SONiC sairedis log format.
///
/// Maintains internal state for G/Q response context association.
/// Requires sequential line processing (not parallelizable).
#[derive(Debug)]
pub struct SairedisParser {
    /// Last context from a `g` (Get) operation.
    last_get_context: RefCell<Option<String>>,
    /// Last context from a `q` (Query) operation.
    last_query_context: RefCell<Option<String>>,
}

impl Default for SairedisParser {
    fn default() -> Self {
        Self::new()
    }
}

impl SairedisParser {
    pub fn new() -> Self {
        Self {
            last_get_context: RefCell::new(None),
            last_query_context: RefCell::new(None),
        }
    }

    pub fn parse_shared(
        &self,
        raw: &str,
        source: &Arc<str>,
        loader_id: &Arc<str>,
        id: u64,
    ) -> Option<LogRecord> {
        self.parse_inner(raw, source, loader_id, id)
    }

    fn parse_inner(
        &self,
        raw: &str,
        source: &Arc<str>,
        loader_id: &Arc<str>,
        id: u64,
    ) -> Option<LogRecord> {
        let b = raw.as_bytes();

        // Minimum: "YYYY-MM-DD.HH:MM:SS.f|x|" = 24 chars
        if b.len() < 24 {
            return None;
        }

        // Parse timestamp: YYYY-MM-DD.HH:MM:SS.ffffff
        if b[4] != b'-' || b[7] != b'-' || b[10] != b'.' || b[13] != b':' || b[16] != b':' {
            return None;
        }
        if b[19] != b'.' {
            return None;
        }

        let year = dig4(b, 0)? as i32;
        let month = dig2(b, 5)?;
        let day = dig2(b, 8)?;
        let hour = dig2(b, 11)?;
        let min = dig2(b, 14)?;
        let sec = dig2(b, 17)?;

        // Find pipe after fractional seconds
        let pipe1 = memchr::memchr(b'|', &b[20..])? + 20;
        let frac_bytes = &b[20..pipe1];
        let micros = parse_fractional_micros(frac_bytes);

        let date = NaiveDate::from_ymd_opt(year, month, day)?;
        let time = NaiveTime::from_hms_micro_opt(hour, min, sec, micros)?;
        let naive = NaiveDateTime::new(date, time);
        let timestamp: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);

        // After first pipe: op code
        let op_start = pipe1 + 1;
        if op_start >= b.len() {
            return None;
        }

        // Op code must be single char followed by '|' or end of line
        let op = b[op_start];
        let after_op = op_start + 1;
        if after_op < b.len() && b[after_op] != b'|' {
            return None; // Multi-char second segment = not sairedis
        }

        // Detail starts after "op|"
        let detail_start = if after_op < b.len() {
            after_op + 1 // skip the '|'
        } else {
            b.len()
        };
        let detail = if detail_start < b.len() {
            &b[detail_start..]
        } else {
            &[]
        };

        // Parse based on op code category
        let (function, component, context, message) = match op {
            // Single ops: c/s/g/p
            b'c' => {
                let (comp, ctx, msg) = self.parse_single_op(detail);
                ("Create".to_string(), comp, ctx, msg)
            }
            b's' => {
                let (comp, ctx, msg) = self.parse_single_op(detail);
                ("Set".to_string(), comp, ctx, msg)
            }
            b'g' => {
                let (comp, ctx, msg) = self.parse_single_op(detail);
                // Save context for G response
                *self.last_get_context.borrow_mut() = ctx.clone();
                ("Get".to_string(), comp, ctx, msg)
            }
            b'p' => {
                let (comp, ctx, msg) = self.parse_single_op(detail);
                ("CounterPoll".to_string(), comp, ctx, msg)
            }
            // Remove: r (context only, no attributes)
            b'r' => {
                let (comp, ctx) = Self::parse_type_context(detail);
                ("Remove".to_string(), comp, ctx, String::new())
            }
            // GetResponse: G (stateful)
            b'G' => {
                let ctx = self.last_get_context.borrow().clone();
                let msg = str_from_bytes(detail);
                ("GetResponse".to_string(), None, ctx, msg)
            }
            // Bulk ops: C/R/S/B
            b'C' => {
                let (comp, msg) = Self::parse_bulk_op(detail);
                ("BulkCreate".to_string(), comp, None, msg)
            }
            b'R' => {
                let (comp, msg) = Self::parse_bulk_op(detail);
                ("BulkRemove".to_string(), comp, None, msg)
            }
            b'S' => {
                let (comp, msg) = Self::parse_bulk_op(detail);
                ("BulkSet".to_string(), comp, None, msg)
            }
            b'B' => {
                let (comp, msg) = Self::parse_bulk_op(detail);
                ("BulkGet".to_string(), comp, None, msg)
            }
            // Query: q
            b'q' => {
                let (name, ctx, msg) = self.parse_query(detail);
                // Save context for Q response
                *self.last_query_context.borrow_mut() = ctx.clone();
                (format!("Query: {}", name), None, ctx, msg)
            }
            // QueryResponse: Q (stateful)
            b'Q' => {
                let (name, msg) = Self::parse_query_response(detail);
                let ctx = self.last_query_context.borrow().clone();
                (format!("QueryResponse: {}", name), None, ctx, msg)
            }
            // Notification: n
            b'n' => {
                let (name, msg) = Self::parse_notification(detail);
                (format!("Notification: {}", name), None, None, msg)
            }
            // NotifySyncd request: a (key = INIT_VIEW, APPLY_VIEW, etc.)
            b'a' => {
                let key = str_from_bytes(detail);
                ("NotifySyncd".to_string(), None, Some(key.clone()), key)
            }
            // NotifySyncd response: A (SAI status code)
            b'A' => {
                let status = str_from_bytes(detail);
                ("NotifySyncdResponse".to_string(), None, None, status)
            }
            // Unknown op code: graceful fallback
            _ => {
                let op_str = str_from_bytes(&[op]);
                let msg = str_from_bytes(detail);
                (op_str, None, None, msg)
            }
        };

        // LogLevel: notification → NOTICE, everything else → INFO
        let level = match op {
            b'n' => Some(LogLevel::Notice),
            _ => Some(LogLevel::Info),
        };

        // Build expanded field
        let expanded = build_expanded(op, &function, &component, &context, &message);

        Some(LogRecord {
            id,
            timestamp,
            level,
            source: Arc::clone(source),
            pid: None,
            tid: None,
            component_name: component,
            process_name: None,
            hostname: None,
            container: None,
            context,
            function: Some(function),
            message,
            raw: String::new(), // Caller sets raw
            metadata: None,
            loader_id: Arc::clone(loader_id),
            expanded,
        })
    }

    /// Parse single op detail: `SAI_OBJECT_TYPE:context|attr=val|...`
    /// Returns (component, context, message=attributes joined by |)
    fn parse_single_op(&self, detail: &[u8]) -> (Option<String>, Option<String>, String) {
        if detail.is_empty() {
            return (None, None, String::new());
        }

        // Find first '|' to split OBJECT_TYPE:context from attributes
        let first_pipe = memchr::memchr(b'|', detail);
        let type_ctx_bytes = match first_pipe {
            Some(pos) => &detail[..pos],
            None => detail,
        };

        let (comp, ctx) = Self::parse_type_context(type_ctx_bytes);

        let message = match first_pipe {
            Some(pos) if pos + 1 < detail.len() => str_from_bytes(&detail[pos + 1..]),
            _ => String::new(),
        };

        (comp, ctx, message)
    }

    /// Parse `SAI_OBJECT_TYPE_XXX:context` into (component, context).
    /// Context can be oid (`oid:0x...`) or JSON (`{"dest":...}`).
    fn parse_type_context(bytes: &[u8]) -> (Option<String>, Option<String>) {
        if bytes.is_empty() {
            return (None, None);
        }

        // Find the first ':' that separates type from context.
        // JSON contexts contain ':', so we look for the first ':' after "SAI_OBJECT_TYPE_*"
        let colon_pos = memchr::memchr(b':', bytes);
        match colon_pos {
            Some(pos) if pos > 0 => {
                let comp = str_from_bytes(&bytes[..pos]);
                let ctx = if pos + 1 < bytes.len() {
                    Some(str_from_bytes(&bytes[pos + 1..]))
                } else {
                    None
                };
                (Some(comp), ctx)
            }
            _ => {
                // No colon — entire thing is the component (e.g. bulk ops)
                (Some(str_from_bytes(bytes)), None)
            }
        }
    }

    /// Parse bulk op detail: `SAI_OBJECT_TYPE||entry1|attrs||entry2|attrs`
    /// Returns (component, message=entire detail after OBJECT_TYPE)
    fn parse_bulk_op(detail: &[u8]) -> (Option<String>, String) {
        if detail.is_empty() {
            return (None, String::new());
        }

        // Find '||' which separates OBJECT_TYPE from entries
        let double_pipe = find_double_pipe(detail);
        match double_pipe {
            Some(pos) => {
                let comp = str_from_bytes(&detail[..pos]);
                let msg = str_from_bytes(&detail[pos..]); // include || in message
                (Some(comp), msg)
            }
            None => {
                // No '||' — treat entire detail as message
                (None, str_from_bytes(detail))
            }
        }
    }

    /// Parse query detail: `query_name|context|attrs...`
    /// Returns (query_name, context, message=rest)
    fn parse_query(&self, detail: &[u8]) -> (String, Option<String>, String) {
        if detail.is_empty() {
            return (String::new(), None, String::new());
        }

        // First '|' separates query_name
        let pipe1 = memchr::memchr(b'|', detail);
        let name = match pipe1 {
            Some(pos) => str_from_bytes(&detail[..pos]),
            None => return (str_from_bytes(detail), None, String::new()),
        };

        let rest = &detail[pipe1.unwrap() + 1..];

        // Second segment is context (SAI_OBJECT_TYPE:oid)
        let pipe2 = memchr::memchr(b'|', rest);
        let (ctx, msg) = match pipe2 {
            Some(pos) => {
                let ctx_str = str_from_bytes(&rest[..pos]);
                // Parse context to get the actual context part (after ':')
                let (_, ctx) = Self::parse_type_context(rest[..pos].as_ref());
                let msg = if pos + 1 < rest.len() {
                    str_from_bytes(&rest[pos + 1..])
                } else {
                    String::new()
                };
                // Use full context string if parse_type_context found a context,
                // otherwise use the whole segment
                let final_ctx = ctx.or(Some(ctx_str));
                (final_ctx, msg)
            }
            None => {
                let (_, ctx) = Self::parse_type_context(rest);
                (ctx, String::new())
            }
        };

        (name, ctx, msg)
    }

    /// Parse query response detail: `query_name|status|attrs...`
    /// Returns (query_name, message=rest after query_name)
    fn parse_query_response(detail: &[u8]) -> (String, String) {
        if detail.is_empty() {
            return (String::new(), String::new());
        }

        let pipe1 = memchr::memchr(b'|', detail);
        match pipe1 {
            Some(pos) => {
                let name = str_from_bytes(&detail[..pos]);
                let msg = if pos + 1 < detail.len() {
                    str_from_bytes(&detail[pos + 1..])
                } else {
                    String::new()
                };
                (name, msg)
            }
            None => (str_from_bytes(detail), String::new()),
        }
    }

    /// Parse notification detail: `event_name|json_data|`
    /// Returns (event_name, message=json_data)
    fn parse_notification(detail: &[u8]) -> (String, String) {
        if detail.is_empty() {
            return (String::new(), String::new());
        }

        let pipe1 = memchr::memchr(b'|', detail);
        match pipe1 {
            Some(pos) => {
                let name = str_from_bytes(&detail[..pos]);
                // Message is everything between first pipe and end (may have trailing |)
                let mut msg_end = detail.len();
                if msg_end > pos + 1 && detail[msg_end - 1] == b'|' {
                    msg_end -= 1; // trim trailing pipe
                }
                let msg = if pos + 1 < msg_end {
                    str_from_bytes(&detail[pos + 1..msg_end])
                } else {
                    String::new()
                };
                (name, msg)
            }
            None => (str_from_bytes(detail), String::new()),
        }
    }
}

/// Build expanded field for sairedis entries.
///
/// Structure: Operation, Object Type, OID (if present), Status (for response ops),
/// Attributes (if present), and Request Context for stateful G/Q responses.
fn build_expanded(
    op: u8,
    function: &str,
    component: &Option<String>,
    context: &Option<String>,
    message: &str,
) -> Option<Vec<ExpandedField>> {
    let mut fields = Vec::new();
    let is_response = matches!(op, b'G' | b'A' | b'Q');

    // Operation (human-readable name)
    fields.push(ExpandedField {
        label: "Operation".to_string(),
        value: ExpandedValue::Text(function.to_string()),
    });

    // Object Type (component_name)
    if let Some(obj_type) = component {
        fields.push(ExpandedField {
            label: "Object Type".to_string(),
            value: ExpandedValue::Text(obj_type.clone()),
        });
    }

    // OID (context)
    if let Some(oid) = context {
        fields.push(ExpandedField {
            label: "OID".to_string(),
            value: ExpandedValue::Text(oid.clone()),
        });
    }

    // For response ops (G/A/Q), extract first attribute as Status
    let attrs_message = if is_response && !message.is_empty() {
        // Split message into segments; first segment is the status code
        let first_pipe = message.find('|');
        let (status_str, remaining) = match first_pipe {
            Some(pos) => (&message[..pos], &message[pos + 1..]),
            None => (message, ""),
        };

        // Extract status: for "key=value" format, the value is the status;
        // for plain text (e.g. "SAI_STATUS_SUCCESS"), the whole thing is status
        let status = if let Some(eq_pos) = status_str.find('=') {
            &status_str[eq_pos + 1..]
        } else {
            status_str
        };

        if !status.is_empty() {
            fields.push(ExpandedField {
                label: "Status".to_string(),
                value: ExpandedValue::Text(status.to_string()),
            });
        }

        remaining
    } else {
        message
    };

    // Attributes from message (pipe-delimited "key=value" pairs)
    if !attrs_message.is_empty() {
        let pairs: Vec<(String, ExpandedValue)> = attrs_message
            .split('|')
            .filter(|s| !s.is_empty())
            .map(|attr| {
                if let Some(pos) = attr.find('=') {
                    let k = &attr[..pos];
                    let v = &attr[pos + 1..];
                    (k.to_string(), ExpandedValue::Text(v.to_string()))
                } else {
                    (attr.to_string(), ExpandedValue::Text(String::new()))
                }
            })
            .collect();

        if !pairs.is_empty() {
            fields.push(ExpandedField {
                label: "Attributes".to_string(),
                value: ExpandedValue::KeyValue(pairs),
            });
        }
    }

    // For G/Q responses, label includes request context already in function name
    // Mark stateful ops
    if matches!(op, b'G' | b'Q') && context.is_some() {
        fields.push(ExpandedField {
            label: "Request Context".to_string(),
            value: ExpandedValue::Text(context.as_ref().unwrap().clone()),
        });
    }

    Some(fields)
}

impl LogParser for SairedisParser {
    fn parse(&self, raw: &str, source: &str, loader_id: &str, id: u64) -> Option<LogRecord> {
        let source = Arc::from(source);
        let loader_id = Arc::from(loader_id);
        self.parse_shared(raw, &source, &loader_id, id)
    }

    fn name(&self) -> &str {
        "sairedis"
    }
}

/// Check if a line looks like sairedis format.
/// `YYYY-MM-DD.HH:MM:SS.ffffff|<single-char-op>|...`
pub fn looks_like_sairedis(line: &str) -> bool {
    let b = line.as_bytes();
    if b.len() < 24 {
        return false;
    }
    // Check timestamp structure
    if b[4] != b'-' || b[7] != b'-' || b[10] != b'.' {
        return false;
    }
    // Find first pipe after position 19 (fractional seconds)
    let pipe1 = match memchr::memchr(b'|', &b[20..]) {
        Some(pos) => 20 + pos,
        None => return false,
    };
    // Op code is single char at pipe1+1, followed by '|'
    let op_pos = pipe1 + 1;
    if op_pos + 1 >= b.len() {
        return false;
    }
    let op = b[op_pos];
    let after_op = op_pos + 1;
    if b[after_op] != b'|' {
        return false;
    }
    // Check op is in known set
    KNOWN_OPS.contains(&op)
}

// ── Helper functions ────────────────────────────────────────────────────────

#[inline]
fn dig2(b: &[u8], pos: usize) -> Option<u32> {
    let d0 = b[pos].wrapping_sub(b'0');
    let d1 = b[pos + 1].wrapping_sub(b'0');
    if d0 > 9 || d1 > 9 {
        return None;
    }
    Some(d0 as u32 * 10 + d1 as u32)
}

#[inline]
fn dig4(b: &[u8], pos: usize) -> Option<u32> {
    let d0 = b[pos].wrapping_sub(b'0') as u32;
    let d1 = b[pos + 1].wrapping_sub(b'0') as u32;
    let d2 = b[pos + 2].wrapping_sub(b'0') as u32;
    let d3 = b[pos + 3].wrapping_sub(b'0') as u32;
    if d0 > 9 || d1 > 9 || d2 > 9 || d3 > 9 {
        return None;
    }
    Some(d0 * 1000 + d1 * 100 + d2 * 10 + d3)
}

#[inline]
fn parse_fractional_micros(bytes: &[u8]) -> u32 {
    let mut result: u32 = 0;
    let mut digits = 0;
    for &byte in bytes {
        let d = byte.wrapping_sub(b'0');
        if d > 9 {
            break;
        }
        if digits < 6 {
            result = result * 10 + d as u32;
            digits += 1;
        }
    }
    // Pad with zeros if fewer than 6 digits
    while digits < 6 {
        result *= 10;
        digits += 1;
    }
    result
}

#[inline]
fn str_from_bytes(bytes: &[u8]) -> String {
    // Safety: input originates from &str, so bytes are valid UTF-8
    unsafe { std::str::from_utf8_unchecked(bytes) }.to_string()
}

/// Find the position of `||` in byte slice.
fn find_double_pipe(bytes: &[u8]) -> Option<usize> {
    if bytes.len() < 2 {
        return None;
    }
    (0..bytes.len() - 1).find(|&i| bytes[i] == b'|' && bytes[i + 1] == b'|')
}
