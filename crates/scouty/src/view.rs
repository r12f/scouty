//! LogStoreView — encapsulates a FilterEngine + cached filtered indices.
//!
//! Provides a snapshot of filter results that can be queried without re-filtering.
//! Used by LogSession's dual-buffer mechanism for non-blocking filter updates.

#[cfg(test)]
#[path = "view_tests.rs"]
mod view_tests;

use crate::filter::engine::FilterEngine;
use crate::record::{LogLevel, LogRecord};
use crate::store::LogStore;
use std::collections::HashMap;

/// Status of a LogStoreView's filter results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewStatus {
    /// Filter has been applied, results are ready.
    Ready,
    /// Filter is being applied (results may be stale or empty).
    Filtering,
}

/// Statistics about the view's filtered results.
#[derive(Debug, Clone, Default)]
pub struct ViewStats {
    /// Total records in the store at last apply.
    pub total_records: usize,
    /// Records passing the filter.
    pub filtered_records: usize,
    /// Per-level counts in the store (pre-filter).
    pub level_counts_total: HashMap<Option<LogLevel>, usize>,
    /// Per-level counts in the filtered view (post-filter).
    pub level_counts_filtered: HashMap<Option<LogLevel>, usize>,
}

impl ViewStats {
    /// Filter rate: fraction of records excluded (0.0 = no filtering, 1.0 = all filtered out).
    pub fn filter_rate(&self) -> f64 {
        if self.total_records == 0 {
            return 0.0;
        }
        1.0 - (self.filtered_records as f64 / self.total_records as f64)
    }
}

/// Encapsulates a FilterEngine and its cached filter results (indices into LogStore).
///
/// Does not own the LogStore — borrows it during `apply()`.
#[derive(Debug)]
pub struct LogStoreView {
    filter_engine: FilterEngine,
    filtered_indices: Vec<usize>,
    status: ViewStatus,
    /// Number of store records processed so far (for incremental apply).
    last_applied_count: usize,
    /// Cached statistics.
    stats: ViewStats,
}

impl LogStoreView {
    /// Create a new view with the given filter engine. Status starts as Filtering
    /// (no results until `apply()` is called).
    pub fn new(filter_engine: FilterEngine) -> Self {
        Self {
            filter_engine,
            filtered_indices: Vec::new(),
            status: ViewStatus::Filtering,
            last_applied_count: 0,
            stats: ViewStats::default(),
        }
    }

    /// Apply the filter engine against the full store, updating cached indices and stats.
    pub fn apply(&mut self, store: &LogStore) {
        self.status = ViewStatus::Filtering;
        self.filtered_indices = self.filter_engine.apply_iter(store.iter());
        self.last_applied_count = store.len();
        self.rebuild_stats(store);
        self.status = ViewStatus::Ready;
    }

    /// Incrementally apply the filter to only new records appended since last apply.
    ///
    /// Only processes records at indices >= `last_applied_count`. For live log
    /// streaming scenarios (tail -f, OTLP). Stats are also updated incrementally.
    pub fn apply_incremental(&mut self, store: &LogStore) {
        let current_len = store.len();
        if current_len <= self.last_applied_count {
            return;
        }

        // Process only new records: update stats and filter incrementally
        let new_records = store.iter().skip(self.last_applied_count);
        let mut idx = self.last_applied_count;
        for record in new_records {
            // Update total level counts
            *self
                .stats
                .level_counts_total
                .entry(record.level)
                .or_insert(0) += 1;

            // Check filter and update filtered indices + filtered level counts
            if self.filter_engine.matches(record) {
                self.filtered_indices.push(idx);
                *self
                    .stats
                    .level_counts_filtered
                    .entry(record.level)
                    .or_insert(0) += 1;
            }
            idx += 1;
        }

        self.last_applied_count = current_len;
        self.stats.total_records = current_len;
        self.stats.filtered_records = self.filtered_indices.len();
        self.status = ViewStatus::Ready;
    }

    /// Apply the filter engine against a record iterator, updating cached indices and stats.
    ///
    /// Unlike `apply()`, this doesn't require a `&LogStore` — useful for background
    /// filtering where records are shared via `Arc`.
    pub fn apply_from_records<'a>(
        &mut self,
        records: impl Iterator<Item = &'a LogRecord>,
        total_count: usize,
    ) {
        self.status = ViewStatus::Filtering;
        let mut level_counts_total: HashMap<Option<LogLevel>, usize> = HashMap::new();
        let mut level_counts_filtered: HashMap<Option<LogLevel>, usize> = HashMap::new();
        self.filtered_indices.clear();

        for (i, record) in records.enumerate() {
            *level_counts_total.entry(record.level).or_insert(0) += 1;
            if self.filter_engine.matches(record) {
                self.filtered_indices.push(i);
                *level_counts_filtered.entry(record.level).or_insert(0) += 1;
            }
        }

        self.last_applied_count = total_count;
        self.stats = ViewStats {
            total_records: total_count,
            filtered_records: self.filtered_indices.len(),
            level_counts_total,
            level_counts_filtered,
        };
        self.status = ViewStatus::Ready;
    }

    /// Get the cached filtered indices.
    pub fn indices(&self) -> &[usize] {
        &self.filtered_indices
    }

    /// Access the filter engine (read-only).
    pub fn filter_engine(&self) -> &FilterEngine {
        &self.filter_engine
    }

    /// Access the filter engine (mutable).
    pub fn filter_engine_mut(&mut self) -> &mut FilterEngine {
        &mut self.filter_engine
    }

    /// Number of records in the filtered view.
    pub fn len(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Whether the filtered view is empty.
    pub fn is_empty(&self) -> bool {
        self.filtered_indices.is_empty()
    }

    /// Get a record by filtered index (0-based index into the filtered view).
    pub fn get_record<'a>(&self, index: usize, store: &'a LogStore) -> Option<&'a LogRecord> {
        let store_index = *self.filtered_indices.get(index)?;
        store.get(store_index)
    }

    /// Current status of the view.
    pub fn status(&self) -> ViewStatus {
        self.status
    }

    /// Get the current view statistics.
    pub fn stats(&self) -> &ViewStats {
        &self.stats
    }

    /// Number of store records already processed by this view.
    pub fn last_applied_count(&self) -> usize {
        self.last_applied_count
    }

    /// Rebuild statistics from current state.
    fn rebuild_stats(&mut self, store: &LogStore) {
        let mut level_counts_total: HashMap<Option<LogLevel>, usize> = HashMap::new();
        let mut level_counts_filtered: HashMap<Option<LogLevel>, usize> = HashMap::new();

        for record in store.iter() {
            *level_counts_total.entry(record.level).or_insert(0) += 1;
        }

        for &idx in &self.filtered_indices {
            if let Some(record) = store.get(idx) {
                *level_counts_filtered.entry(record.level).or_insert(0) += 1;
            }
        }

        self.stats = ViewStats {
            total_records: store.len(),
            filtered_records: self.filtered_indices.len(),
            level_counts_total,
            level_counts_filtered,
        };
    }
}
