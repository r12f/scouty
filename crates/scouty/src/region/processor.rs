//! Region processor — evaluates log records against region definitions.

#[cfg(test)]
#[path = "processor_tests.rs"]
mod processor_tests;

use super::config::{CompiledMatchPoint, RegionDefinition};
use super::Region;
use crate::filter::eval;
use crate::record::LogRecord;
use std::collections::HashMap;

/// A pending start point awaiting a matching end.
#[derive(Debug, Clone)]
struct PendingStart {
    /// Index of the start record in the LogStore.
    record_index: usize,
    /// Timestamp of the start record.
    timestamp: chrono::DateTime<chrono::Utc>,
    /// Extracted metadata from the start record.
    metadata: HashMap<String, String>,
    /// Rendered reason from the matched start point.
    reason: Option<String>,
}

/// Processes log records against region definitions to detect regions.
pub struct RegionProcessor {
    definitions: Vec<RegionDefinition>,
    /// Pending starts per definition index.
    pending: Vec<Vec<PendingStart>>,
    /// Detected regions.
    regions: Vec<Region>,
    /// Next record index to process (for incremental processing).
    next_index: usize,
}

impl RegionProcessor {
    /// Create a new processor with the given region definitions.
    pub fn new(definitions: Vec<RegionDefinition>) -> Self {
        let pending = vec![Vec::new(); definitions.len()];
        Self {
            definitions,
            pending,
            regions: Vec::new(),
            next_index: 0,
        }
    }

    /// Process a batch of records from the store.
    /// Records are processed in order; call repeatedly for incremental processing.
    /// Returns the records that were tagged with region metadata (indices).
    pub fn process_records(&mut self, records: &[LogRecord]) {
        for (i, record) in records.iter().enumerate() {
            let absolute_index = self.next_index + i;

            for def_idx in 0..self.definitions.len() {
                // Check END points first
                if let Some((end_meta, end_reason)) =
                    try_match_points(&self.definitions[def_idx].end_points, record)
                {
                    // Try correlation with pending starts
                    if let Some(region) =
                        self.try_correlate(def_idx, absolute_index, end_meta, end_reason, record)
                    {
                        self.regions.push(region);
                    }
                }

                // Check START points
                if let Some((start_meta, start_reason)) =
                    try_match_points(&self.definitions[def_idx].start_points, record)
                {
                    // Create timed-out regions for expired pending starts
                    if let Some(timeout) = self.definitions[def_idx].timeout {
                        let ts = records[i].timestamp;
                        let timeout_chrono =
                            chrono::Duration::from_std(timeout).unwrap_or(chrono::Duration::MAX);
                        let (expired, remaining): (Vec<_>, Vec<_>) = self.pending[def_idx]
                            .drain(..)
                            .partition(|p| ts.signed_duration_since(p.timestamp) >= timeout_chrono);
                        self.pending[def_idx] = remaining;

                        // Create timed-out regions for expired starts
                        let def = &self.definitions[def_idx];
                        for pending in expired {
                            let timeout_reason = def
                                .timeout_reason
                                .as_ref()
                                .map(|t| super::config::render_template(t, &pending.metadata));

                            let mut metadata = pending.metadata.clone();
                            if let Some(tr) = &timeout_reason {
                                metadata.insert("end_reason".to_string(), tr.clone());
                            }
                            if let Some(sr) = &pending.reason {
                                let rendered =
                                    super::config::render_template(sr, &pending.metadata);
                                metadata.insert("start_reason".to_string(), rendered.clone());
                            }

                            let name =
                                super::config::render_template(&def.name_template, &metadata);
                            let description = def
                                .description_template
                                .as_ref()
                                .map(|t| super::config::render_template(t, &metadata));

                            self.regions.push(Region {
                                definition_name: def.name.clone(),
                                name,
                                description,
                                start_reason: pending
                                    .reason
                                    .as_ref()
                                    .map(|r| super::config::render_template(r, &pending.metadata)),
                                end_reason: timeout_reason,
                                start_index: pending.record_index,
                                end_index: pending.record_index, // timed-out: end = start
                                metadata,
                                timed_out: true,
                            });
                        }
                    }

                    self.pending[def_idx].push(PendingStart {
                        record_index: absolute_index,
                        timestamp: records[i].timestamp,
                        metadata: start_meta,
                        reason: start_reason,
                    });
                }
            }
        }

        self.next_index += records.len();
    }

