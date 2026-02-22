# Performance Optimization

## Overview

End-to-end performance optimization covering the full loading pipeline (file I/O → parsing → LogStore insertion → TUI first render) and TUI navigation responsiveness.

## Current Status

✅ Implemented

## Design

### E2E Loading Pipeline

**Target:** 1.52M lines syslog (169MB) loaded in < 1 second (release build)

**Profiling approach:**
- `cargo flamegraph` / `perf` to identify top-5 hotspots
- Measure each stage: file read, parse, insert, render

**Optimization directions (based on profiling):**
- **Reduce allocations**: `&str` over `String`, arena allocator / string interning
- **Batch I/O**: larger BufReader buffer, or mmap
- **Timestamp parsing**: hand-written parser replacing chrono for common formats
- **Parallel parsing**: multi-thread parse + single-thread insert
- **Lazy loading**: parse first N lines for first screen, async load remainder

**Loading progress indicator:** TUI shows line count / percentage during load.

### TUI Navigation Performance

**Problem:** j/k navigation caused visible stutter because density chart was recomputed every frame — O(N) allocation + traversal on 1.5M records.

**Solution: Density Chart Caching**

Cache is only invalidated when:
1. Filter conditions change (filtered_indices version)
2. Window width changes (bucket count)

Navigation only updates:
- Selected row highlight — O(1)
- Line count display — O(1)
- Cursor position in density chart — O(log N) binary search on cached time range

**Input coalescing:** Long-press j/k consumes all pending events before drawing one frame.

### Criterion Benchmarks

Located in `crates/scouty/benches/`:
- `parse_syslog_single` — single line
- `parse_syslog_batch_1k` / `parse_syslog_batch_100k`
- End-to-end loading benchmark

## Performance Benchmarks

| Metric | Target |
|--------|--------|
| 1.52M syslog e2e load | < 1s (release) |
| Navigation frame time | < 16ms (60fps) |
| Syslog parse throughput | ≥ 10M rec/sec |
| Density chart cache overhead | < 1KB |

## Change Log

| Date | Change |
|------|--------|
| 2026-02-21 | E2E performance profiling and optimization spec |
| 2026-02-22 | Navigation performance: density chart caching, input coalescing |
