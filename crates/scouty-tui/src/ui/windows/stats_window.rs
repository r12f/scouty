//! Statistics data computation (used by stats panel widget).

#[cfg(test)]
#[path = "stats_window_tests.rs"]
mod stats_window_tests;

use crate::app::App;
use scouty::record::LogLevel;
use std::collections::HashMap;

/// Pre-computed statistics for the overlay.
pub struct StatsData {
    pub total_records: usize,
    pub filtered_records: usize,
    pub time_first: Option<String>,
    pub time_last: Option<String>,
    pub level_counts: Vec<(LogLevel, usize)>,
    pub top_components: Vec<(String, usize)>,
}

impl StatsData {
    /// Compute statistics from the app state.
    pub fn compute(app: &App) -> Self {
        let total_records = app.total_records;
        let filtered_records = app.filtered_indices.len();

        let mut time_first: Option<chrono::DateTime<chrono::Utc>> = None;
        let mut time_last: Option<chrono::DateTime<chrono::Utc>> = None;
        let mut level_map: HashMap<LogLevel, usize> = HashMap::new();
        let mut component_map: HashMap<String, usize> = HashMap::new();

        for &idx in &app.filtered_indices {
            let record = &app.records[idx];

            // Time range
            let ts = record.timestamp;
            match time_first {
                None => time_first = Some(ts),
                Some(f) if ts < f => time_first = Some(ts),
                _ => {}
            }
            match time_last {
                None => time_last = Some(ts),
                Some(l) if ts > l => time_last = Some(ts),
                _ => {}
            }

            // Level distribution
            if let Some(level) = record.level {
                *level_map.entry(level).or_insert(0) += 1;
            }

            // Component counts
            if let Some(ref comp) = record.component_name {
                *component_map.entry(comp.clone()).or_insert(0) += 1;
            }
        }

        // Sort levels by severity order
        let mut level_counts: Vec<(LogLevel, usize)> = level_map.into_iter().collect();
        level_counts.sort_by_key(|(l, _)| *l);

        // Top 10 components by count
        let mut top_components: Vec<(String, usize)> = component_map.into_iter().collect();
        top_components.sort_by(|a, b| b.1.cmp(&a.1));
        top_components.truncate(10);

        let fmt_ts = |ts: chrono::DateTime<chrono::Utc>| -> String {
            ts.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
        };

        StatsData {
            total_records,
            filtered_records,
            time_first: time_first.map(fmt_ts),
            time_last: time_last.map(fmt_ts),
            level_counts,
            top_components,
        }
    }
}
