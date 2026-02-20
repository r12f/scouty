//! LogSession — top-level orchestrator for a log viewing session.

use crate::filter::engine::FilterEngine;
use crate::parser::group::ParserGroup;
use crate::record::LogRecord;
use crate::store::LogStore;
use crate::traits::{LogLoader, LogProcessor, Result};
use crate::view::LogStoreView;
use rayon::prelude::*;
use std::sync::mpsc;
use std::sync::Arc;

/// Represents a registered loader paired with its parser group.
struct LoaderSlot {
    loader: Box<dyn LogLoader>,
    parser_group: ParserGroup,
}

/// The top-level session managing the full pipeline:
/// Load → Parse → Store → Process → Filter → Filtered View.
pub struct LogSession {
    loader_slots: Vec<LoaderSlot>,
    store: LogStore,
    processors: Vec<Box<dyn LogProcessor>>,
    /// Currently active view — serves TUI, always has valid results.
    active_view: LogStoreView,
    /// Pending view — created when filter changes, not yet applied (sync path).
    pending_view: Option<LogStoreView>,
    /// Receiver for async background filtering result.
    /// When present, a background thread is filtering.
    async_pending: Option<mpsc::Receiver<LogStoreView>>,
    /// Records that failed parsing by all parsers in their group.
    pub failing_parsing_logs: Vec<FailedLog>,
    /// Auto-incrementing record ID counter.
    next_id: u64,
}

/// A log line that could not be parsed.
#[derive(Debug, Clone)]
pub struct FailedLog {
    pub raw: String,
    pub source: String,
    pub loader_id: String,
}

impl LogSession {
    /// Create a new empty session.
    pub fn new() -> Self {
        Self {
            loader_slots: Vec::new(),
            store: LogStore::new(),
            processors: Vec::new(),
            active_view: LogStoreView::new(FilterEngine::new()),
            pending_view: None,
            async_pending: None,
            failing_parsing_logs: Vec::new(),
            next_id: 0,
        }
    }

    /// Register a loader with its associated parser group.
    pub fn add_loader(&mut self, loader: Box<dyn LogLoader>, parser_group: ParserGroup) {
        self.loader_slots.push(LoaderSlot {
            loader,
            parser_group,
        });
    }

    /// Register a post-processor.
    pub fn add_processor(&mut self, processor: Box<dyn LogProcessor>) {
        self.processors.push(processor);
    }

    /// Access the filter engine of the active view for adding/removing filters.
    ///
    /// Note: after modifying filters, call `apply_pending()` or use `update_filter()`
    /// for the dual-buffer workflow.
    pub fn filter_engine_mut(&mut self) -> &mut FilterEngine {
        self.active_view.filter_engine_mut()
    }

    /// Access the store.
    pub fn store(&self) -> &LogStore {
        &self.store
    }

    /// Get the currently active view.
    pub fn active_view(&self) -> &LogStoreView {
        &self.active_view
    }

    /// Whether a pending view exists (sync or async filter update in progress).
    pub fn has_pending_view(&self) -> bool {
        self.pending_view.is_some() || self.async_pending.is_some()
    }

    /// Whether an async background filter is in progress.
    pub fn is_filtering(&self) -> bool {
        self.async_pending.is_some()
    }

    /// Start a filter update using the synchronous dual-buffer mechanism.
    ///
    /// Creates a new pending view with the given filter engine.
    /// If a pending view already exists, it is discarded and replaced.
    /// Also cancels any in-flight async filtering.
    pub fn update_filter(&mut self, filter_engine: FilterEngine) {
        self.async_pending = None; // cancel any async work
        self.pending_view = Some(LogStoreView::new(filter_engine));
    }

    /// Start a filter update that runs in a background thread.
    ///
    /// Snapshots current store records and spawns a thread to apply the filter.
    /// Call `poll_pending()` to check for completion and swap views.
    /// If called again before completion, the previous async work is cancelled
    /// (receiver dropped, thread result discarded).
    pub fn update_filter_async(&mut self, filter_engine: FilterEngine) {
        self.pending_view = None; // discard sync pending
                                  // Share store records with background thread via Arc (zero-copy, no deep clone)
        let records: Arc<[Arc<LogRecord>]> = self.store.iter_arc().cloned().collect();
        let total_count = records.len();

        let (tx, rx) = mpsc::channel();
        self.async_pending = Some(rx);

        std::thread::spawn(move || {
            let mut view = LogStoreView::new(filter_engine);
            view.apply_from_records(records.iter().map(|r| r.as_ref()), total_count);
            // Send might fail if receiver was dropped (cancelled) — that's OK
            let _ = tx.send(view);
        });
    }

