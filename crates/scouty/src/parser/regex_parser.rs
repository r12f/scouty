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
use chrono::{DateTime, NaiveDateTime, Utc};
use regex::Regex;
use std::collections::HashMap;

/// A log parser driven by a single regex with named capture groups.
#[derive(Debug)]
pub struct RegexParser {
    name: String,
    pattern: Regex,
    /// Optional timestamp format string (chrono strftime) for parsing the `timestamp` group.
    /// If None, tries ISO 8601 by default.
    timestamp_format: Option<String>,
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
        Ok(Self {
            name: name.into(),
            pattern: regex,
            timestamp_format,
        })
    }

    fn parse_timestamp(&self, s: &str) -> Option<DateTime<Utc>> {
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
        let caps = self.pattern.captures(raw)?;

        let timestamp = caps
            .name("timestamp")
            .and_then(|m| self.parse_timestamp(m.as_str()))
            .unwrap_or_else(|| Utc::now());

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

        // Collect any extra named groups into metadata
        let mut metadata = HashMap::new();
        for name in self.pattern.capture_names().flatten() {
            if !KNOWN_FIELDS.contains(&name) {
                if let Some(m) = caps.name(name) {
                    metadata.insert(name.to_string(), m.as_str().to_string());
                }
            }
        }

        Some(LogRecord {
            id,
            timestamp,
            level,
            source: source.to_string(),
            pid,
            tid,
            component_name,
            process_name,
            message,
            raw: raw.to_string(),
            metadata,
            loader_id: loader_id.to_string(),
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}
