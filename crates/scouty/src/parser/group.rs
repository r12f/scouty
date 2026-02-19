//! Parser group — an ordered fallback chain of parsers.

#[cfg(test)]
#[path = "group_tests.rs"]
mod group_tests;

use crate::record::LogRecord;
use crate::traits::LogParser;

/// A parser group: an ordered list of parsers tried in sequence (fallback chain).
#[derive(Debug)]
pub struct ParserGroup {
    /// Human-readable name of this group.
    pub name: String,
    /// Ordered list of parsers; tried first-to-last.
    pub parsers: Vec<Box<dyn LogParser>>,
}

impl ParserGroup {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            parsers: Vec::new(),
        }
    }

    pub fn add_parser(&mut self, parser: Box<dyn LogParser>) {
        self.parsers.push(parser);
    }

    /// Try each parser in order. Returns the first successful parse, or None.
    pub fn parse(&self, raw: &str, source: &str, loader_id: &str, id: u64) -> Option<LogRecord> {
        for parser in &self.parsers {
            if let Some(record) = parser.parse(raw, source, loader_id, id) {
                return Some(record);
            }
        }
        None
    }
}
