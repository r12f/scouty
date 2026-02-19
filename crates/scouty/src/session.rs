//! LogSession — top-level orchestrator for a log viewing session.

use crate::filter::engine::FilterEngine;
use crate::record::LogRecord;
use crate::store::LogStore;
use crate::traits::{LogLoader, LogParser, LogProcessor, Result};

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
    filter_engine: FilterEngine,
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
            filter_engine: FilterEngine::new(),
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

    /// Access the filter engine for adding/removing filters.
    pub fn filter_engine_mut(&mut self) -> &mut FilterEngine {
        &mut self.filter_engine
    }

    /// Access the store.
    pub fn store(&self) -> &LogStore {
        &self.store
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

        // 2. Process
        let records = self.store.records();
        for processor in &self.processors {
            processor.process(records)?;
        }

        // 3. Filter → Filtered View
        let filtered = self.filter_engine.apply(records);
        Ok(filtered)
    }

    /// Get the filtered view based on current filters (without re-running load/parse).
    pub fn filtered_view(&self) -> Vec<usize> {
        self.filter_engine.apply(self.store.records())
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
