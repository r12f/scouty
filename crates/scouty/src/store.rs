//! Log storage using segmented sorted arrays for high-performance insert and query.
//!
//! Architecture:
//! - Multiple fixed-capacity segments, each containing a sorted Vec<LogRecord>
//! - One "active" segment receives live inserts
//! - When active segment reaches capacity, it freezes and a new active segment is created
//! - Frozen segments are immutable for cache-friendly sequential access
//! - Out-of-order records go to a separate OOO buffer instead of mutating frozen segments
//! - OOO buffer auto-compacts when it reaches threshold (segment_capacity / 4)

use crate::record::LogRecord;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tracing::instrument;

/// Default segment capacity (number of records per segment).
const DEFAULT_SEGMENT_CAPACITY: usize = 64 * 1024; // 64K

/// Compute optimal segment capacity based on total record count.
///
/// - < 100K records: 16K segments (reduce memory waste for small datasets)
/// - 100K - 1M records: 64K segments (balanced default)
/// - > 1M records: 128K segments (reduce segment count overhead)
/// - > 10M records: 256K segments (minimize traversal overhead)
fn optimal_segment_capacity(total_records: usize) -> usize {
    match total_records {
        0..100_000 => 16 * 1024,             // 16K
        100_000..1_000_000 => 64 * 1024,     // 64K
        1_000_000..10_000_000 => 128 * 1024, // 128K
        _ => 256 * 1024,                     // 256K
    }
}

/// A single segment of log records, sorted by timestamp.
#[derive(Debug)]
struct Segment {
    records: Vec<Arc<LogRecord>>,
    frozen: bool,
}

impl Segment {
    fn new(capacity: usize) -> Self {
        Self {
            records: Vec::with_capacity(capacity),
            frozen: false,
        }
    }

    fn from_sorted(records: Vec<Arc<LogRecord>>) -> Self {
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
    fn insert(&mut self, record: Arc<LogRecord>) {
        let pos = self
            .records
            .partition_point(|r| r.timestamp <= record.timestamp);
        self.records.insert(pos, record);
    }

    /// Append a record at the end (fast path for monotonic timestamps).
    fn push(&mut self, record: Arc<LogRecord>) {
        self.records.push(record);
    }

    fn freeze(&mut self) {
        self.frozen = true;
    }
}

/// Zero-copy iterator over a range of records spanning multiple segments.
pub struct SegmentRangeIter<'a> {
    /// References to segment slices we need to iterate over.
    slices: Vec<&'a [Arc<LogRecord>]>,
    /// Current slice index.
    slice_idx: usize,
    /// Current position within the current slice.
    pos: usize,
}

impl<'a> SegmentRangeIter<'a> {
    fn new(slices: Vec<&'a [Arc<LogRecord>]>) -> Self {
        Self {
            slices,
            slice_idx: 0,
            pos: 0,
        }
    }
}

impl<'a> Iterator for SegmentRangeIter<'a> {
    type Item = &'a LogRecord;

