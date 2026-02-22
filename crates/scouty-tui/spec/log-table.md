# Log Table

## Overview

The main log table widget displays parsed log records in a scrollable, column-based view with level coloring and selection tracking.


## Design

### Default Columns

Time | Level | ProcessName | Pid | Tid | Component | Log

- **Log** column auto-fills remaining width
- Column widths adapt to content
- Empty fields display blank
- Optional columns (via `c` selector): Hostname, Container, Context, Function, Source (hidden by default)

### Level Coloring

| Level | Color |
|-------|-------|
| FATAL | Red bold |
| ERROR | Red |
| WARN | Yellow |
| NOTICE | Yellow (dim) |
| INFO | Green |
| DEBUG | Gray |
| TRACE | Dark gray |

### Navigation

- `j`/`k` — move one row
- `Ctrl+j`/`Ctrl+k`/`Ctrl+↑`/`Ctrl+↓` — page (half-screen)
- `g` — first row, `G` — last row
- `Ctrl+G` — go to specific line number (input dialog)
- Current selected row is highlighted

### Data Source

The table reads from `LogStoreView.filtered_indices` via the active view — it never copies log data.

## Change Log

| Date | Change |
|------|--------|
| 2026-02-20 | Initial log table design with columns, colors, navigation |
| 2026-02-20 | Added Hostname/Container optional columns |
| 2026-02-21 | Added Context/Function optional columns |
