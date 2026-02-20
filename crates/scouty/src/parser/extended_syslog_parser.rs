//! Hand-written parser for extended/SONiC syslog format.
//!
//! Format: `YYYY MMM DD HH:MM:SS.ffffff HOSTNAME LEVEL PROCESS_PART: MESSAGE`
//!
//! PROCESS_PART variants:
//! - `process` → process_name only
//! - `container#process` → container + process_name
//! - `process[pid]` → process_name + pid
//! - `container#process[pid]` → container + process_name + pid

#[cfg(test)]
#[path = "extended_syslog_parser_tests.rs"]
mod extended_syslog_parser_tests;

use crate::record::{LogLevel, LogRecord};
use crate::traits::LogParser;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use std::sync::Arc;

/// Parser for extended/enterprise syslog format (SONiC, etc.).
#[derive(Debug)]
pub struct ExtendedSyslogParser {
    name: String,
}

impl ExtendedSyslogParser {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Parse with shared Arc<str> for source and loader_id.
    #[inline]
    pub fn parse_shared(
        &self,
        raw: &str,
        source: &Arc<str>,
        loader_id: &Arc<str>,
        id: u64,
    ) -> Option<LogRecord> {
        self.parse_inner(raw, source, loader_id, id)
    }

    #[inline]
    fn parse_inner(
        &self,
        raw: &str,
        source: &Arc<str>,
        loader_id: &Arc<str>,
        id: u64,
    ) -> Option<LogRecord> {
        let b = raw.as_bytes();
        // Minimum: "YYYY MMM DD HH:MM:SS.ffffff H L P: M" ~36 chars
        if b.len() < 30 {
            return None;
        }

        // Parse year: YYYY
        let year = parse_u32(&b[0..4])? as i32;
        if b[4] != b' ' {
            return None;
        }

        // Parse month: MMM
        let month = parse_month(&b[5..8])?;
        if b[8] != b' ' {
            return None;
        }

        // Parse day: DD (may be " D" or "DD")
        let day = if b[9] == b' ' {
            (b[10] - b'0') as u32
        } else {
            parse_u32(&b[9..11])?
        };
        if b[11] != b' ' {
            return None;
        }

        // Parse time: HH:MM:SS.ffffff
        let hour = parse_u32(&b[12..14])?;
        if b[14] != b':' {
            return None;
        }
        let min = parse_u32(&b[15..17])?;
        if b[17] != b':' {
            return None;
        }
        let sec = parse_u32(&b[18..20])?;

        // Parse microseconds if present
        let (micros, time_end) = if b.len() > 20 && b[20] == b'.' {
            // Find end of fractional part
            let mut end = 21;
            while end < b.len() && b[end].is_ascii_digit() {
                end += 1;
            }
            let frac_str = unsafe { std::str::from_utf8_unchecked(&b[21..end]) };
            // Pad or truncate to 6 digits for microseconds
            let micros = if frac_str.len() >= 6 {
                parse_u32(&b[21..27])?
            } else {
                let mut val = parse_u32(&b[21..end])?;
                for _ in 0..(6 - frac_str.len()) {
                    val *= 10;
                }
                val
            };
            (micros, end)
        } else {
            (0, 20)
        };

        if time_end >= b.len() || b[time_end] != b' ' {
            return None;
        }

        let date = NaiveDate::from_ymd_opt(year, month, day)?;
        let time = NaiveTime::from_hms_micro_opt(hour, min, sec, micros)?;
        let timestamp: DateTime<Utc> =
            DateTime::from_naive_utc_and_offset(NaiveDateTime::new(date, time), Utc);

        // After timestamp: HOSTNAME LEVEL PROCESS_PART: MESSAGE
        let rest = &b[time_end + 1..];

        // Find hostname (ends at next space)
        let hostname_end = memchr_space(rest, 0)?;
        let hostname =
            unsafe { std::str::from_utf8_unchecked(&rest[..hostname_end]) }.to_string();

        let after_host = hostname_end + 1;
        if after_host >= rest.len() {
            return None;
        }

        // Find level (ends at next space)
        let level_end = memchr_space(rest, after_host)?;
        let level_str = unsafe { std::str::from_utf8_unchecked(&rest[after_host..level_end]) };
        let level = LogLevel::from_str_loose(level_str);

        let after_level = level_end + 1;
        if after_level >= rest.len() {
            return None;
        }

        // Find PROCESS_PART (ends at ": ")
        let colon_pos = find_colon_space(rest, after_level)?;
        let proc_section = &rest[after_level..colon_pos];
        let (container, process_name, pid) = parse_process_part(proc_section);

        // Message starts after ": "
        let msg_start = colon_pos + 2;
        let message = if msg_start < rest.len() {
            unsafe { std::str::from_utf8_unchecked(&rest[msg_start..]) }.to_string()
        } else {
            String::new()
        };

        Some(LogRecord {
            id,
            timestamp,
            level,
            source: Arc::clone(source),
            pid,
            tid: None,
            component_name: None,
            process_name: Some(process_name),
            hostname: Some(hostname),
            container,
            message,
            raw: raw.to_string(),
            metadata: None,
            loader_id: Arc::clone(loader_id),
        })
    }
}

