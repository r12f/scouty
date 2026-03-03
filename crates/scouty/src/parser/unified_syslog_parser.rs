//! Unified zero-regex syslog parser — handles BSD, Extended, and ISO 8601 formats.
//!
//! ## Format Detection (by first bytes)
//!
//! | First bytes | Format | Example |
//! |---|---|---|
//! | `A-Z` (month name) | BSD | `Nov 24 17:56:03 hostname process[pid]: msg` |
//! | `0-9{4} ` (year+space) | Extended | `2025 Nov 24 17:56:03.073872 hostname LEVEL container#process[pid]: msg` |
//! | `0-9{4}-` + `T` at pos 10 | ISO 8601 | `2025-11-24T17:56:03.073872-08:00 hostname process[pid]: msg` |
//! | `0-9{4}-` + ` ` at pos 10 + `T` at pos 30 | Dual-timestamp | `2026-03-03 06:54:06 2026-03-01T00:00:39.241739-08:00 hostname process[pid]: msg` |
//!
//! All parsing is hand-written byte-level — zero regex dependency.

#[cfg(test)]
#[path = "unified_syslog_parser_tests.rs"]
mod unified_syslog_parser_tests;

use crate::record::{LogLevel, LogRecord};
use crate::traits::LogParser;
use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use std::sync::Arc;

/// Unified zero-regex syslog parser.
#[derive(Debug)]
pub struct UnifiedSyslogParser {
    name: String,
    /// Year used for BSD format (lacks year in timestamp).
    current_year: i32,
}

