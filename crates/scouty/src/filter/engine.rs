//! FilterEngine — manages exclude/include filters and produces a filtered view.

use crate::filter::expr;
use crate::filter::expr_filter::ExprFilter;
use crate::record::LogRecord;
use crate::traits::LogFilter;
use tracing::{instrument, warn};

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

    /// Add a filter from an expression string.
    ///
    /// Parses and validates the expression, then wraps it as a `LogFilter`.
    #[instrument(skip(self), fields(expression))]
    pub fn add_expr_filter(
        &mut self,
        action: FilterAction,
        expression: &str,
    ) -> Result<(), String> {
        let parsed = expr::parse(expression)?;
        expr::validate(&parsed)?;
        self.add_filter(action, Box::new(ExprFilter::new(parsed, expression)));
        Ok(())
    }

    /// Remove all filters.
    pub fn clear(&mut self) {
        self.filters.clear();
    }

    /// Check if a single record passes all filters.
    ///
    /// Same logic as `apply_iter` but for one record.
    pub fn matches(&self, record: &LogRecord) -> bool {
        // Step 1: check excludes
        for entry in &self.filters {
            if entry.action == FilterAction::Exclude && entry.filter.matches(record) {
                return false;
            }
        }
        // Step 2: check includes
        let has_includes = self
            .filters
            .iter()
            .any(|e| e.action == FilterAction::Include);
        if has_includes {
            self.filters
                .iter()
                .any(|e| e.action == FilterAction::Include && e.filter.matches(record))
        } else {
            true
        }
    }

    /// Apply all filters to the records.
    /// Returns indices of records that pass the filter pipeline.
    ///
    /// Logic:
    /// 1. Any record matching an Exclude filter is excluded.
    /// 2. If there are Include filters, only records matching at least one are included.
    /// 3. If there are no Include filters, all non-excluded records are included.
    #[instrument(skip(self, records), fields(record_count = records.len()))]
    pub fn apply(&self, records: &[LogRecord]) -> Vec<usize> {
        self.apply_iter(records.iter())
    }

    /// Apply all filters using an iterator over records.
    /// Returns indices of records that pass the filter pipeline.
    pub fn apply_iter<'a>(&self, records: impl Iterator<Item = &'a LogRecord>) -> Vec<usize> {
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

#[cfg(test)]
#[path = "engine_tests.rs"]
mod engine_tests;
