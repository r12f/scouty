//! LogStoreView — encapsulates a FilterEngine + cached filtered indices.
//!
//! Provides a snapshot of filter results that can be queried without re-filtering.
//! Used by LogSession's dual-buffer mechanism for non-blocking filter updates.

#[cfg(test)]
#[path = "view_tests.rs"]
mod view_tests;

use crate::filter::engine::FilterEngine;
use crate::record::LogRecord;
use crate::store::LogStore;

/// Status of a LogStoreView's filter results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewStatus {
    /// Filter has been applied, results are ready.
    Ready,
    /// Filter is being applied (results may be stale or empty).
    Filtering,
}

/// Encapsulates a FilterEngine and its cached filter results (indices into LogStore).
///
/// Does not own the LogStore — borrows it during `apply()`.
#[derive(Debug)]
pub struct LogStoreView {
    filter_engine: FilterEngine,
    filtered_indices: Vec<usize>,
    status: ViewStatus,
}

impl LogStoreView {
    /// Create a new view with the given filter engine. Status starts as Filtering
    /// (no results until `apply()` is called).
    pub fn new(filter_engine: FilterEngine) -> Self {
        Self {
            filter_engine,
            filtered_indices: Vec::new(),
            status: ViewStatus::Filtering,
        }
    }

    /// Apply the filter engine against the store, updating cached indices.
    pub fn apply(&mut self, store: &LogStore) {
        self.status = ViewStatus::Filtering;
        self.filtered_indices = self.filter_engine.apply_iter(store.iter());
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
}
