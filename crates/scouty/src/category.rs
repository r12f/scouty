//! Log categorization — data model, config loading, and stats tracking.
//!
//! Categories are named classification rules with filter expressions.
//! Each incoming log record is evaluated against all categories;
//! matching records increment the category's count and density histogram.

#[cfg(test)]
#[path = "category_tests.rs"]
mod category_tests;

use crate::filter::expr::{self, Expr};
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
