//! Log categorization — data model, config loading, and stats tracking.
//!
//! Categories are named classification rules with filter expressions.
//! Each incoming log record is evaluated against all categories;
//! matching records increment the category's count and density histogram.

#[cfg(test)]
#[path = "category_tests.rs"]
mod category_tests;

use crate::filter::eval;
use crate::filter::expr::{self, Expr};
use crate::record::LogRecord;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::path::Path;

// ── Data Model ──────────────────────────────────────────────────────

/// A parsed category definition ready for evaluation.
#[derive(Debug)]
pub struct CategoryDefinition {
    pub name: String,
    pub filter: Expr,
}

/// Per-category runtime statistics.
#[derive(Debug)]
pub struct CategoryStats {
    pub definition: CategoryDefinition,
    pub count: usize,
    /// Time-bucketed histogram (same bucketing as density chart).
    pub density: Vec<u64>,
}

impl CategoryStats {
    pub fn new(definition: CategoryDefinition, bucket_count: usize) -> Self {
        Self {
            definition,
            count: 0,
            density: vec![0; bucket_count],
        }
    }

    /// Record a match, optionally updating the density bucket.
    pub fn record_match(&mut self, bucket_index: Option<usize>) {
        self.count += 1;
        if let Some(idx) = bucket_index {
            if idx < self.density.len() {
                self.density[idx] += 1;
            }
        }
    }

    /// Resize density histogram (e.g., when time range changes).
    pub fn resize_density(&mut self, new_len: usize) {
        self.density.resize(new_len, 0);
    }
}

/// Collection of category stats, ordered as defined in config.
#[derive(Debug, Default)]
pub struct CategoryStore {
    pub categories: Vec<CategoryStats>,
}

impl CategoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a store from parsed definitions with a given density bucket count.
    pub fn from_definitions(definitions: Vec<CategoryDefinition>, bucket_count: usize) -> Self {
        let categories = definitions
            .into_iter()
            .map(|d| CategoryStats::new(d, bucket_count))
            .collect();
        Self { categories }
    }

    /// Reset all counts and density data.
    pub fn reset(&mut self) {
        for cat in &mut self.categories {
            cat.count = 0;
            cat.density.fill(0);
        }
    }
}

// ── Config Loading ──────────────────────────────────────────────────

/// Raw YAML category entry (before filter parsing).
#[derive(Debug, Deserialize)]
struct RawCategory {
    name: String,
    filter: String,
}

/// Raw YAML config file.
#[derive(Debug, Deserialize)]
struct RawCategoryConfig {
    categories: Vec<RawCategory>,
}

/// Load category definitions from a single YAML file.
/// Invalid filters produce warnings (via the returned Vec) and are skipped.
pub(crate) fn load_file(path: &Path) -> (Vec<CategoryDefinition>, Vec<String>) {
    let mut definitions = Vec::new();
    let mut warnings = Vec::new();

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            warnings.push(format!("Failed to read {}: {}", path.display(), e));
            return (definitions, warnings);
        }
    };

    let config: RawCategoryConfig = match serde_yaml::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            warnings.push(format!("Failed to parse {}: {}", path.display(), e));
            return (definitions, warnings);
        }
    };

    for raw in config.categories {
        match expr::parse(&raw.filter) {
            Ok(filter) => {
                tracing::debug!(name = %raw.name, filter = %raw.filter, "Loaded category");
                definitions.push(CategoryDefinition {
                    name: raw.name,
                    filter,
                });
            }
            Err(e) => {
                warnings.push(format!(
                    "Category '{}' has invalid filter '{}': {}",
                    raw.name, raw.filter, e
                ));
            }
        }
    }

    (definitions, warnings)
}

