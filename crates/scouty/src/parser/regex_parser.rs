//! Regex-based log parser.
//!
//! Uses named capture groups to extract fields from log lines.
//! Supported named groups: `timestamp`, `level`, `message`, `pid`, `tid`,
//! `component`, `process`, and any others go into `metadata`.

#[cfg(test)]
#[path = "regex_parser_tests.rs"]
mod regex_parser_tests;

use crate::record::{LogLevel, LogRecord};
use crate::traits::LogParser;
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use regex::bytes::Regex as BytesRegex;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;

/// A log parser driven by a single regex with named capture groups.
///
/// Supports two execution modes:
/// - **Text mode** (default): uses `regex::Regex` on `&str`
/// - **Bytes mode** (`use_bytes_regex`): uses `regex::bytes::Regex`, skipping UTF-8 re-validation
///   for known-ASCII/UTF-8 inputs. Enabled automatically for ASCII-safe patterns.
#[derive(Debug)]
pub struct RegexParser {
    name: String,
    pattern: Regex,
    /// Bytes-mode regex for ASCII-safe patterns (avoids UTF-8 validation overhead).
    bytes_pattern: Option<BytesRegex>,
    /// Optional timestamp format string (chrono strftime) for parsing the `timestamp` group.
    /// If None, tries ISO 8601 by default.
    timestamp_format: Option<String>,
    /// Whether to use fast syslog timestamp parsing.
    fast_syslog_ts: bool,
    /// Cached current year for syslog timestamp parsing.
    current_year: i32,
    /// Pre-computed list of extra named groups (not in KNOWN_FIELDS).
    extra_groups: Vec<String>,
    /// Whether any extra named groups exist.
    has_extra_groups: bool,
}

impl RegexParser {
    /// Create a new RegexParser.
    ///
    /// `pattern` must be a valid regex with named capture groups.
    /// Common groups: `(?P<timestamp>...)`, `(?P<level>...)`, `(?P<message>...)`.
    pub fn new(
        name: impl Into<String>,
        pattern: &str,
        timestamp_format: Option<String>,
    ) -> Result<Self, regex::Error> {
        let regex = Regex::new(pattern)?;

        // Try to compile bytes regex (always possible if text regex compiles)
        let bytes_pattern = BytesRegex::new(pattern).ok();

        // Pre-compute extra named groups
        let extra_groups: Vec<String> = regex
            .capture_names()
            .flatten()
            .filter(|n| !KNOWN_FIELDS.contains(n))
            .map(|n| n.to_string())
            .collect();
        let has_extra_groups = !extra_groups.is_empty();

        // Detect if syslog fast path applies
        let fast_syslog_ts = timestamp_format
            .as_deref()
            .is_some_and(|fmt| fmt == "%b %e %H:%M:%S" || fmt == "%b %d %H:%M:%S");

        Ok(Self {
            name: name.into(),
            pattern: regex,
            bytes_pattern,
            timestamp_format,
            fast_syslog_ts,
            current_year: Utc::now().year(),
            extra_groups,
            has_extra_groups,
        })
    }

    #[inline]
    fn parse_timestamp(&self, s: &str) -> Option<DateTime<Utc>> {
        if self.fast_syslog_ts {
            return parse_syslog_timestamp_fast(s, self.current_year);
        }

        if let Some(fmt) = &self.timestamp_format {
            NaiveDateTime::parse_from_str(s, fmt)
                .ok()
                .map(|dt| dt.and_utc())
        } else {
            // Try ISO 8601 first
            s.parse::<DateTime<Utc>>()
                .ok()
                .or_else(|| {
                    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                        .ok()
                        .map(|dt| dt.and_utc())
                })
                .or_else(|| {
                    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
                        .ok()
                        .map(|dt| dt.and_utc())
                })
        }
    }

    /// Parse with shared Arc<str> for source and loader_id (batch-friendly).
    pub fn parse_shared(
        &self,
        raw: &str,
        source: &Arc<str>,
        loader_id: &Arc<str>,
        id: u64,
    ) -> Option<LogRecord> {
        // Use bytes regex if available (avoids redundant UTF-8 checks)
        if let Some(ref bytes_re) = self.bytes_pattern {
            return self.parse_bytes_inner(raw, bytes_re, source, loader_id, id);
        }

        let caps = self.pattern.captures(raw)?;
        self.build_record_from_str_caps(&caps, raw, source, loader_id, id)
    }

    /// Parse taking ownership of the raw String (avoids one allocation).
    pub fn parse_shared_owned(
        &self,
        raw: String,
        source: &Arc<str>,
        loader_id: &Arc<str>,
        id: u64,
    ) -> Option<LogRecord> {
        if let Some(ref bytes_re) = self.bytes_pattern {
            return self.parse_bytes_inner_owned(raw, bytes_re, source, loader_id, id);
        }

        let caps = self.pattern.captures(&raw)?;
        let mut record = self.build_record_from_str_caps(&caps, &raw, source, loader_id, id)?;
        drop(caps);
        record.raw = raw;
        Some(record)
    }

