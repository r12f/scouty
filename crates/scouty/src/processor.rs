//! Log processor implementations.
//!
//! Processors run after parsing, before filtering. They can inspect or
//! annotate records but should not modify ordering.

#[cfg(test)]
#[path = "processor_tests.rs"]
mod processor_tests;

use crate::record::LogRecord;
use crate::traits::{LogProcessor, Result};

/// A no-op processor that does nothing. Useful as a pipeline placeholder
/// and for testing that the processor stage executes correctly.
#[derive(Debug)]
pub struct NoOpProcessor {
    name: String,
}

impl NoOpProcessor {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl LogProcessor for NoOpProcessor {
    fn process(&self, _records: &[LogRecord]) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// A counting processor that counts records by level. Useful for summary stats.
#[derive(Debug)]
pub struct CountingProcessor {
    name: String,
}

impl CountingProcessor {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Count records, returning a map of level → count.
    pub fn count_by_level(
        records: &[LogRecord],
    ) -> std::collections::HashMap<Option<crate::record::LogLevel>, usize> {
        let mut counts = std::collections::HashMap::new();
        for record in records {
            *counts.entry(record.level).or_insert(0) += 1;
        }
        counts
    }
}

impl LogProcessor for CountingProcessor {
    fn process(&self, _records: &[LogRecord]) -> Result<()> {
        // In a real implementation, this would store counts somewhere accessible.
        // For now, the counting logic is available via count_by_level().
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}
