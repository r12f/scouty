//! Multi-line log merging.
//!
//! Merges continuation lines into the preceding log entry based on a
//! "start pattern" regex. Lines that match the start pattern begin a new
//! record; lines that don't are appended to the current record.

#[cfg(test)]
#[path = "multiline_tests.rs"]
mod multiline_tests;

use regex::Regex;

/// Merges raw log lines into multi-line blocks.
///
/// The `start_pattern` regex identifies lines that begin a new log entry.
/// All subsequent lines that do *not* match are appended to the current block.
#[derive(Debug)]
pub struct MultilineMerger {
    start_pattern: Regex,
    separator: String,
}

impl MultilineMerger {
    /// Create a new merger.
    ///
    /// `start_pattern` — regex that matches the first line of a new log entry.
    /// `separator` — string inserted between merged lines (typically `"\n"`).
    pub fn new(start_pattern: &str, separator: impl Into<String>) -> Result<Self, String> {
        let re = Regex::new(start_pattern)
            .map_err(|e| format!("Invalid start pattern '{}': {}", start_pattern, e))?;
        Ok(Self {
            start_pattern: re,
            separator: separator.into(),
        })
    }

    /// Merge a sequence of raw lines into multi-line blocks.
    ///
    /// Returns a vec of merged strings, each representing one logical log entry.
    pub fn merge(&self, lines: &[String]) -> Vec<String> {
        let mut blocks: Vec<String> = Vec::new();
        let mut current: Option<String> = None;

        for line in lines {
            if self.start_pattern.is_match(line) {
                // This line starts a new block
                if let Some(block) = current.take() {
                    blocks.push(block);
                }
                current = Some(line.clone());
            } else {
                // Continuation line
                match &mut current {
                    Some(block) => {
                        block.push_str(&self.separator);
                        block.push_str(line);
                    }
                    None => {
                        // Orphan continuation line — treat as its own block
                        current = Some(line.clone());
                    }
                }
            }
        }

        // Flush last block
        if let Some(block) = current {
            blocks.push(block);
        }

        blocks
    }
}
