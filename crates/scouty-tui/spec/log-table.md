# Log Table

## Overview

The main log table widget displays parsed log records in a scrollable, column-based view with level coloring and selection tracking.


## Design

### Default Columns

Time | Log

- **Log** column auto-fills remaining width
- **Column separator**: a vertical line is displayed between adjacent columns for visual clarity. The separator character and color are themeable (default: `│`, see theme spec). For example, the `landmine` theme uses `♡`.
- Column widths adapt to content
- Empty fields display blank
- Optional columns (via `c` selector): Level, ProcessName, Pid, Tid, Component, Hostname, Container, Context, Function, Source (all hidden by default)

### Level Coloring

| Level | Color |
|-------|-------|
| FATAL | Red bold |
| ERROR | Soft red (`#FF6B6B`) |
| WARN | Warm yellow (`#FFD93D`) |
| NOTICE | Soft green (`#6BCB77`) |
| INFO | Light blue (`#4FC3F7`) |
| DEBUG | Medium gray (`#8B8B8B`) |
| TRACE | Dark gray (`#5C5C5C`) |

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
| 2026-02-23 | Default columns changed to Time + Log only; all others optional |
| 2026-02-23 | Add vertical separator (│) between columns |
| 2026-02-23 | Update level colors to match new theme (softer palette) |