    /// Try to correlate an end match with a pending start.
    fn try_correlate(
        &mut self,
        def_idx: usize,
        end_index: usize,
        end_meta: HashMap<String, String>,
        end_reason: Option<String>,
        _end_record: &LogRecord,
    ) -> Option<Region> {
        let def = &self.definitions[def_idx];
        let pendings = &mut self.pending[def_idx];

        // Walk backwards (LIFO) through pending starts
        let mut found_idx = None;
        for (i, pending) in pendings.iter().enumerate().rev() {
            // Check timeout
            if let Some(timeout) = def.timeout {
                let elapsed = _end_record
                    .timestamp
                    .signed_duration_since(pending.timestamp);
                if elapsed >= chrono::Duration::from_std(timeout).unwrap_or(chrono::Duration::MAX) {
                    continue;
                }
            }

            if def.correlate.is_empty() {
                // No correlation fields → nearest pending (LIFO)
                found_idx = Some(i);
                break;
            }

            // Check all correlate fields match
            let all_match = def.correlate.iter().all(|field| {
                let start_val = pending.metadata.get(field);
                let end_val = end_meta.get(field);
                match (start_val, end_val) {
                    (Some(s), Some(e)) => s == e,
                    _ => false,
                }
            });

            if all_match {
                found_idx = Some(i);
                break;
            }
        }

        let found_idx = found_idx?;
        let pending = pendings.remove(found_idx);

        // Merge metadata (start + end, end overwrites on conflict)
        let mut metadata = pending.metadata.clone();
        metadata.extend(end_meta);

        // Add start_reason and end_reason to metadata for template rendering
        let start_reason = pending
            .reason
            .as_ref()
            .map(|r| super::config::render_template(r, &pending.metadata));
        let end_reason_rendered = end_reason
            .as_ref()
            .map(|r| super::config::render_template(r, &metadata));

        if let Some(sr) = &start_reason {
            metadata.insert("start_reason".to_string(), sr.clone());
        }
        if let Some(er) = &end_reason_rendered {
            metadata.insert("end_reason".to_string(), er.clone());
        }

        let name = super::config::render_template(&def.name_template, &metadata);
        let description = def
            .description_template
            .as_ref()
            .map(|t| super::config::render_template(t, &metadata));

        Some(Region {
            definition_name: def.name.clone(),
            name,
            description,
            start_reason,
            end_reason: end_reason_rendered,
            start_index: pending.record_index,
            end_index,
            metadata,
            timed_out: false,
        })
    }

    /// Get all detected regions.
    pub fn regions(&self) -> &[Region] {
        &self.regions
    }

    /// Get the number of detected regions.
    pub fn region_count(&self) -> usize {
        self.regions.len()
    }

    /// Get pending start count (for diagnostics).
    pub fn pending_count(&self) -> usize {
        self.pending.iter().map(|p| p.len()).sum()
    }
}

/// Try to match a record against a list of match points.
/// Returns extracted metadata and rendered reason if any point matches.
fn try_match_points(
    points: &[CompiledMatchPoint],
    record: &LogRecord,
) -> Option<(HashMap<String, String>, Option<String>)> {
    for point in points {
        if eval::eval(&point.filter, record) {
            let mut metadata = HashMap::new();

            // Extract metadata via regex
            if let Some(re) = &point.regex {
                if let Some(caps) = re.captures(&record.message) {
                    for name in re.capture_names().flatten() {
                        if let Some(m) = caps.name(name) {
                            metadata.insert(name.to_string(), m.as_str().to_string());
                        }
                    }
                }
            }

            return Some((metadata, point.reason.clone()));
        }
    }
    None
}

// tag_record removed: region metadata now lives in RegionStore, not on LogRecord.