impl UnifiedSyslogParser {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            current_year: Utc::now().year(),
        }
    }

    pub fn new_with_year(name: impl Into<String>, year: i32) -> Self {
        Self {
            name: name.into(),
            current_year: year,
        }
    }

    /// Parse with shared Arc<str> references.
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

        let first = b[0];
        if first.is_ascii_uppercase() {
            // BSD: starts with month name
            self.parse_bsd(b, raw, source, loader_id, id)
        } else if first.is_ascii_digit() {
            // Year-prefixed: check 5th byte
            if b.len() < 11 {
                return None;
            }
            if b[4] == b' ' {
                // Extended: "YYYY Mon ..."
                self.parse_extended(b, raw, source, loader_id, id)
            } else if b[4] == b'-' && b[10] == b'T' {
                // ISO 8601: "YYYY-MM-DDT..."
                self.parse_iso(b, raw, source, loader_id, id)
            } else if b[4] == b'-' && b.len() > 30 && b[10] == b' ' && b[19] == b' ' {
                // Dual-timestamp: "YYYY-MM-DD HH:MM:SS YYYY-MM-DDT..."
                // Skip the prepended timestamp and parse the ISO portion
                let rest = &b[20..];
                if rest.len() >= 11 && rest[4] == b'-' && rest[10] == b'T' {
                    let rest_str = &raw[20..];
                    self.parse_iso(rest, rest_str, source, loader_id, id)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    // ── BSD syslog ──────────────────────────────────────────────────────
    // Format: "MMM DD HH:MM:SS hostname process[pid]: message"
    #[inline]
    fn parse_bsd(
        &self,
        b: &[u8],
        raw: &str,
        source: &Arc<str>,
        loader_id: &Arc<str>,
        id: u64,
    ) -> Option<LogRecord> {
        let month = parse_month_3(b)?;

        // Day: bytes[4..6], " D" or "DD"
        let day: u32 = if b[4] == b' ' {
            let d = b[5].wrapping_sub(b'0');
            if d > 9 {
                return None;
            }
            d as u32
        } else {
            dig2(b, 4)?
        };

        // Time: HH:MM:SS at bytes[7..15]
        if b[9] != b':' || b[12] != b':' {
            return None;
        }
        let hour = dig2(b, 7)?;
        let min = dig2(b, 10)?;
        let sec = dig2(b, 13)?;

        if b[15] != b' ' {
            return None;
        }

        let date = NaiveDate::from_ymd_opt(self.current_year, month, day)?;
        let time = NaiveTime::from_hms_opt(hour, min, sec)?;
        let timestamp = NaiveDateTime::new(date, time).and_utc();

        // hostname starts at 16
        let hostname_end = memchr_space(b, 16)?;
        let hostname = str_slice(b, 16, hostname_end);

        let after_host = hostname_end + 1;
        if after_host >= b.len() {
            return None;
        }

        let colon_pos = find_colon_space(b, after_host)?;
        let (container, process_name, pid) = parse_process_part(&b[after_host..colon_pos]);

        let msg_start = colon_pos + 2;
        let message = if msg_start < b.len() {
            str_slice(b, msg_start, b.len()).to_string()
        } else {
            String::new()
        };

        Some(build_record(
            id,
            timestamp,
            None,
            source,
            loader_id,
            Some(hostname.to_string()),
            container,
            Some(process_name),
            pid,
            message,
            raw,
        ))
    }

    // ── Extended syslog ─────────────────────────────────────────────────
    // Format: "YYYY MMM DD HH:MM:SS.ffffff hostname LEVEL container#process[pid]: msg"
    #[inline]
    fn parse_extended(
        &self,
        b: &[u8],
        raw: &str,
        source: &Arc<str>,
        loader_id: &Arc<str>,
        id: u64,
    ) -> Option<LogRecord> {
        if b.len() < 30 {
            return None;
        }

        let year = dig4(b, 0)? as i32;
        // b[4] == ' ' already verified
        let month = parse_month_3(&b[5..])?;
        if b[8] != b' ' {
            return None;
        }

        // Day at [9..11]: " D" or "DD"
        let day: u32 = if b[9] == b' ' {
            let d = b[10].wrapping_sub(b'0');
            if d > 9 {
                return None;
            }
            d as u32
        } else {
            dig2(b, 9)?
        };
        if b[11] != b' ' {
            return None;
        }

        // Time: HH:MM:SS at [12..20]
        if b[14] != b':' || b[17] != b':' {
            return None;
        }
        let hour = dig2(b, 12)?;
        let min = dig2(b, 15)?;
        let sec = dig2(b, 18)?;

        // Fractional seconds
        let (micros, time_end) = if b.len() > 20 && b[20] == b'.' {
            parse_fractional(b, 21)
        } else {
            (0, 20)
        };

        if time_end >= b.len() || b[time_end] != b' ' {
            return None;
        }

        let date = NaiveDate::from_ymd_opt(year, month, day)?;
        let time = NaiveTime::from_hms_micro_opt(hour, min, sec, micros)?;
        let timestamp = NaiveDateTime::new(date, time).and_utc();

        let rest = &b[time_end + 1..];

        // hostname
        let hostname_end = memchr_space(rest, 0)?;
        let hostname = str_slice(rest, 0, hostname_end);

        // level
        let after_host = hostname_end + 1;
        if after_host >= rest.len() {
            return None;
        }
        let level_end = memchr_space(rest, after_host)?;
        let level_str = str_slice(rest, after_host, level_end);
        let level = LogLevel::from_str_loose(level_str);

        // process part
        let after_level = level_end + 1;
        if after_level >= rest.len() {
            return None;
        }
        let colon_pos = find_colon_space(rest, after_level)?;
        let (container, process_name, pid) = parse_process_part(&rest[after_level..colon_pos]);

        let msg_start = colon_pos + 2;
        let message = if msg_start < rest.len() {
            str_slice(rest, msg_start, rest.len()).to_string()
        } else {
            String::new()
        };

        Some(build_record(
            id,
            timestamp,
            level,
            source,
            loader_id,
            Some(hostname.to_string()),
            container,
            Some(process_name),
            pid,
            message,
            raw,
        ))
    }

    // ── ISO 8601 syslog ─────────────────────────────────────────────────
    // Format: "YYYY-MM-DDTHH:MM:SS.ffffffTZ hostname process[pid]: msg"
    // TZ: Z, +HH:MM, -HH:MM
    #[inline]
    fn parse_iso(
        &self,
        b: &[u8],
        raw: &str,
        source: &Arc<str>,
        loader_id: &Arc<str>,
        id: u64,
    ) -> Option<LogRecord> {
        // Minimum: "YYYY-MM-DDTHH:MM:SS hostname p: m" = 34 chars
        if b.len() < 20 {
            return None;
        }

        let year = dig4(b, 0)? as i32;
        // b[4]='-' already verified
        if b[7] != b'-' {
            return None;
        }
        let month = dig2(b, 5)?;
        let day = dig2(b, 8)?;
        // b[10]='T' already verified
        if b[13] != b':' || b[16] != b':' {
            return None;
        }
        let hour = dig2(b, 11)?;
        let min = dig2(b, 14)?;
        let sec = dig2(b, 17)?;

        // After seconds: optional fractional, then timezone
        let mut pos = 19;

        // Fractional seconds
        let micros = if pos < b.len() && b[pos] == b'.' {
            pos += 1;
            let (m, end) = parse_fractional(b, pos);
            pos = end;
            m
        } else {
            0
        };

        // Timezone: Z, +HH:MM, -HH:MM
        let offset_secs = if pos < b.len() {
            match b[pos] {
                b'Z' => {
                    pos += 1;
                    0
                }
                b'+' | b'-' => {
                    let sign: i32 = if b[pos] == b'+' { 1 } else { -1 };
                    pos += 1;
                    if pos + 5 > b.len() {
                        return None;
                    }
                    let tz_h = dig2(b, pos)? as i32;
                    pos += 2;
                    if b[pos] == b':' {
                        pos += 1;
                    }
                    let tz_m = dig2(b, pos)? as i32;
                    pos += 2;
                    sign * (tz_h * 3600 + tz_m * 60)
                }
                _ => 0,
            }
        } else {
            0
        };

        // Expect space after timestamp
        if pos >= b.len() || b[pos] != b' ' {
            return None;
        }
        pos += 1;

        let date = NaiveDate::from_ymd_opt(year, month, day)?;
        let time = NaiveTime::from_hms_micro_opt(hour, min, sec, micros)?;
        let naive = NaiveDateTime::new(date, time);
        let offset = FixedOffset::east_opt(offset_secs)?;
        let timestamp: DateTime<Utc> = naive
            .and_local_timezone(offset)
            .single()?
            .with_timezone(&Utc);

        // hostname
        let hostname_end = memchr_space(b, pos)?;
        let hostname = str_slice(b, pos, hostname_end);

        // process[pid]: message
        let after_host = hostname_end + 1;
        if after_host >= b.len() {
            return None;
        }
        let colon_pos = find_colon_space(b, after_host)?;
        let (container, process_name, pid) = parse_process_part(&b[after_host..colon_pos]);

        let msg_start = colon_pos + 2;
        let message = if msg_start < b.len() {
            str_slice(b, msg_start, b.len()).to_string()
        } else {
            String::new()
        };

        Some(build_record(
            id,
            timestamp,
            None,
            source,
            loader_id,
            Some(hostname.to_string()),
            container,
            Some(process_name),
            pid,
            message,
            raw,
        ))
    }
}

impl LogParser for UnifiedSyslogParser {
    fn parse(&self, raw: &str, source: &str, loader_id: &str, id: u64) -> Option<LogRecord> {
        let source_arc: Arc<str> = Arc::from(source);
        let loader_arc: Arc<str> = Arc::from(loader_id);
        self.parse_shared(raw, &source_arc, &loader_arc, id)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// ── Shared helpers ──────────────────────────────────────────────────────

/// Build a LogRecord from parsed components.
#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn build_record(
    id: u64,
    timestamp: DateTime<Utc>,
    level: Option<LogLevel>,
    source: &Arc<str>,
    loader_id: &Arc<str>,
    hostname: Option<String>,
    container: Option<String>,
    process_name: Option<String>,
    pid: Option<u32>,
    message: String,
    _raw: &str,
) -> LogRecord {
    LogRecord {
        id,
        timestamp,
        level,
        source: Arc::clone(source),
        pid,
        tid: None,
        component_name: None,
        process_name,
        hostname,
        container,
        context: None,
        function: None,
        message,
        raw: String::new(), // Caller should set raw to avoid double allocation
        metadata: None,
        loader_id: Arc::clone(loader_id),
        expanded: None,
    }
}

/// Parse 3-letter month name starting at `b[0..3]`.
#[inline(always)]
fn parse_month_3(b: &[u8]) -> Option<u32> {
    if b.len() < 3 {
        return None;
    }
    match (b[0], b[1], b[2]) {
        (b'J', b'a', b'n') => Some(1),
        (b'F', b'e', b'b') => Some(2),
        (b'M', b'a', b'r') => Some(3),
        (b'A', b'p', b'r') => Some(4),
        (b'M', b'a', b'y') => Some(5),
        (b'J', b'u', b'n') => Some(6),
        (b'J', b'u', b'l') => Some(7),
        (b'A', b'u', b'g') => Some(8),
        (b'S', b'e', b'p') => Some(9),
        (b'O', b'c', b't') => Some(10),
        (b'N', b'o', b'v') => Some(11),
        (b'D', b'e', b'c') => Some(12),
        _ => None,
    }
}

/// Parse 2 ASCII digits at `b[offset..offset+2]`.
#[inline(always)]
fn dig2(b: &[u8], offset: usize) -> Option<u32> {
    let d0 = b[offset].wrapping_sub(b'0');
    let d1 = b[offset + 1].wrapping_sub(b'0');
    if d0 > 9 || d1 > 9 {
        return None;
    }
    Some(d0 as u32 * 10 + d1 as u32)
}

/// Parse 4 ASCII digits at `b[offset..offset+4]`.
#[inline(always)]
fn dig4(b: &[u8], offset: usize) -> Option<u32> {
    let d0 = b[offset].wrapping_sub(b'0');
    let d1 = b[offset + 1].wrapping_sub(b'0');
    let d2 = b[offset + 2].wrapping_sub(b'0');
    let d3 = b[offset + 3].wrapping_sub(b'0');
    if d0 > 9 || d1 > 9 || d2 > 9 || d3 > 9 {
        return None;
    }
    Some(d0 as u32 * 1000 + d1 as u32 * 100 + d2 as u32 * 10 + d3 as u32)
}

/// Parse fractional seconds starting at `b[start]` (after the '.').
/// Returns (microseconds, end position).
#[inline]
fn parse_fractional(b: &[u8], start: usize) -> (u32, usize) {
    let mut val: u32 = 0;
    let mut digits: u32 = 0;
    let mut pos = start;
    while pos < b.len() && digits < 6 && b[pos].is_ascii_digit() {
        val = val * 10 + (b[pos] - b'0') as u32;
        digits += 1;
        pos += 1;
    }
    // Skip remaining fractional digits
    while pos < b.len() && b[pos].is_ascii_digit() {
        pos += 1;
    }
    // Pad to 6 digits
    for _ in digits..6 {
        val *= 10;
    }
    (val, pos)
}

/// Find next space byte.
#[inline(always)]
fn memchr_space(b: &[u8], start: usize) -> Option<usize> {
    let mut i = start;
    while i < b.len() {
        if b[i] == b' ' {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Find `: ` (colon + space).
#[inline(always)]
fn find_colon_space(b: &[u8], start: usize) -> Option<usize> {
    let end = b.len().saturating_sub(1);
    let mut i = start;
    while i < end {
        if b[i] == b':' && b[i + 1] == b' ' {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Parse `container#process[pid]` or `process[pid]` or `process`.
#[inline(always)]
fn parse_process_part(section: &[u8]) -> (Option<String>, String, Option<u32>) {
    // Find '#' for container split
    let hash_pos = memchr_byte(section, b'#');
    let (container, proc_bytes) = match hash_pos {
        Some(pos) => {
            let c = str_slice(section, 0, pos).to_string();
            (Some(c), &section[pos + 1..])
        }
        None => (None, section),
    };

    // Find '[' for pid
    let bracket_pos = memchr_byte(proc_bytes, b'[');
    let (process_name, pid) = match bracket_pos {
        Some(pos) => {
            let name = str_slice(proc_bytes, 0, pos).to_string();
            let pid_end = memchr_byte(proc_bytes, b']').unwrap_or(proc_bytes.len());
            let mut pid: u32 = 0;
            for &byte in &proc_bytes[pos + 1..pid_end] {
                let d = byte.wrapping_sub(b'0');
                if d > 9 {
                    return (container, name, None);
                }
                pid = pid * 10 + d as u32;
            }
            (name, Some(pid))
        }
        None => {
            let name = unsafe { std::str::from_utf8_unchecked(proc_bytes) }.to_string();
            (name, None)
        }
    };

    (container, process_name, pid)
}

/// Find a byte in a slice.
#[inline(always)]
fn memchr_byte(b: &[u8], needle: u8) -> Option<usize> {
    b.iter().position(|&c| c == needle)
}

/// Slice bytes as &str (unsafe — caller guarantees valid UTF-8 input).
#[inline(always)]
fn str_slice(b: &[u8], start: usize, end: usize) -> &str {
    unsafe { std::str::from_utf8_unchecked(&b[start..end]) }
}