impl LogParser for ExtendedSyslogParser {
    fn parse(&self, raw: &str, source: &str, loader_id: &str, id: u64) -> Option<LogRecord> {
        let source = Arc::from(source);
        let loader_id = Arc::from(loader_id);
        self.parse_inner(raw, &source, &loader_id, id)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// --- Helper functions ---

#[inline]
fn parse_u32(bytes: &[u8]) -> Option<u32> {
    let mut val = 0u32;
    for &b in bytes {
        if !b.is_ascii_digit() {
            return None;
        }
        val = val * 10 + (b - b'0') as u32;
    }
    Some(val)
}

fn parse_month(b: &[u8]) -> Option<u32> {
    if b.len() < 3 {
        return None;
    }
    match (b[0] | 0x20, b[1] | 0x20, b[2] | 0x20) {
        (b'j', b'a', b'n') => Some(1),
        (b'f', b'e', b'b') => Some(2),
        (b'm', b'a', b'r') => Some(3),
        (b'a', b'p', b'r') => Some(4),
        (b'm', b'a', b'y') => Some(5),
        (b'j', b'u', b'n') => Some(6),
        (b'j', b'u', b'l') => Some(7),
        (b'a', b'u', b'g') => Some(8),
        (b's', b'e', b'p') => Some(9),
        (b'o', b'c', b't') => Some(10),
        (b'n', b'o', b'v') => Some(11),
        (b'd', b'e', b'c') => Some(12),
        _ => None,
    }
}

#[inline]
fn memchr_space(b: &[u8], start: usize) -> Option<usize> {
    b[start..].iter().position(|&c| c == b' ').map(|p| p + start)
}

#[inline]
fn find_colon_space(b: &[u8], start: usize) -> Option<usize> {
    let slice = &b[start..];
    for i in 0..slice.len().saturating_sub(1) {
        if slice[i] == b':' && slice[i + 1] == b' ' {
            return Some(start + i);
        }
    }
    None
}

/// Parse PROCESS_PART into (container, process_name, pid).
///
/// Variants:
/// - `memory_checker` → (None, "memory_checker", None)
/// - `restapi#root` → (Some("restapi"), "root", None)
/// - `pmon#stormond[37]` → (Some("pmon"), "stormond", Some(37))
/// - `dockerd[871]` → (None, "dockerd", Some(871))
fn parse_process_part(b: &[u8]) -> (Option<String>, String, Option<u32>) {
    // Find # for container split
    let hash_pos = b.iter().position(|&c| c == b'#');
    let (container, proc_bytes) = match hash_pos {
        Some(pos) => {
            let container = unsafe { std::str::from_utf8_unchecked(&b[..pos]) }.to_string();
            (Some(container), &b[pos + 1..])
        }
        None => (None, b),
    };

    // Find [pid]
    let bracket_pos = proc_bytes.iter().position(|&c| c == b'[');
    let (process_name, pid) = match bracket_pos {
        Some(pos) => {
            let name =
                unsafe { std::str::from_utf8_unchecked(&proc_bytes[..pos]) }.to_string();
            // Extract pid between [ and ]
            let pid_start = pos + 1;
            let pid_end = proc_bytes.iter().position(|&c| c == b']').unwrap_or(proc_bytes.len());
            let pid = parse_u32(&proc_bytes[pid_start..pid_end]);
            (name, pid)
        }
        None => {
            let name = unsafe { std::str::from_utf8_unchecked(proc_bytes) }.to_string();
            (name, None)
        }
    };

    (container, process_name, pid)
}