    /// Batch parse: parse multiple lines at once, sharing source/loader_id Arc.
    ///
    /// Pre-allocates output Vec. Skips lines that don't match.
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

    /// Batch parse from owned Strings (avoids raw clone).
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

    /// Internal: parse using bytes regex for zero-copy field extraction.
    #[inline]
    fn parse_bytes_inner(
        &self,
        raw: &str,
        bytes_re: &BytesRegex,
        source: &Arc<str>,
        loader_id: &Arc<str>,
        id: u64,
    ) -> Option<LogRecord> {
        let caps = bytes_re.captures(raw.as_bytes())?;

        let timestamp = caps
            .name("timestamp")
            .and_then(|m| {
                // SAFETY: input is &str so matched bytes are valid UTF-8
                let s = unsafe { std::str::from_utf8_unchecked(m.as_bytes()) };
                self.parse_timestamp(s)
            })
            .unwrap_or_else(Utc::now);

        let level = caps.name("level").and_then(|m| {
            let s = unsafe { std::str::from_utf8_unchecked(m.as_bytes()) };
            LogLevel::from_str_loose(s)
        });

        let message = caps
            .name("message")
            .map(|m| unsafe { String::from_utf8_unchecked(m.as_bytes().to_vec()) })
            .unwrap_or_default();

        let pid = caps.name("pid").and_then(|m| {
            let s = unsafe { std::str::from_utf8_unchecked(m.as_bytes()) };
            s.parse::<u32>().ok()
        });

        let tid = caps.name("tid").and_then(|m| {
            let s = unsafe { std::str::from_utf8_unchecked(m.as_bytes()) };
            s.parse::<u32>().ok()
        });

        let component_name = caps
            .name("component")
            .map(|m| unsafe { String::from_utf8_unchecked(m.as_bytes().to_vec()) });
        let process_name = caps
            .name("process")
            .map(|m| unsafe { String::from_utf8_unchecked(m.as_bytes().to_vec()) });
        let hostname = caps
            .name("hostname")
            .map(|m| unsafe { String::from_utf8_unchecked(m.as_bytes().to_vec()) });
        let container = caps
            .name("container")
            .map(|m| unsafe { String::from_utf8_unchecked(m.as_bytes().to_vec()) });

        let metadata = if self.has_extra_groups {
            let mut map = HashMap::new();
            for name in &self.extra_groups {
                if let Some(m) = caps.name(name) {
                    map.insert(name.clone(), unsafe {
                        String::from_utf8_unchecked(m.as_bytes().to_vec())
                    });
                }
            }
            if map.is_empty() {
                None
            } else {
                Some(map)
            }
        } else {
            None
        };

        Some(LogRecord {
            id,
            timestamp,
            level,
            source: Arc::clone(source),
            pid,
            tid,
            component_name,
            process_name,
            hostname,
            container,
            message,
            raw: raw.to_string(),
            metadata,
            loader_id: Arc::clone(loader_id),
        })
    }

    /// Internal: parse using bytes regex with owned raw string.
    #[inline]
    fn parse_bytes_inner_owned(
        &self,
        raw: String,
        bytes_re: &BytesRegex,
        source: &Arc<str>,
        loader_id: &Arc<str>,
        id: u64,
    ) -> Option<LogRecord> {
        let caps = bytes_re.captures(raw.as_bytes())?;

        let timestamp = caps
            .name("timestamp")
            .and_then(|m| {
                let s = unsafe { std::str::from_utf8_unchecked(m.as_bytes()) };
                self.parse_timestamp(s)
            })
            .unwrap_or_else(Utc::now);

        let level = caps.name("level").and_then(|m| {
            let s = unsafe { std::str::from_utf8_unchecked(m.as_bytes()) };
            LogLevel::from_str_loose(s)
        });

        let message = caps
            .name("message")
            .map(|m| unsafe { String::from_utf8_unchecked(m.as_bytes().to_vec()) })
            .unwrap_or_default();

        let pid = caps.name("pid").and_then(|m| {
            let s = unsafe { std::str::from_utf8_unchecked(m.as_bytes()) };
            s.parse::<u32>().ok()
        });

        let tid = caps.name("tid").and_then(|m| {
            let s = unsafe { std::str::from_utf8_unchecked(m.as_bytes()) };
            s.parse::<u32>().ok()
        });

        let component_name = caps
            .name("component")
            .map(|m| unsafe { String::from_utf8_unchecked(m.as_bytes().to_vec()) });
        let process_name = caps
            .name("process")
            .map(|m| unsafe { String::from_utf8_unchecked(m.as_bytes().to_vec()) });
        let hostname = caps
            .name("hostname")
            .map(|m| unsafe { String::from_utf8_unchecked(m.as_bytes().to_vec()) });
        let container = caps
            .name("container")
            .map(|m| unsafe { String::from_utf8_unchecked(m.as_bytes().to_vec()) });

        let metadata = if self.has_extra_groups {
            let mut map = HashMap::new();
            for name in &self.extra_groups {
                if let Some(m) = caps.name(name) {
                    map.insert(name.clone(), unsafe {
                        String::from_utf8_unchecked(m.as_bytes().to_vec())
                    });
                }
            }
            if map.is_empty() {
                None
            } else {
                Some(map)
            }
        } else {
            None
        };

        // Drop caps before consuming raw
        drop(caps);

        Some(LogRecord {
            id,
            timestamp,
            level,
            source: Arc::clone(source),
            pid,
            tid,
            component_name,
            process_name,
            hostname,
            container,
            message,
            raw,
            metadata,
            loader_id: Arc::clone(loader_id),
        })
    }

