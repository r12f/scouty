//! Hand-written zero-regex syslog parser for maximum performance.
//!
//! Parses Linux syslog format: `MMM DD HH:MM:SS hostname process[pid]: message`
//! Uses byte-level splitting — no regex, no UTF-8 re-validation overhead.

#[cfg(test)]
#[path = "syslog_parser_tests.rs"]
mod syslog_parser_tests;

use crate::record::LogRecord;
use crate::traits::LogParser;
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use std::sync::Arc;

/// A hand-written, zero-regex syslog parser optimized for maximum throughput.
///
/// Parses standard BSD/Linux syslog format:
/// ```text
/// Feb 19 14:23:45 myhost myapp[12345]: This is a log message
/// ```
///
/// Fields extracted: timestamp, process_name, pid, message.
/// No heap allocation for source/loader_id (uses Arc sharing).
/// No HashMap allocation (syslog has no extra metadata).
#[derive(Debug)]
pub struct SyslogParser {
    name: String,
    current_year: i32,
}

impl SyslogParser {
    /// Create a new SyslogParser.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            current_year: Utc::now().year(),
        }
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
        let b = raw.as_bytes();
        if b.len() < 16 {
            return None;
        }

        // Parse timestamp: "MMM DD HH:MM:SS" (15 bytes)
        let timestamp = self.parse_timestamp_inline(b)?;

        // Skip past timestamp + space to hostname
        // Position 15 should be a space
        if b.len() < 17 || b[15] != b' ' {
            return None;
        }

        // Find end of hostname (next space after position 16)
        let hostname_end = memchr_space(b, 16)?;
        let hostname = unsafe { std::str::from_utf8_unchecked(&b[16..hostname_end]) }.to_string();

        // After hostname: "process[pid]: message" or "process: message"
        let after_host = hostname_end + 1;
        if after_host >= b.len() {
            return None;
        }

        // Find the colon-space ": " separator
        let colon_pos = find_colon_space(b, after_host)?;

        // Extract process and pid from between hostname and colon
        let proc_section = &b[after_host..colon_pos];
        let (process_name, pid) = parse_process_pid(proc_section);

        // Message starts after ": "
        let msg_start = colon_pos + 2;
        let message = if msg_start < b.len() {
            // SAFETY: input is &str, so all slices are valid UTF-8
            unsafe { std::str::from_utf8_unchecked(&b[msg_start..]) }.to_string()
        } else {
            String::new()
        };

        Some(LogRecord {
            id,
            timestamp,
            level: None, // Standard syslog doesn't have level in the line
            source: Arc::clone(source),
            pid,
            tid: None,
            component_name: None,
            process_name: Some(process_name),
            hostname: Some(hostname),
            container: None,
            message,
            raw: raw.to_string(),
            metadata: None,
            loader_id: Arc::clone(loader_id),
        })
    }

    /// Parse taking ownership of raw string.
    #[inline]
    pub fn parse_shared_owned(
        &self,
        raw: String,
        source: &Arc<str>,
        loader_id: &Arc<str>,
        id: u64,
    ) -> Option<LogRecord> {
        let b = raw.as_bytes();
        if b.len() < 16 {
            return None;
        }

        let timestamp = self.parse_timestamp_inline(b)?;

        if b.len() < 17 || b[15] != b' ' {
            return None;
        }

        let hostname_end = memchr_space(b, 16)?;
        let hostname = unsafe { std::str::from_utf8_unchecked(&b[16..hostname_end]) }.to_string();
        let after_host = hostname_end + 1;
        if after_host >= b.len() {
            return None;
        }

        let colon_pos = find_colon_space(b, after_host)?;
        let proc_section = &b[after_host..colon_pos];
        let (process_name, pid) = parse_process_pid(proc_section);

        let msg_start = colon_pos + 2;
        let message = if msg_start < b.len() {
            unsafe { std::str::from_utf8_unchecked(&b[msg_start..]) }.to_string()
        } else {
            String::new()
        };

        Some(LogRecord {
            id,
            timestamp,
            level: None,
            source: Arc::clone(source),
            pid,
            tid: None,
            component_name: None,
            process_name: Some(process_name),
            hostname: Some(hostname),
            container: None,
            message,
            raw,
            metadata: None,
            loader_id: Arc::clone(loader_id),
        })
    }

    /// Batch parse for maximum throughput.
    pub fn parse_batch(
        &self,
        lines: &[&str],
        source: &Arc<str>,
        loader_id: &Arc<str>,
        start_id: u64,
    ) -> Vec<LogRecord> {
        let mut results = Vec::with_capacity(lines.len());
        let mut id = start_id;
        for line in lines {
            if let Some(record) = self.parse_shared(line, source, loader_id, id) {
                results.push(record);
            }
            id += 1;
        }
        results
    }

    /// Batch parse from owned strings.
    pub fn parse_batch_owned(
        &self,
        lines: Vec<String>,
        source: &Arc<str>,
        loader_id: &Arc<str>,
        start_id: u64,
    ) -> Vec<LogRecord> {
        let mut results = Vec::with_capacity(lines.len());
        let mut id = start_id;
        for line in lines {
            if let Some(record) = self.parse_shared_owned(line, source, loader_id, id) {
                results.push(record);
            }
            id += 1;
        }
        results
    }

    /// Inline timestamp parser — maximum speed, no function call overhead.
    #[inline(always)]
    fn parse_timestamp_inline(&self, b: &[u8]) -> Option<DateTime<Utc>> {
        // Month: bytes[0..3]
        let month: u32 = match (b[0], b[1], b[2]) {
            (b'J', b'a', b'n') => 1,
            (b'F', b'e', b'b') => 2,
            (b'M', b'a', b'r') => 3,
            (b'A', b'p', b'r') => 4,
            (b'M', b'a', b'y') => 5,
            (b'J', b'u', b'n') => 6,
            (b'J', b'u', b'l') => 7,
            (b'A', b'u', b'g') => 8,
            (b'S', b'e', b'p') => 9,
            (b'O', b'c', b't') => 10,
            (b'N', b'o', b'v') => 11,
            (b'D', b'e', b'c') => 12,
            _ => return None,
        };

        // Day: bytes[4..6], " 9" or "19"
        let day: u32 = if b[4] == b' ' {
            (b[5] - b'0') as u32
        } else {
            ((b[4] - b'0') * 10 + (b[5] - b'0')) as u32
        };

        // Time: HH:MM:SS at bytes[7..15]
        let hour = ((b[7] - b'0') * 10 + (b[8] - b'0')) as u32;
        let min = ((b[10] - b'0') * 10 + (b[11] - b'0')) as u32;
        let sec = ((b[13] - b'0') * 10 + (b[14] - b'0')) as u32;

        let date = NaiveDate::from_ymd_opt(self.current_year, month, day)?;
        let time = NaiveTime::from_hms_opt(hour, min, sec)?;
        Some(NaiveDateTime::new(date, time).and_utc())
    }
}

