//! Core data types for log records.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Log severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Notice,
    Warn,
    Error,
    Fatal,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Notice => write!(f, "NOTICE"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Fatal => write!(f, "FATAL"),
        }
    }
}

impl LogLevel {
    /// Parse a log level from a string (case-insensitive, zero-allocation).
    pub fn from_str_loose(s: &str) -> Option<LogLevel> {
        // Fast path: match common patterns without allocating
        let bytes = s.as_bytes();
        if bytes.is_empty() {
            return None;
        }
        match bytes[0] | 0x20 {
            // lowercase first byte
            b't' => match bytes.len() {
                5 if s.eq_ignore_ascii_case("TRACE") => Some(LogLevel::Trace),
                3 if s.eq_ignore_ascii_case("TRC") => Some(LogLevel::Trace),
                _ => None,
            },
            b'd' => match bytes.len() {
                5 if s.eq_ignore_ascii_case("DEBUG") => Some(LogLevel::Debug),
                3 if s.eq_ignore_ascii_case("DBG") => Some(LogLevel::Debug),
                _ => None,
            },
            b'i' => match bytes.len() {
                4 if s.eq_ignore_ascii_case("INFO") => Some(LogLevel::Info),
                3 if s.eq_ignore_ascii_case("INF") => Some(LogLevel::Info),
                _ => None,
            },
            b'n' => match bytes.len() {
                6 if s.eq_ignore_ascii_case("NOTICE") => Some(LogLevel::Notice),
                _ => None,
            },
            b'w' => match bytes.len() {
                4 if s.eq_ignore_ascii_case("WARN") => Some(LogLevel::Warn),
                7 if s.eq_ignore_ascii_case("WARNING") => Some(LogLevel::Warn),
                3 if s.eq_ignore_ascii_case("WRN") => Some(LogLevel::Warn),
                _ => None,
            },
            b'e' => match bytes.len() {
                5 if s.eq_ignore_ascii_case("ERROR") => Some(LogLevel::Error),
                3 if s.eq_ignore_ascii_case("ERR") => Some(LogLevel::Error),
                _ => None,
            },
            b'f' => match bytes.len() {
                5 if s.eq_ignore_ascii_case("FATAL") => Some(LogLevel::Fatal),
                3 if s.eq_ignore_ascii_case("FTL") => Some(LogLevel::Fatal),
                _ => None,
            },
            b'c' => match bytes.len() {
                8 if s.eq_ignore_ascii_case("CRITICAL") => Some(LogLevel::Fatal),
                4 if s.eq_ignore_ascii_case("CRIT") => Some(LogLevel::Fatal),
                _ => None,
            },
            _ => None,
        }
    }
}

/// A single parsed log record. Immutable after parsing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    /// Unique ID within the session.
    pub id: u64,
    /// Log timestamp.
    pub timestamp: DateTime<Utc>,
    /// Log severity level.
    pub level: Option<LogLevel>,
    /// Log source identifier (e.g. filename, syslog host).
    pub source: Arc<str>,
    /// Process ID.
    pub pid: Option<u32>,
    /// Thread ID.
    pub tid: Option<u32>,
    /// Component name.
    pub component_name: Option<String>,
    /// Process name.
    pub process_name: Option<String>,
    /// Hostname.
    pub hostname: Option<String>,
    /// Container name.
    pub container: Option<String>,
    /// Log message body.
    pub message: String,
    /// Raw original log text (may be multi-line).
    pub raw: String,
    /// Extensible key-value metadata (None when no extra fields).
    pub metadata: Option<HashMap<String, String>>,
    /// Which loader produced this record.
    pub loader_id: Arc<str>,
}

#[cfg(test)]
#[path = "record_tests.rs"]
mod record_tests;
