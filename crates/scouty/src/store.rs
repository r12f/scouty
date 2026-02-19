//! Log storage, ordered by timestamp.

use crate::record::LogRecord;
use chrono::{DateTime, Utc};

/// Stores log records in timestamp-sorted order.
///
/// Uses a Vec for cache-friendly sequential access (important for TUI scrolling).
/// Batch inserts use append+sort for efficiency. Single live inserts use binary
/// search + insert which is O(n) for the shift but acceptable for streaming rates.
#[derive(Debug)]
pub struct LogStore {
    records: Vec<LogRecord>,
}

impl LogStore {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// Create a store with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            records: Vec::with_capacity(capacity),
        }
    }

    /// Insert a single record, maintaining timestamp order.
    /// For live streaming inserts — uses binary search to find position.
    pub fn insert(&mut self, record: LogRecord) {
        let pos = self
            .records
            .partition_point(|r| r.timestamp <= record.timestamp);
        self.records.insert(pos, record);
    }

    /// Bulk insert records, then re-sort. More efficient than repeated single inserts.
    pub fn insert_batch(&mut self, mut batch: Vec<LogRecord>) {
        self.records.append(&mut batch);
        self.records.sort_by_key(|r| r.timestamp);
    }

    /// Get all records (sorted by timestamp).
    pub fn records(&self) -> &[LogRecord] {
        &self.records
    }

    /// Get a record by index.
    pub fn get(&self, index: usize) -> Option<&LogRecord> {
        self.records.get(index)
    }

    /// Number of stored records.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Find the index of the first record at or after the given timestamp.
    /// Useful for time-based jumping in the TUI.
    pub fn find_by_timestamp(&self, ts: &DateTime<Utc>) -> usize {
        self.records.partition_point(|r| r.timestamp < *ts)
    }

    /// Get a slice of records in the given index range.
    /// Useful for paginated TUI display.
    pub fn range(&self, start: usize, end: usize) -> &[LogRecord] {
        let end = end.min(self.records.len());
        let start = start.min(end);
        &self.records[start..end]
    }

    /// Clear all records.
    pub fn clear(&mut self) {
        self.records.clear();
    }
}

impl Default for LogStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod store_tests;