impl LogParser for SyslogParser {
    fn parse(&self, raw: &str, source: &str, loader_id: &str, id: u64) -> Option<LogRecord> {
        let source_arc: Arc<str> = Arc::from(source);
        let loader_arc: Arc<str> = Arc::from(loader_id);
        self.parse_shared(raw, &source_arc, &loader_arc, id)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Find next space byte starting from `start`.
#[inline(always)]
fn memchr_space(b: &[u8], start: usize) -> Option<usize> {
    // Simple scan — for short strings this beats memchr crate
    let mut i = start;
    while i < b.len() {
        if b[i] == b' ' {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Find ": " (colon + space) starting from `start`.
#[inline(always)]
fn find_colon_space(b: &[u8], start: usize) -> Option<usize> {
    let mut i = start;
    let end = b.len().saturating_sub(1);
    while i < end {
        if b[i] == b':' && b[i + 1] == b' ' {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Parse "process[pid]" or "process" from a byte slice.
/// Returns (process_name as String, Option<pid>).
#[inline]
fn parse_process_pid(section: &[u8]) -> (String, Option<u32>) {
    // Look for '[' to split process name and pid
    let mut bracket_pos = None;
    for (i, &byte) in section.iter().enumerate() {
        if byte == b'[' {
            bracket_pos = Some(i);
            break;
        }
    }

    match bracket_pos {
        Some(bp) => {
            let name = unsafe { std::str::from_utf8_unchecked(&section[..bp]) }.to_string();
            // Parse pid between '[' and ']'
            let pid_start = bp + 1;
            let mut pid: u32 = 0;
            let mut i = pid_start;
            while i < section.len() && section[i] != b']' {
                pid = pid * 10 + (section[i] - b'0') as u32;
                i += 1;
            }
            (name, Some(pid))
        }
        None => {
            let name = unsafe { std::str::from_utf8_unchecked(section) }.to_string();
            (name, None)
        }
    }
}
