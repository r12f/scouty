//! Log storage using segmented sorted arrays for high-performance insert and query.
//!
//! Architecture:
//! - Multiple fixed-capacity segments, each containing a sorted Vec<LogRecord>
//! - One "active" segment receives live inserts
//! - When active segment reaches capacity, it freezes and a new active segment is created
//! - Frozen segments are immutable for cache-friendly sequential access

use crate::record::LogRecord;
use chrono::{DateTime, Utc};

/// Default segment capacity (number of records per segment).
const DEFAULT_SEGMENT_CAPACITY: usize = 64 * 1024; // 64K

/// A single segment of log records, sorted by timestamp.
#[derive(Debug)]
struct Segment {
    records: Vec<LogRecord>,
    frozen: bool,
}

impl Segment {
    fn new(capacity: usize) -> Self {
        Self {
            records: Vec::with_capacity(capacity),
            frozen: false,
        }
    }

    fn from_sorted(records: Vec<LogRecord>) -> Self {
        Self {
            records,
            frozen: true,
        }
    }

    fn min_timestamp(&self) -> Option<DateTime<Utc>> {
        self.records.first().map(|r| r.timestamp)
    }

    fn max_timestamp(&self) -> Option<DateTime<Utc>> {
        self.records.last().map(|r| r.timestamp)
    }

    fn len(&self) -> usize {
        self.records.len()
    }

    fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Insert a record maintaining sort order within this segment.
    fn insert(&mut self, record: LogRecord) {
        let pos = self
            .records
            .partition_point(|r| r.timestamp <= record.timestamp);
        self.records.insert(pos, record);
    }

    /// Append a record at the end (fast path for monotonic timestamps).
    fn push(&mut self, record: LogRecord) {
        self.records.push(record);
    }

    fn freeze(&mut self) {
        self.frozen = true;
    }
}

/// Stores log records in timestamp-sorted order using segmented arrays.
///
/// Provides O(1) append for monotonic live inserts and O(log s + s) for
/// out-of-order inserts (where s = segment capacity, typically 64K).
/// Batch inserts create frozen segments directly for efficiency.
#[derive(Debug)]
pub struct LogStore {
    /// Frozen segments (sorted by time range, immutable).
    frozen: Vec<Segment>,
    /// Active segment receiving live inserts.
    active: Segment,
    /// Segment capacity threshold.
    segment_capacity: usize,
    /// Cached total record count.
    total_len: usize,
}

impl LogStore {
    pub fn new() -> Self {
        Self {
            frozen: Vec::new(),
            active: Segment::new(DEFAULT_SEGMENT_CAPACITY),
            segment_capacity: DEFAULT_SEGMENT_CAPACITY,
            total_len: 0,
        }
    }

    /// Create a store with pre-allocated capacity hint.
    pub fn with_capacity(_capacity: usize) -> Self {
        // Capacity hint used for initial segment sizing
        Self::new()
    }

    /// Insert a single record, maintaining timestamp order.
    ///
    /// Fast path: if timestamp >= last record's timestamp, append to active segment (O(1)).
    /// Slow path: out-of-order insert into appropriate segment (O(log s + s)).
    pub fn insert(&mut self, record: LogRecord) {
        // Fast path: monotonic timestamp — append to active segment
        if self.active.records.is_empty()
            || record.timestamp >= self.active.records.last().unwrap().timestamp
        {
            // Check if we also need to verify against frozen segments
            if self.frozen.is_empty()
                || self
                    .frozen
                    .last()
                    .and_then(|s| s.max_timestamp())
                    .is_none_or(|t| record.timestamp >= t)
            {
                self.active.push(record);
                self.total_len += 1;
                self.maybe_freeze_active();
                return;
            }
        }

        // Slow path: out-of-order — find the right segment
        // Check if it belongs in the active segment's time range
        if self.active.records.is_empty()
            || self
                .active
                .min_timestamp()
                .is_none_or(|t| record.timestamp >= t)
        {
            self.active.insert(record);
            self.total_len += 1;
            self.maybe_freeze_active();
            return;
        }

        // Find the frozen segment where this record belongs
        let seg_idx = self.find_segment_for_timestamp(&record.timestamp);
        if seg_idx < self.frozen.len() {
            // Insert into frozen segment (temporarily unfreezing)
            self.frozen[seg_idx].insert(record);
            self.total_len += 1;
            // If segment grew too large, split it
            if self.frozen[seg_idx].len() > self.segment_capacity * 2 {
                self.split_segment(seg_idx);
            }
        } else {
            // Belongs in active segment
            self.active.insert(record);
            self.total_len += 1;
            self.maybe_freeze_active();
        }
    }

