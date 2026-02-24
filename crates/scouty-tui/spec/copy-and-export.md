# Copy and Export

## Overview

Features for copying log records to clipboard and exporting filtered results to files.


## Design

### Copy to Clipboard

- `y` — copy current selected row's raw text to clipboard
- `Y` — open format selection dialog: Raw (default) / JSON / YAML
- Multi-row selection supported
- JSON/YAML serializes all structured LogRecord fields
- Uses **OSC 52 escape sequence** for cross-terminal clipboard access
- Dialog supports j/k/↑/↓ navigation

### Save / Export (`s` key)

Press `s` to open the save dialog:

```
┌─ Save Logs ──────────────────────────────┐
│                                          │
│  Path: ~/export.log█                     │
│                                          │
│  Format:                                 │
│    > Raw (one line per record)           │
│      JSON                                │
│      YAML                                │
│                                          │
│              [Enter] Save  [Esc] Cancel  │
└──────────────────────────────────────────┘
```

**Behavior:**
- Path input field is focused by default, with a sensible default path (e.g., `./scouty-export.log`)
- `Tab` or `↓` moves focus to format selector
- Format selector uses `↑`/`↓` or `j`/`k` to navigate, `Enter` to confirm
- Format options: **Raw** (default) / **JSON** / **YAML**
  - Raw: one `raw` field per line (as-is from log)
  - JSON: array of LogRecord objects with all structured fields
  - YAML: list of LogRecord objects with all structured fields
- `Enter` on Save: exports current filtered view's records to the specified path
- Completion message in status bar: `Saved 5,678 records to ~/export.log (raw)`
- Path supports `~` expansion
- Empty path: show inline error "Path required"
- Write failure: show error in status bar (e.g., `Save failed: Permission denied`)

**Replaces:** The old `:w <filename>` command mode export is removed. Command mode (`:`) remains available for future extensibility.

## Change Log

| Date | Change |
|------|--------|
| 2026-02-20 | Copy to clipboard (y/Y with format selection, OSC 52) |
| 2026-02-22 | Command mode `:w` export |
| 2026-02-24 | Replace `:w` with `s` key save dialog (path input + format selection) |
