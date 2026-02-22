# Log Store View

## Overview

`LogStoreView` encapsulates a `FilterEngine` and its cached filter results, enabling a **double-buffering** mechanism: the active view serves the TUI while a pending view filters in the background, then atomically replaces the active view.

## Current Status

✅ Implemented

## Design

### LogStoreView Structure

```rust
struct LogStoreView {
    filter_engine: FilterEngine,
    filtered_indices: Vec<usize>,   // Cached indices into LogStore
    status: ViewStatus,             // Ready | Filtering
    last_applied_index: usize,      // For incremental filtering
}
```

**Methods:**
- `new(filter_engine)` — create view
- `apply(&mut self, store: &LogStore)` — full filter, update cached indices
- `apply_incremental(&mut self, store: &LogStore)` — filter only new records (for live streams)
- `indices() -> &[usize]` — current filter results
- `get_record(index, store) -> Option<&LogRecord>` — get record by filtered index
- `len()` — filtered record count
- `filter_engine() / filter_engine_mut()` — access filter engine

### Double-Buffering in LogSession

```rust
struct LogSession {
    active_view: LogStoreView,          // Current view serving TUI reads
    pending_view: Option<LogStoreView>, // Next view being filtered
}
```

**Workflow:**
1. User modifies filter → create new `LogStoreView` with new filter → set as `pending_view`
2. Call `pending_view.apply(&store)` to execute filtering
3. On completion, `pending_view` replaces `active_view`
4. If filter changes again during pending filter, discard old pending and create new one

Key decisions:
- **Active view always available** — UI never blocks waiting for filter results
- **Indices only, no record copies** — `filtered_indices` stores `Vec<usize>`, not records
- **Incremental filtering** — for live log streams, only new records are filtered and appended
- **View statistics** — total count / filtered count / per-level counts for status bar display

## Change Log

| Date | Change |
|------|--------|
| 2026-02-20 | LogStoreView design with double-buffering mechanism |
