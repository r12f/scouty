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
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;

/// A log parser driven by a single regex with named capture groups.
#[derive(Debug)]
pub struct RegexParser {
    name: String,
    pattern: Regex,
    /// Optional timestamp format string (chrono strftime) for parsing the `timestamp` group.
    /// If None, tries ISO 8601 by default.
    timestamp_format: Option<String>,
    /// Whether to use fast syslog timestamp parsing.
    fast_syslog_ts: bool,
    /// Cached current year for syslog timestamp parsing.
    current_year: i32,
    /// Pre-computed list of extra named groups (not in KNOWN_FIELDS).
    extra_groups: Vec<String>,
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

        // Pre-compute extra named groups
        let extra_groups: Vec<String> = regex
            .capture_names()
            .flatten()
            .filter(|n| !KNOWN_FIELDS.contains(n))
            .map(|n| n.to_string())
            .collect();

        // Detect if syslog fast path applies
        let fast_syslog_ts = timestamp_format
            .as_deref()
            .is_some_and(|fmt| fmt == "%b %e %H:%M:%S" || fmt == "%b %d %H:%M:%S");

        Ok(Self {
            name: name.into(),
            pattern: regex,
            timestamp_format,
            fast_syslog_ts,
            current_year: Utc::now().year(),
            extra_groups,
        })
    }

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
        let caps = self.pattern.captures(raw)?;

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

        let metadata = if self.extra_groups.is_empty() {
            None
        } else {
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
            message,
            raw: raw.to_string(),
            metadata,
            loader_id: Arc::clone(loader_id),
        })
    }

    /// Parse taking ownership of the raw String (avoids one allocation).
    pub fn parse_shared_owned(
        &self,
        raw: String,
        source: &Arc<str>,
        loader_id: &Arc<str>,
        id: u64,
    ) -> Option<LogRecord> {
        let caps = self.pattern.captures(&raw)?;

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

        let metadata = if self.extra_groups.is_empty() {
            None
        } else {
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
        };

        // Drop caps before moving raw
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
            message,
            raw,
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
/// Returns DateTime<Utc> using current year.
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