/// Load category definitions from all standard config directories.
/// Returns definitions in precedence order (system → user → project).
pub fn load_categories() -> (Vec<CategoryDefinition>, Vec<String>) {
    let mut all_definitions = Vec::new();
    let mut all_warnings = Vec::new();

    let dirs = category_config_dirs();
    for dir in dirs {
        if !dir.exists() {
            continue;
        }
        tracing::debug!(dir = %dir.display(), "Scanning category config directory");

        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(e) => {
                all_warnings.push(format!("Failed to read directory {}: {}", dir.display(), e));
                continue;
            }
        };

        let mut files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "yaml" || ext == "yml")
                    .unwrap_or(false)
            })
            .collect();

        // Sort for deterministic order
        files.sort_by_key(|e| e.file_name());

        for entry in files {
            let (defs, warns) = load_file(&entry.path());
            all_definitions.extend(defs);
            all_warnings.extend(warns);
        }
    }

    (all_definitions, all_warnings)
}

/// Return the standard category config directories in precedence order.
fn category_config_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = Vec::new();

    // System
    dirs.push(std::path::PathBuf::from("/etc/scouty/categories"));

    // User
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".scouty").join("categories"));
    }

    // Project
    dirs.push(std::path::PathBuf::from("./scouty-categories"));

    dirs
}

// ── Categorization Processor ────────────────────────────────────────

/// Categorization processor — evaluates log records against all category
/// definitions and updates per-category stats (count + density histogram).
pub struct CategoryProcessor {
    pub store: CategoryStore,
    bucket_count: usize,
}

impl CategoryProcessor {
    /// Create a new processor from definitions with a given density bucket count.
    pub fn new(definitions: Vec<CategoryDefinition>, bucket_count: usize) -> Self {
        Self {
            store: CategoryStore::from_definitions(definitions, bucket_count),
            bucket_count,
        }
    }

    /// Process a batch of records, updating all category stats.
    ///
    /// The time range is derived from the min and max timestamps across all records
    /// to compute density histogram bucket indices.
    pub fn process_records<R: AsRef<LogRecord>>(&mut self, records: &[R]) {
        if records.is_empty() || self.store.categories.is_empty() {
            return;
        }

        // Compute time range for density bucketing
        let (time_min, time_max) = Self::time_range(records);
        let range_ms = (time_max - time_min).num_milliseconds().max(1) as f64;
        let bucket_count = self.bucket_count;

        for record in records {
            let record = record.as_ref();
            for cat in &mut self.store.categories {
                if eval::eval(&cat.definition.filter, record) {
                    let bucket =
                        Self::compute_bucket(record.timestamp, time_min, range_ms, bucket_count);
                    cat.record_match(Some(bucket));
                }
            }
        }

        tracing::debug!(
            categories = self.store.categories.len(),
            records = records.len(),
            "Categorization complete"
        );
    }

    /// Process a single record (for streaming/tailing).
    /// Requires pre-computed time range; caller should provide `time_min` and `range_ms`.
    pub fn process_record(&mut self, record: &LogRecord, time_min: DateTime<Utc>, range_ms: f64) {
        let bucket_count = self.bucket_count;
        for cat in &mut self.store.categories {
            if eval::eval(&cat.definition.filter, record) {
                let bucket =
                    Self::compute_bucket(record.timestamp, time_min, range_ms, bucket_count);
                cat.record_match(Some(bucket));
            }
        }
    }

    /// Resize all density histograms.
    pub fn resize_density(&mut self, new_len: usize) {
        self.bucket_count = new_len;
        for cat in &mut self.store.categories {
            cat.resize_density(new_len);
        }
    }

    /// Reset all stats (e.g., on reload).
    pub fn reset(&mut self) {
        self.store.reset();
    }

    fn time_range<R: AsRef<LogRecord>>(records: &[R]) -> (DateTime<Utc>, DateTime<Utc>) {
        let first = records[0].as_ref().timestamp;
        let mut min = first;
        let mut max = first;
        for r in records.iter().skip(1) {
            let ts = r.as_ref().timestamp;
            if ts < min {
                min = ts;
            }
            if ts > max {
                max = ts;
            }
        }
        (min, max)
    }

    fn compute_bucket(
        ts: DateTime<Utc>,
        time_min: DateTime<Utc>,
        range_ms: f64,
        bucket_count: usize,
    ) -> usize {
        let offset = (ts - time_min).num_milliseconds().max(0) as f64;
        let idx = (offset / range_ms * (bucket_count as f64 - 1.0)) as usize;
        idx.min(bucket_count.saturating_sub(1))
    }
}
