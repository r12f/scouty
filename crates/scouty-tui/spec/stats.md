# Stats Panel

## Overview

A statistics panel providing a quick overview of log data distribution: level breakdown, top components, time range, and record counts. Part of the [panel system](panel-system.md).

## Design

### Panel Integration

**Tab name:** `Stats`
**Shortcut:** `S` (opens panel and switches to Stats tab; if already active, closes panel)
**Default height:** `Percentage(40)` (same as Region panel)

Tab order: `[Detail] [Region] [Stats]`

### Stats Content

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

Statistics are based on current `filtered_indices` (not full dataset). Switching to the Stats tab after a filter change shows updated stats.

### Status Bar Hints

When Stats panel has focus:
```
[STATS] Tab: Next Tab │ Ctrl+↑: Back │ z: Maximize │ Esc: Close
```

### P1: Top Source Files

When multiple files loaded, show per-file record count.

## Change Log

| Date | Change |
|------|--------|
| 2026-02-22 | Stats summary overlay with level distribution, top components |
| 2026-02-28 | Migrated from overlay to panel system as third panel tab |
