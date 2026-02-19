//! Log storage, ordered by timestamp.

use crate::record::LogRecord;

/// Stores log records in timestamp-sorted order.
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

    /// Insert a record, maintaining timestamp order.
    pub fn insert(&mut self, record: LogRecord) {
        let pos = self
            .records
            .partition_point(|r| r.timestamp <= record.timestamp);
        self.records.insert(pos, record);
    }

    /// Bulk insert records, then re-sort.
    pub fn insert_batch(&mut self, mut batch: Vec<LogRecord>) {
        self.records.append(&mut batch);
        self.records.sort_by_key(|r| r.timestamp);
    }

    /// Get all records (sorted).
    pub fn records(&self) -> &[LogRecord] {
        &self.records
    }

    /// Number of stored records.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
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