    /// Bulk insert records efficiently.
    ///
    /// Sorts the batch, then creates frozen segments directly for large batches.
    pub fn insert_batch(&mut self, mut batch: Vec<LogRecord>) {
        if batch.is_empty() {
            return;
        }

        batch.sort_by_key(|r| r.timestamp);
        let count = batch.len();

        if self.total_len == 0 && self.active.is_empty() {
            // Empty store: create frozen segments directly from sorted batch
            for chunk in batch.chunks(self.segment_capacity) {
                let seg = Segment::from_sorted(chunk.to_vec());
                self.frozen.push(seg);
            }
            // Last chunk becomes active if under capacity
            if let Some(last) = self.frozen.last() {
                if last.len() < self.segment_capacity {
                    let mut seg = self.frozen.pop().unwrap();
                    seg.frozen = false;
                    self.active = seg;
                }
            }
        } else {
            // Non-empty store: merge batch into existing segments
            // Simple approach: insert remaining active records + batch, rebuild
            let mut all_records: Vec<LogRecord> = Vec::with_capacity(self.total_len + count);
            for seg in self.frozen.drain(..) {
                all_records.extend(seg.records);
            }
            all_records.append(&mut self.active.records);
            all_records.extend(batch);
            all_records.sort_by_key(|r| r.timestamp);

            self.frozen.clear();
            self.active = Segment::new(self.segment_capacity);

            for chunk in all_records.chunks(self.segment_capacity) {
                let seg = Segment::from_sorted(chunk.to_vec());
                self.frozen.push(seg);
            }
            if let Some(last) = self.frozen.last() {
                if last.len() < self.segment_capacity {
                    let mut seg = self.frozen.pop().unwrap();
                    seg.frozen = false;
                    self.active = seg;
                }
            }
        }
        self.total_len = self
            .frozen
            .iter()
            .map(|s| s.len())
            .sum::<usize>()
            + self.active.len();
    }

    /// Get all records as a collected Vec (sorted by timestamp).
    ///
    /// Note: This allocates a new Vec. For large stores, prefer `iter()` or `range()`.
    pub fn records(&self) -> Vec<LogRecord> {
        let mut result = Vec::with_capacity(self.total_len);
        for seg in &self.frozen {
            result.extend(seg.records.iter().cloned());
        }
        result.extend(self.active.records.iter().cloned());
        result
    }

    /// Iterate over all records in timestamp order without allocation.
    pub fn iter(&self) -> impl Iterator<Item = &LogRecord> {
        self.frozen
            .iter()
            .flat_map(|s| s.records.iter())
            .chain(self.active.records.iter())
    }

    /// Get a record by global index.
    pub fn get(&self, index: usize) -> Option<&LogRecord> {
        if index >= self.total_len {
            return None;
        }
        let mut offset = 0;
        for seg in &self.frozen {
            if index < offset + seg.len() {
                return Some(&seg.records[index - offset]);
            }
            offset += seg.len();
        }
        // Must be in active segment
        let local_idx = index - offset;
        self.active.records.get(local_idx)
    }

    /// Number of stored records.
    pub fn len(&self) -> usize {
        self.total_len
    }

    pub fn is_empty(&self) -> bool {
        self.total_len == 0
    }

    /// Find the global index of the first record at or after the given timestamp.
    pub fn find_by_timestamp(&self, ts: &DateTime<Utc>) -> usize {
        let mut global_offset = 0;
        for seg in &self.frozen {
            if seg.max_timestamp().is_some_and(|max| max < *ts) {
                global_offset += seg.len();
                continue;
            }
            // Target is in this segment
            let local_pos = seg.records.partition_point(|r| r.timestamp < *ts);
            return global_offset + local_pos;
        }
        // Check active segment
        let local_pos = self.active.records.partition_point(|r| r.timestamp < *ts);
        global_offset + local_pos
    }

    /// Get records in the given global index range.
    ///
    /// Returns a Vec since the range may span multiple segments.
    pub fn range(&self, start: usize, end: usize) -> Vec<LogRecord> {
        let end = end.min(self.total_len);
        let start = start.min(end);
        if start == end {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(end - start);
        let mut global_offset = 0;

        for seg in &self.frozen {
            let seg_end = global_offset + seg.len();
            if start < seg_end && end > global_offset {
                let local_start = start.saturating_sub(global_offset);
                let local_end = (end - global_offset).min(seg.len());
                result.extend(seg.records[local_start..local_end].iter().cloned());
            }
            global_offset = seg_end;
            if global_offset >= end {
                return result;
            }
        }

        // Active segment
        let seg_end = global_offset + self.active.len();
        if start < seg_end && end > global_offset {
            let local_start = start.saturating_sub(global_offset);
            let local_end = (end - global_offset).min(self.active.len());
            result.extend(self.active.records[local_start..local_end].iter().cloned());
        }

        result
    }

    /// Clear all records.
    pub fn clear(&mut self) {
        self.frozen.clear();
        self.active = Segment::new(self.segment_capacity);
        self.total_len = 0;
    }

    /// Number of segments (frozen + active).
    pub fn segment_count(&self) -> usize {
        self.frozen.len() + 1
    }

    // --- Internal helpers ---

    /// Freeze active segment and create a new one if capacity reached.
    fn maybe_freeze_active(&mut self) {
        if self.active.len() >= self.segment_capacity {
            self.active.freeze();
            let old = std::mem::replace(&mut self.active, Segment::new(self.segment_capacity));
            self.frozen.push(old);
        }
    }

    /// Find which frozen segment a timestamp belongs to.
    fn find_segment_for_timestamp(&self, ts: &DateTime<Utc>) -> usize {
        self.frozen
            .partition_point(|s| s.max_timestamp().is_some_and(|max| max < *ts))
    }

    /// Split an oversized segment into two.
    fn split_segment(&mut self, idx: usize) {
        let seg = &mut self.frozen[idx];
        let mid = seg.len() / 2;
        let second_half = seg.records.split_off(mid);
        let new_seg = Segment::from_sorted(second_half);
        self.frozen.insert(idx + 1, new_seg);
    }
}

impl Default for LogStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod store_tests;
