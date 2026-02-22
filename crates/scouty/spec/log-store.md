# Log Store

## Overview

LogStore is the central storage for all parsed `LogRecord`s, maintaining timestamp-sorted order and supporting both batch and live insertion. It uses a **Segmented Sorted Array** architecture for high performance.

## Current Status

✅ Implemented

## Design

### Architecture: Segmented Sorted Array

```
LogStore
├── Segment Index
├── Segment 0: [record_0 .. record_N]     (time range: T0 ~ T1)
├── Segment 1: [record_N+1 .. record_M]   (time range: T1 ~ T2)
├── ...
└── Active Segment (receives live inserts)
```

```rust
struct LogStore {
    segments: Vec<Segment>,           // Frozen segments, time-ordered
    active_segment: Segment,          // Current segment receiving inserts
    segment_capacity: usize,          // Default 64K~128K records
    total_count: usize,
}

struct Segment {
    records: Vec<LogRecord>,
    min_timestamp: DateTime<Utc>,
    max_timestamp: DateTime<Utc>,
}
```

### Insertion Strategy

**Live single insert:**
1. Most live logs are timestamp-increasing → append to active segment tail, O(1)
2. Out-of-order logs → binary search within segment, O(log s) find + O(s) shift (s = segment size, much smaller than total n)
3. If timestamp falls in frozen segment range → locate and insert into that segment
4. Active segment at capacity → freeze and create new active segment

**Batch insert (merge optimization):**
- Sorted batch merges with existing segments — only affected segments are rebuilt
- Unaffected frozen segments remain untouched (zero-copy)
- Complexity: O(N) instead of O(N log N) full rebuild

### Query Strategy

| Operation | Complexity |
|-----------|-----------|
| Get by global index | O(log k), k = segment count |
| Time range query | O(log n) via segment min/max skip |
| Sequential traversal | O(1) amortized, cache-friendly within segments |

### API

- `get(index)` — by global index
- `range(start, end)` — returns zero-copy `SegmentRangeIter`
- `find_by_timestamp()` — binary search
- `iter()` — `impl Iterator<Item = &LogRecord>`
- `len()` — total count

Key decisions:
- **`records()` returning `&[LogRecord]` is deprecated** — segmented storage cannot return contiguous slice; all consumers migrated to iterators.
- **Out-of-Order Buffer** — out-of-order records go to a separate buffer, compacted in batches to preserve frozen segment immutability.
- **Why not BTreeMap** — same timestamp may have multiple records; no efficient global index; cache-unfriendly for TUI traversal.

## Performance Benchmarks

| Metric | Target | Achieved |
|--------|--------|----------|
| 1M live single inserts | — | 282ms |
| 1M sequential traversal | — | 5ms |
| Non-empty store (1M) batch insert 10K | < 10ms | — |
| Batch insert 1M records | < 1s | — |

## Change Log

| Date | Change |
|------|--------|
| 2026-02-19 | Segmented Sorted Array design |
| 2026-02-19 | Merge-based batch insert, deprecated `records()`, OOO buffer, range iterator |