    /// Poll for async filtering completion. If the background thread finished,
    /// swap the completed view into active_view and return `true`.
    /// Returns `false` if no async work is pending or it hasn't completed yet.
    pub fn poll_pending(&mut self) -> bool {
        if let Some(ref rx) = self.async_pending {
            match rx.try_recv() {
                Ok(view) => {
                    self.active_view = view;
                    self.async_pending = None;
                    true
                }
                Err(mpsc::TryRecvError::Empty) => false,
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Thread finished but send failed (shouldn't happen normally)
                    self.async_pending = None;
                    false
                }
            }
        } else {
            false
        }
    }

    /// Apply the pending view's filter against the store, then replace active view.
    /// For synchronous pending views only.
    ///
    /// If no sync pending view exists, this is a no-op.
    pub fn apply_pending(&mut self) {
        if let Some(mut pending) = self.pending_view.take() {
            pending.apply(&self.store);
            self.active_view = pending;
        }
    }

    /// Re-apply the active view's filter against the store.
    ///
    /// Use after modifying filters via `filter_engine_mut()`.
    pub fn refresh_active_view(&mut self) {
        self.active_view.apply(&self.store);
    }

    /// Execute the full pipeline: Load → Parse → Store → Process → Filter.
    /// Returns the filtered view (indices into the store).
    pub fn run(&mut self) -> Result<Vec<usize>> {
        // 1. Load + Parse
        for slot in &mut self.loader_slots {
            let info = slot.loader.info().clone();
            let lines = slot.loader.load()?;
            let source = &info.id;

            for line in &lines {
                let id = self.next_id;
                self.next_id += 1;

                match slot.parser_group.parse(line, source, &info.id, id) {
                    Some(record) => {
                        self.store.insert(record);
                    }
                    None => {
                        self.failing_parsing_logs.push(FailedLog {
                            raw: line.clone(),
                            source: source.clone(),
                            loader_id: info.id.clone(),
                        });
                    }
                }
            }
        }

        // 2. Process (collect only if processors exist)
        if !self.processors.is_empty() {
            let records: Vec<LogRecord> = self.store.iter().cloned().collect();
            for processor in &self.processors {
                processor.process(&records)?;
            }
        }

        // 3. Apply active view filter
        self.active_view.apply(&self.store);
        Ok(self.active_view.indices().to_vec())
    }

    /// Get the filtered view based on current active view's cache.
    pub fn filtered_view(&self) -> Vec<usize> {
        self.active_view.indices().to_vec()
    }

    /// Execute the pipeline with parallel loading and parsing across loader slots.
    pub fn run_parallel(&mut self) -> Result<Vec<usize>> {
        // 1. Load + Parse in parallel
        let results: Vec<Result<(Vec<LogRecord>, Vec<FailedLog>)>> = self
            .loader_slots
            .par_iter_mut()
            .map(|slot| {
                let info = slot.loader.info().clone();
                let lines = slot.loader.load()?;
                let source = &info.id;

                let mut records = Vec::new();
                let mut failures = Vec::new();

                for (i, line) in lines.iter().enumerate() {
                    match slot.parser_group.parse(line, source, &info.id, i as u64) {
                        Some(record) => records.push(record),
                        None => failures.push(FailedLog {
                            raw: line.clone(),
                            source: source.clone(),
                            loader_id: info.id.clone(),
                        }),
                    }
                }

                Ok((records, failures))
            })
            .collect();

        // 2. Merge results sequentially
        for result in results {
            let (records, failures) = result?;
            for mut record in records {
                record.id = self.next_id;
                self.next_id += 1;
                self.store.insert(record);
            }
            self.failing_parsing_logs.extend(failures);
        }

        // 3. Process (collect only if processors exist)
        if !self.processors.is_empty() {
            let records: Vec<LogRecord> = self.store.iter().cloned().collect();
            for processor in &self.processors {
                processor.process(&records)?;
            }
        }

        // 4. Apply active view filter
        self.active_view.apply(&self.store);
        Ok(self.active_view.indices().to_vec())
    }
}

impl Default for LogSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod session_tests;
