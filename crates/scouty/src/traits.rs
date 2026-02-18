//! Core traits defining the log processing pipeline.

use crate::record::LogRecord;
use std::fmt::Debug;

/// Errors that can occur in the pipeline.
#[derive(Debug, thiserror::Error)]
pub enum ScoutyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Filter error: {0}")]
    Filter(String),
    #[error("Other: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, ScoutyError>;

/// Describes the type of a loader, used by ParserFactory to select parser groups.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoaderType {
    /// Plain text file.
    TextFile,
    /// Compressed archive (gz/zip/7z).
    Archive,
    /// Live syslog stream.
    Syslog,
    /// OTLP (gRPC or HTTP).
    Otlp,
}

/// Metadata about a loader, used by the parser factory.
#[derive(Debug, Clone)]
pub struct LoaderInfo {
    /// Unique identifier for this loader instance.
    pub id: String,
    /// The type of the loader.
    pub loader_type: LoaderType,
    /// Whether this loader's logs may contain multi-line records.
    pub multiline_enabled: bool,
    /// Optional hint: first few lines of the log for parser auto-detection.
    pub sample_lines: Vec<String>,
}

/// A source of raw log lines.
pub trait LogLoader: Debug + Send {
    /// Returns metadata about this loader.
    fn info(&self) -> &LoaderInfo;

    /// Load all lines (for batch sources). Returns raw lines.
    fn load(&mut self) -> Result<Vec<String>>;
}

/// Parses a raw log line (or multi-line block) into a LogRecord.
pub trait LogParser: Debug + Send {
    /// Attempt to parse a raw log string into a LogRecord.
    /// Returns None if this parser cannot handle the input.
    fn parse(&self, raw: &str, source: &str, loader_id: &str, id: u64) -> Option<LogRecord>;

    /// A human-readable name for this parser.
    fn name(&self) -> &str;
}

/// Filters log records.
pub trait LogFilter: Debug + Send {
    /// Returns true if the record matches this filter's condition.
    fn matches(&self, record: &LogRecord) -> bool;

    /// A human-readable description of this filter.
    fn description(&self) -> &str;
}

/// Post-processes log records after storage (placeholder).
pub trait LogProcessor: Debug + Send {
    /// Process a batch of records. Called after all parsing is complete.
    fn process(&self, records: &[LogRecord]) -> Result<()>;

    /// A human-readable name for this processor.
    fn name(&self) -> &str;
}

/// Analyzes log records (placeholder for future use).
pub trait LogAnalyzer: Debug + Send {
    /// Analyze a set of records and return a textual summary.
    fn analyze(&self, records: &[LogRecord]) -> Result<String>;

    /// A human-readable name for this analyzer.
    fn name(&self) -> &str;
}
