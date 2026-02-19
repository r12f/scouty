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
    /// For empty stores: sorts batch and creates frozen segments directly.
    /// For non-empty stores: uses merge-based approach — only affected segments are rebuilt.
    pub fn insert_batch(&mut self, mut batch: Vec<LogRecord>) {
        if batch.is_empty() {
            return;
        }

        batch.sort_by_key(|r| r.timestamp);

        if self.total_len == 0 && self.active.is_empty() {
            self.insert_batch_empty(batch);
        } else {
            self.insert_batch_merge(batch);
        }
        self.total_len = self.frozen.iter().map(|s| s.len()).sum::<usize>() + self.active.len();
    }

    /// Fast path for inserting into an empty store.
    fn insert_batch_empty(&mut self, batch: Vec<LogRecord>) {
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
    }

    /// Merge-based batch insert for non-empty stores.
    ///
    /// Strategy:
    /// 1. If batch timestamps are all >= max existing timestamp, just append (fast path).
    /// 2. Otherwise, find affected segment range and merge only those segments with the batch.
    ///    Unaffected frozen segments remain untouched (zero-copy).
    fn insert_batch_merge(&mut self, batch: Vec<LogRecord>) {
        let batch_min = batch.first().unwrap().timestamp;
        let batch_max = batch.last().unwrap().timestamp;

        // Fast path: batch is entirely after all existing records — just append
        let store_max = self
            .active
            .max_timestamp()
            .or_else(|| self.frozen.last().and_then(|s| s.max_timestamp()));

        if store_max.is_none_or(|max| batch_min >= max) {
            // Append: drain active into batch, re-segment
            let mut combined = Vec::with_capacity(self.active.len() + batch.len());
            combined.append(&mut self.active.records);
            // Merge active (sorted) + batch (sorted) — both sorted, so merge
            let merged = Self::merge_sorted(combined, batch);
            self.active = Segment::new(self.segment_capacity);
            self.append_records_as_segments(merged);
            return;
        }

        // General case: find affected frozen segment range
        // Segments whose time range overlaps with [batch_min, batch_max]
        let first_affected = self
            .frozen
            .partition_point(|s| s.max_timestamp().is_some_and(|max| max < batch_min));
        let last_affected = self
            .frozen
            .partition_point(|s| s.min_timestamp().is_some_and(|min| min <= batch_max));

        // Collect records from affected segments + active (if overlapping) + batch
        let active_overlaps = self
            .active
            .min_timestamp()
            .is_none_or(|min| min <= batch_max)
            || self
                .active
                .max_timestamp()
                .is_none_or(|max| max >= batch_min)
            || self.active.is_empty();

        let affected_count: usize = self.frozen[first_affected..last_affected]
            .iter()
            .map(|s| s.len())
            .sum::<usize>()
            + if active_overlaps {
                self.active.len()
            } else {
                0
            }
            + batch.len();

        let mut merged_records = Vec::with_capacity(affected_count);

        // Drain affected frozen segments
        for seg in self.frozen.drain(first_affected..last_affected) {
            merged_records = Self::merge_sorted(merged_records, seg.records);
        }

        // Include active segment if overlapping
        if active_overlaps {
            let active_records = std::mem::replace(
                &mut self.active.records,
                Vec::with_capacity(self.segment_capacity),
            );
            merged_records = Self::merge_sorted(merged_records, active_records);
        }

        // Merge with batch
        merged_records = Self::merge_sorted(merged_records, batch);

        // Create new segments from merged records and insert them at the right position
        let mut new_segments = Vec::new();
        for chunk in merged_records.chunks(self.segment_capacity) {
            new_segments.push(Segment::from_sorted(chunk.to_vec()));
        }

        // Last new segment becomes active if under capacity and active was included
        if active_overlaps {
            self.active = Segment::new(self.segment_capacity);
            if let Some(last) = new_segments.last() {
                if last.len() < self.segment_capacity {
                    let mut seg = new_segments.pop().unwrap();
                    seg.frozen = false;
                    self.active = seg;
                }
            }
        }

        // Splice new frozen segments into position
        let insert_pos = first_affected;
        for (i, seg) in new_segments.into_iter().enumerate() {
            self.frozen.insert(insert_pos + i, seg);
        }
    }

    /// Merge two sorted Vec<LogRecord> into one sorted Vec.
    fn merge_sorted(a: Vec<LogRecord>, b: Vec<LogRecord>) -> Vec<LogRecord> {
        if a.is_empty() {
            return b;
        }
        if b.is_empty() {
            return a;
        }

        let mut result = Vec::with_capacity(a.len() + b.len());
        let mut ai = a.into_iter().peekable();
        let mut bi = b.into_iter().peekable();

        loop {
            match (ai.peek(), bi.peek()) {
                (Some(a_rec), Some(b_rec)) => {
                    if a_rec.timestamp <= b_rec.timestamp {
                        result.push(ai.next().unwrap());
                    } else {
                        result.push(bi.next().unwrap());
                    }
                }
                (Some(_), None) => {
                    result.extend(ai);
                    break;
                }
                (None, Some(_)) => {
                    result.extend(bi);
                    break;
                }
                (None, None) => break,
            }
        }
        result
    }

    /// Append sorted records as frozen segments (+ possibly active).
    fn append_records_as_segments(&mut self, records: Vec<LogRecord>) {
        for chunk in records.chunks(self.segment_capacity) {
            self.frozen.push(Segment::from_sorted(chunk.to_vec()));
        }
        // Last chunk becomes active if under capacity
        if let Some(last) = self.frozen.last() {
            if last.len() < self.segment_capacity {
                let mut seg = self.frozen.pop().unwrap();
                seg.frozen = false;
                self.active = seg;
            }
        }
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
