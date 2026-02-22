# Stats Summary

## Overview

A statistics overlay providing a quick overview of log data distribution: level breakdown, top components, time range, and record counts.


## Design

### Stats Overlay (`S`)

Centered popup (similar to Help window) showing:

**Level Distribution** — count, percentage, and horizontal bar chart:
```
FATAL:     2 (0.0%)  ▎
ERROR:   156 (1.2%)  ██
WARN:    489 (3.8%)  █████
INFO:  8,234 (63.1%) ████████████████████████████████
DEBUG: 4,012 (30.7%) ████████████████
```

**Top 10 Components** — sorted by record count

**Time Range** — earliest → latest timestamp, total span

**Record Counts** — total / filtered count

### Data Source

Statistics are based on current `filtered_indices` (not full dataset). Reopening after filter change shows updated stats.

### P1: Top Source Files

When multiple files loaded, show per-file record count.

## Change Log

| Date | Change |
|------|--------|
| 2026-02-22 | Stats summary overlay with level distribution, top components |
