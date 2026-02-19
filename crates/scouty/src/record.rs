//! Core data types for log records.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Log severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
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
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Fatal => write!(f, "FATAL"),
        }
    }
}

impl LogLevel {
    /// Parse a log level from a string (case-insensitive).
    pub fn from_str_loose(s: &str) -> Option<LogLevel> {
        match s.to_uppercase().as_str() {
            "TRACE" | "TRC" => Some(LogLevel::Trace),
            "DEBUG" | "DBG" => Some(LogLevel::Debug),
            "INFO" | "INF" => Some(LogLevel::Info),
            "WARN" | "WARNING" | "WRN" => Some(LogLevel::Warn),
            "ERROR" | "ERR" => Some(LogLevel::Error),
            "FATAL" | "CRITICAL" | "CRIT" | "FTL" => Some(LogLevel::Fatal),
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
    pub source: String,
    /// Process ID.
    pub pid: Option<u32>,
    /// Thread ID.
    pub tid: Option<u32>,
    /// Component name.
    pub component_name: Option<String>,
    /// Process name.
    pub process_name: Option<String>,
    /// Log message body.
    pub message: String,
    /// Raw original log text (may be multi-line).
    pub raw: String,
    /// Extensible key-value metadata.
    pub metadata: HashMap<String, String>,
    /// Which loader produced this record.
    pub loader_id: String,
}

#[cfg(test)]
#[path = "record_tests.rs"]
mod record_tests;
