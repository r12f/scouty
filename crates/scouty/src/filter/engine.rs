//! FilterEngine — manages exclude/include filters and produces a filtered view.

use crate::record::LogRecord;
use crate::traits::LogFilter;

/// The action to take when a filter matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterAction {
    /// Exclude matching records.
    Exclude,
    /// Include matching records.
    Include,
}

/// A filter with an associated action.
#[derive(Debug)]
pub struct FilterEntry {
    pub action: FilterAction,
    pub filter: Box<dyn LogFilter>,
}

/// Manages all filters and applies exclude-first → include logic.
#[derive(Debug)]
pub struct FilterEngine {
    filters: Vec<FilterEntry>,
}

impl FilterEngine {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    /// Add a filter with the given action.
    pub fn add_filter(&mut self, action: FilterAction, filter: Box<dyn LogFilter>) {
        self.filters.push(FilterEntry { action, filter });
    }

    /// Remove all filters.
    pub fn clear(&mut self) {
        self.filters.clear();
    }

    /// Apply all filters to the records.
    /// Returns indices of records that pass the filter pipeline.
    ///
    /// Logic:
    /// 1. Any record matching an Exclude filter is excluded.
    /// 2. If there are Include filters, only records matching at least one are included.
    /// 3. If there are no Include filters, all non-excluded records are included.
    pub fn apply(&self, records: &[LogRecord]) -> Vec<usize> {
        let excludes: Vec<&dyn LogFilter> = self
            .filters
            .iter()
            .filter(|e| e.action == FilterAction::Exclude)
            .map(|e| e.filter.as_ref())
            .collect();

        let includes: Vec<&dyn LogFilter> = self
            .filters
            .iter()
            .filter(|e| e.action == FilterAction::Include)
            .map(|e| e.filter.as_ref())
            .collect();

        let has_includes = !includes.is_empty();

        records
            .iter()
            .enumerate()
            .filter(|(_, record)| {
                // Step 1: check excludes
                if excludes.iter().any(|f| f.matches(record)) {
                    return false;
                }
                // Step 2: check includes
                if has_includes {
                    includes.iter().any(|f| f.matches(record))
                } else {
                    true
                }
            })
            .map(|(i, _)| i)
            .collect()
    }
}

impl Default for FilterEngine {
    fn default() -> Self {
        Self::new()
    }
}