    fn next(&mut self) -> Option<Self::Item> {
        while self.slice_idx < self.slices.len() {
            let slice = self.slices[self.slice_idx];
            if self.pos < slice.len() {
                let item = &*slice[self.pos];
                self.pos += 1;
                return Some(item);
            }
            self.slice_idx += 1;
            self.pos = 0;
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining: usize = self.slices[self.slice_idx..]
            .iter()
            .map(|s| s.len())
            .sum::<usize>()
            - if self.slice_idx < self.slices.len() {
                self.pos
            } else {
                0
            };
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for SegmentRangeIter<'a> {}

/// Configuration for LogStore segment capacity.
#[derive(Debug, Clone)]
pub struct LogStoreConfig {
    /// If set, use this fixed segment capacity (overrides auto-tuning).
    pub segment_capacity: Option<usize>,
    /// Enable dynamic segment capacity auto-tuning based on total record count.
    /// Default: true. Ignored if `segment_capacity` is set.
    pub auto_tune: bool,
}

impl Default for LogStoreConfig {
    fn default() -> Self {
        Self {
            segment_capacity: None,
            auto_tune: true,
        }
    }
}

/// Stores log records in timestamp-sorted order using segmented arrays.
///
/// Provides O(1) append for monotonic live inserts. Out-of-order records
/// go to a separate OOO buffer that auto-compacts at threshold.
/// Batch inserts create frozen segments directly for efficiency.
#[derive(Debug)]
pub struct LogStore {
    /// Frozen segments (sorted by time range, immutable).
    frozen: Vec<Segment>,
    /// Active segment receiving live inserts.
    active: Segment,
    /// Out-of-order buffer: records that arrived before the latest frozen timestamp.
    ooo_buffer: Vec<Arc<LogRecord>>,
    /// Segment capacity threshold.
    segment_capacity: usize,
    /// Cached total record count (frozen + active, excluding OOO).
    main_len: usize,
    /// Whether auto-tuning is enabled.
    auto_tune: bool,
    /// Whether segment capacity was explicitly set by user.
    user_override: bool,
}

impl LogStore {
    pub fn new() -> Self {
        Self {
            frozen: Vec::new(),
            active: Segment::new(DEFAULT_SEGMENT_CAPACITY),
            ooo_buffer: Vec::new(),
            segment_capacity: DEFAULT_SEGMENT_CAPACITY,
            main_len: 0,
            auto_tune: true,
            user_override: false,
        }
    }

    /// Create a store with explicit configuration.
    pub fn with_config(config: LogStoreConfig) -> Self {
        let capacity = config.segment_capacity.unwrap_or(DEFAULT_SEGMENT_CAPACITY);
        Self {
            frozen: Vec::new(),
            active: Segment::new(capacity),
            ooo_buffer: Vec::new(),
            segment_capacity: capacity,
            main_len: 0,
            auto_tune: config.auto_tune && config.segment_capacity.is_none(),
            user_override: config.segment_capacity.is_some(),
        }
    }

    /// Create a store with pre-allocated capacity hint.
    pub fn with_capacity(_capacity: usize) -> Self {
        Self::new()
    }

    /// Insert a single record, maintaining timestamp order.
    ///
    /// Fast path: if timestamp >= last record's timestamp, append to active segment (O(1)).
    /// OOO path: record goes to OOO buffer if it belongs before frozen segments.
    pub fn insert(&mut self, record: LogRecord) {
        let record = Arc::new(record);
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
                self.main_len += 1;
                self.maybe_freeze_active();
                return;
            }
        }

        // Check if it belongs in the active segment's time range
        if self.active.records.is_empty()
            || self
                .active
                .min_timestamp()
                .is_none_or(|t| record.timestamp >= t)
        {
            self.active.insert(record);
            self.main_len += 1;
            self.maybe_freeze_active();
            return;
        }

        // Out-of-order: goes to OOO buffer (never mutate frozen segments)
        self.ooo_buffer.push(record);
        self.maybe_compact_ooo();
    }

    /// Bulk insert records efficiently.
    ///
    /// For empty stores: sorts batch and creates frozen segments directly.
    /// For non-empty stores: uses merge-based approach — only affected segments are rebuilt.
    #[instrument(skip(self, batch), fields(batch_size = batch.len()))]
    pub fn insert_batch(&mut self, mut batch: Vec<LogRecord>) {
        if batch.is_empty() {
            return;
        }

        batch.sort_by_key(|r| r.timestamp);
        let batch: Vec<Arc<LogRecord>> = batch.into_iter().map(Arc::new).collect();

        if self.main_len == 0 && self.active.is_empty() {
            self.insert_batch_empty(batch);
        } else {
            self.insert_batch_merge(batch);
        }
        self.main_len = self.frozen.iter().map(|s| s.len()).sum::<usize>() + self.active.len();
        self.maybe_auto_tune();
    }

    /// Fast path for inserting into an empty store.
    fn insert_batch_empty(&mut self, batch: Vec<Arc<LogRecord>>) {
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
    fn insert_batch_merge(&mut self, batch: Vec<Arc<LogRecord>>) {
        let batch_min = batch.first().unwrap().timestamp;
        let batch_max = batch.last().unwrap().timestamp;

        // Fast path: batch is entirely after all existing records — just append
        let store_max = self
            .active
            .max_timestamp()
            .or_else(|| self.frozen.last().and_then(|s| s.max_timestamp()));

        if store_max.is_none_or(|max| batch_min >= max) {
            let mut combined = Vec::with_capacity(self.active.len() + batch.len());
            combined.append(&mut self.active.records);
            let merged = Self::merge_sorted(combined, batch);
            self.active = Segment::new(self.segment_capacity);
            self.append_records_as_segments(merged);
            return;
        }

        // General case: find affected frozen segment range
        let first_affected = self
            .frozen
            .partition_point(|s| s.max_timestamp().is_some_and(|max| max < batch_min));
        let last_affected = self
            .frozen
            .partition_point(|s| s.min_timestamp().is_some_and(|min| min <= batch_max));

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

        for seg in self.frozen.drain(first_affected..last_affected) {
            merged_records = Self::merge_sorted(merged_records, seg.records);
        }

        if active_overlaps {
            let active_records = std::mem::replace(
                &mut self.active.records,
                Vec::with_capacity(self.segment_capacity),
            );
            merged_records = Self::merge_sorted(merged_records, active_records);
        }

        merged_records = Self::merge_sorted(merged_records, batch);

        let mut new_segments = Vec::new();
        for chunk in merged_records.chunks(self.segment_capacity) {
            new_segments.push(Segment::from_sorted(chunk.to_vec()));
        }

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

        let insert_pos = first_affected;
        for (i, seg) in new_segments.into_iter().enumerate() {
            self.frozen.insert(insert_pos + i, seg);
        }
    }

    /// Merge two sorted Vec<LogRecord> into one sorted Vec.
    fn merge_sorted(a: Vec<Arc<LogRecord>>, b: Vec<Arc<LogRecord>>) -> Vec<Arc<LogRecord>> {
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
    fn append_records_as_segments(&mut self, records: Vec<Arc<LogRecord>>) {
        for chunk in records.chunks(self.segment_capacity) {
            self.frozen.push(Segment::from_sorted(chunk.to_vec()));
        }
        if let Some(last) = self.frozen.last() {
            if last.len() < self.segment_capacity {
                let mut seg = self.frozen.pop().unwrap();
                seg.frozen = false;
                self.active = seg;
            }
        }
    }

    /// Get all records as a collected Vec (sorted by timestamp).
    /// Includes OOO buffer records merged in correct order.
    ///
    /// Note: This allocates a new Vec. For large stores, prefer `iter()` or `range()`.
    pub fn records(&self) -> Vec<LogRecord> {
        let main: Vec<LogRecord> = self
            .frozen
            .iter()
            .flat_map(|s| s.records.iter())
            .chain(self.active.records.iter())
            .map(|r| (**r).clone())
            .collect();

        if self.ooo_buffer.is_empty() {
            return main;
        }

        let mut sorted_ooo: Vec<LogRecord> =
            self.ooo_buffer.iter().map(|r| (**r).clone()).collect();
        sorted_ooo.sort_by_key(|r| r.timestamp);
        Self::merge_sorted(
            main.into_iter().map(Arc::new).collect(),
            sorted_ooo.into_iter().map(Arc::new).collect(),
        )
        .into_iter()
        .map(|r| Arc::try_unwrap(r).unwrap_or_else(|a| (*a).clone()))
        .collect()
    }

    /// Iterate over all main records in timestamp order without allocation.
    /// Does NOT include OOO buffer records. Call `compact_ooo()` first if needed.
    pub fn iter(&self) -> impl Iterator<Item = &LogRecord> {
        self.frozen
            .iter()
            .flat_map(|s| s.records.iter())
            .chain(self.active.records.iter())
            .map(|r| r.as_ref())
    }

    /// Iterate over all main records as Arc references (zero-copy sharing).
    /// Use this for async operations to avoid cloning.
    pub fn iter_arc(&self) -> impl Iterator<Item = &Arc<LogRecord>> {
        self.frozen
            .iter()
            .flat_map(|s| s.records.iter())
            .chain(self.active.records.iter())
    }

    /// Iterate over all records including OOO buffer, in timestamp order.
    /// Returns owned records since merging requires sorting the OOO buffer.
    pub fn iter_all(&self) -> Vec<LogRecord> {
        self.records()
    }

    /// Get a record by global index (includes OOO buffer records).
    pub fn get(&self, index: usize) -> Option<&LogRecord> {
        // Fast path: no OOO records, direct index into main segments
        if self.ooo_buffer.is_empty() {
            return self.get_main(index);
        }
        // With OOO records, we can't efficiently index without sorting.
        // Return from main segments if index < main_len.
        if index < self.main_len {
            return self.get_main(index);
        }
        // Index into OOO buffer (unsorted, but accessible)
        self.ooo_buffer
            .get(index - self.main_len)
            .map(|r| r.as_ref())
    }

    /// Get a record from main segments (frozen + active) by global index.
    fn get_main(&self, index: usize) -> Option<&LogRecord> {
        if index >= self.main_len {
            return None;
        }
        let mut offset = 0;
        for seg in &self.frozen {
            if index < offset + seg.len() {
                return Some(&seg.records[index - offset]);
            }
            offset += seg.len();
        }
        let local_idx = index - offset;
        self.active.records.get(local_idx).map(|r| r.as_ref())
    }

    /// Number of stored records (main + OOO buffer).
    pub fn len(&self) -> usize {
        self.main_len + self.ooo_buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Number of records in the OOO buffer.
    pub fn ooo_len(&self) -> usize {
        self.ooo_buffer.len()
    }

    /// Find the global index of the first record at or after the given timestamp.
    /// Searches main segments only. Call `compact_ooo()` first for full accuracy.
    pub fn find_by_timestamp(&self, ts: &DateTime<Utc>) -> usize {
        let mut global_offset = 0;
        for seg in &self.frozen {
            if seg.max_timestamp().is_some_and(|max| max < *ts) {
                global_offset += seg.len();
                continue;
            }
            let local_pos = seg.records.partition_point(|r| r.timestamp < *ts);
            return global_offset + local_pos;
        }
        let local_pos = self.active.records.partition_point(|r| r.timestamp < *ts);
        global_offset + local_pos
    }

    /// Get records in the given global index range as a zero-copy iterator.
    ///
    /// Iterates across segment boundaries without heap allocation.
    /// Does NOT include OOO buffer records.
    pub fn range(&self, start: usize, end: usize) -> SegmentRangeIter<'_> {
        let end = end.min(self.main_len);
        let start = start.min(end);

        if start == end {
            return SegmentRangeIter::new(vec![]);
        }

        let mut slices = Vec::new();
        let mut global_offset = 0;

        for seg in &self.frozen {
            let seg_end = global_offset + seg.len();
            if start < seg_end && end > global_offset {
                let local_start = start.saturating_sub(global_offset);
                let local_end = (end - global_offset).min(seg.len());
                slices.push(&seg.records[local_start..local_end]);
            }
            global_offset = seg_end;
            if global_offset >= end {
                return SegmentRangeIter::new(slices);
            }
        }

        // Active segment
        let seg_end = global_offset + self.active.len();
        if start < seg_end && end > global_offset {
            let local_start = start.saturating_sub(global_offset);
            let local_end = (end - global_offset).min(self.active.len());
            slices.push(&self.active.records[local_start..local_end]);
        }

        SegmentRangeIter::new(slices)
    }

    /// Get records in the given global index range as a collected Vec.
    pub fn range_collected(&self, start: usize, end: usize) -> Vec<LogRecord> {
        self.range(start, end).cloned().collect()
    }

    /// Compact the OOO buffer: sort, group by timestamp range, merge into frozen segments.
    pub fn compact_ooo(&mut self) {
        if self.ooo_buffer.is_empty() {
            return;
        }

        let mut ooo = std::mem::take(&mut self.ooo_buffer);
        ooo.sort_by_key(|r| r.timestamp);

        // Group OOO records by which frozen segment they belong to
        let mut seg_groups: Vec<(usize, Vec<Arc<LogRecord>>)> = Vec::new();

        for record in ooo {
            let seg_idx = self.find_segment_for_timestamp(&record.timestamp);
            if let Some(last) = seg_groups.last_mut() {
                if last.0 == seg_idx {
                    last.1.push(record);
                    continue;
                }
            }
            seg_groups.push((seg_idx, vec![record]));
        }

        // Merge each group into its target segment (in reverse to preserve indices)
        for (seg_idx, records) in seg_groups.into_iter().rev() {
            if seg_idx < self.frozen.len() {
                // Merge into frozen segment
                let existing = std::mem::take(&mut self.frozen[seg_idx].records);
                self.frozen[seg_idx].records = Self::merge_sorted(existing, records);
                self.main_len += self.frozen[seg_idx].records.len();

                // Split if oversized
                if self.frozen[seg_idx].len() > self.segment_capacity * 2 {
                    self.split_segment(seg_idx);
                }
            } else {
                // Merge into active segment
                for record in records {
                    self.active.insert(record);
                }
            }
        }

        // Recalculate main_len
        self.main_len = self.frozen.iter().map(|s| s.len()).sum::<usize>() + self.active.len();
    }

    /// Check if OOO buffer should be compacted and do so.
    fn maybe_compact_ooo(&mut self) {
        let threshold = self.segment_capacity / 4;
        if self.ooo_buffer.len() >= threshold {
            self.compact_ooo();
        }
    }

    /// Clear all records.
    pub fn clear(&mut self) {
        self.frozen.clear();
        self.ooo_buffer.clear();
        self.main_len = 0;
        if self.auto_tune && !self.user_override {
            self.segment_capacity = DEFAULT_SEGMENT_CAPACITY;
        }
        self.active = Segment::new(self.segment_capacity);
    }

    /// Number of segments (frozen + active).
    pub fn segment_count(&self) -> usize {
        self.frozen.len() + 1
    }

    /// Current segment capacity.
    pub fn segment_capacity(&self) -> usize {
        self.segment_capacity
    }

    // --- Internal helpers ---

    /// Auto-tune segment capacity based on current total record count.
    /// Only affects future segments — existing segments are not resized.
    fn maybe_auto_tune(&mut self) {
        if !self.auto_tune || self.user_override {
            return;
        }
        let optimal = optimal_segment_capacity(self.main_len);
        if optimal != self.segment_capacity {
            self.segment_capacity = optimal;
        }
    }

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