    /// Build record from text-mode captures (fallback when bytes regex unavailable).
    #[inline]
    fn build_record_from_str_caps(
        &self,
        caps: &regex::Captures<'_>,
        raw: &str,
        source: &Arc<str>,
        loader_id: &Arc<str>,
        id: u64,
    ) -> Option<LogRecord> {
        let timestamp = caps
            .name("timestamp")
            .and_then(|m| self.parse_timestamp(m.as_str()))
            .unwrap_or_else(Utc::now);

        let level = caps
            .name("level")
            .and_then(|m| LogLevel::from_str_loose(m.as_str()));

        let message = caps
            .name("message")
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();

        let pid = caps
            .name("pid")
            .and_then(|m| m.as_str().parse::<u32>().ok());

        let tid = caps
            .name("tid")
            .and_then(|m| m.as_str().parse::<u32>().ok());

        let component_name = caps.name("component").map(|m| m.as_str().to_string());
        let process_name = caps.name("process").map(|m| m.as_str().to_string());
        let hostname = caps.name("hostname").map(|m| m.as_str().to_string());
        let container = caps.name("container").map(|m| m.as_str().to_string());

        let metadata = if self.has_extra_groups {
            let mut map = HashMap::new();
            for name in &self.extra_groups {
                if let Some(m) = caps.name(name) {
                    map.insert(name.clone(), m.as_str().to_string());
                }
            }
            if map.is_empty() {
                None
            } else {
                Some(map)
            }
        } else {
            None
        };

        Some(LogRecord {
            id,
            timestamp,
            level,
            source: Arc::clone(source),
            pid,
            tid,
            component_name,
            process_name,
            hostname,
            container,
            message,
            raw: raw.to_string(),
            metadata,
            loader_id: Arc::clone(loader_id),
        })
    }
}

/// Well-known capture group names that map to LogRecord fields.
const KNOWN_FIELDS: &[&str] = &[
    "timestamp",
    "level",
    "message",
    "pid",
    "tid",
    "component",
    "process",
    "hostname",
    "container",
];

impl LogParser for RegexParser {
    fn parse(&self, raw: &str, source: &str, loader_id: &str, id: u64) -> Option<LogRecord> {
        let source_arc: Arc<str> = Arc::from(source);
        let loader_arc: Arc<str> = Arc::from(loader_id);
        self.parse_shared(raw, &source_arc, &loader_arc, id)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Hand-written fast syslog timestamp parser.
///
/// Parses format: "Feb 19 14:23:45" or "Feb  9 14:23:45" (single-digit day with space padding)
/// Returns DateTime<Utc> using provided year.
#[inline]
fn parse_syslog_timestamp_fast(s: &str, year: i32) -> Option<DateTime<Utc>> {
    let bytes = s.as_bytes();
    if bytes.len() < 15 {
        return None;
    }

    let month = match &bytes[0..3] {
        b"Jan" => 1,
        b"Feb" => 2,
        b"Mar" => 3,
        b"Apr" => 4,
        b"May" => 5,
        b"Jun" => 6,
        b"Jul" => 7,
        b"Aug" => 8,
        b"Sep" => 9,
        b"Oct" => 10,
        b"Nov" => 11,
        b"Dec" => 12,
        _ => return None,
    };

    // Day: bytes[4..6], may be " 9" or "19"
    let day = if bytes[4] == b' ' {
        (bytes[5] - b'0') as u32
    } else {
        ((bytes[4] - b'0') * 10 + (bytes[5] - b'0')) as u32
    };

    let hour = ((bytes[7] - b'0') * 10 + (bytes[8] - b'0')) as u32;
    let min = ((bytes[10] - b'0') * 10 + (bytes[11] - b'0')) as u32;
    let sec = ((bytes[13] - b'0') * 10 + (bytes[14] - b'0')) as u32;

    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    let time = NaiveTime::from_hms_opt(hour, min, sec)?;
    Some(NaiveDateTime::new(date, time).and_utc())
}
