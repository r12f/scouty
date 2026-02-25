//! RegionStore — stores detected regions with efficient index-based queries.

#[cfg(test)]
#[path = "store_tests.rs"]
mod store_tests;

use super::Region;

/// Stores all detected regions sorted by start_index for efficient lookup.
#[derive(Debug, Clone, Default)]
pub struct RegionStore {
    /// Regions sorted by start_index.
    regions: Vec<Region>,
}

impl RegionStore {
    /// Create an empty store.
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
        }
    }

    /// Build a store from a list of regions (sorts internally).
    pub fn from_regions(mut regions: Vec<Region>) -> Self {
        regions.sort_by_key(|r| r.start_index);
        Self { regions }
    }

    /// Add a region, maintaining sorted order.
    pub fn push(&mut self, region: Region) {
        let pos = self
            .regions
            .partition_point(|r| r.start_index <= region.start_index);
        self.regions.insert(pos, region);
    }

    /// Query all regions that contain the given record index.
    /// Returns regions where `start_index <= index <= end_index`.
    pub fn regions_at(&self, index: usize) -> Vec<&Region> {
        // Binary search for the rightmost region with start_index <= index
        let upper = self.regions.partition_point(|r| r.start_index <= index);
        // Check all regions up to that point
        self.regions[..upper]
            .iter()
            .filter(|r| r.end_index >= index)
            .collect()
    }

    /// Get the innermost (shortest span) region at an index.
    pub fn innermost_at(&self, index: usize) -> Option<&Region> {
        self.regions_at(index)
            .into_iter()
            .min_by_key(|r| r.end_index - r.start_index)
    }

    /// Get all regions.
    pub fn regions(&self) -> &[Region] {
        &self.regions
    }

    /// Number of regions.
    pub fn len(&self) -> usize {
        self.regions.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }
}
