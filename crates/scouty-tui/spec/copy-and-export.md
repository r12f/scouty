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

### Export (Command Mode)

- `:` — enter command mode, status bar: `[CMD] :█`
- `:w <filename>` — export current filtered view's records (`raw` field, one line per record)
- Completion: `Saved 5,678 records to filtered.log`
- No filename: error `Usage: :w <filename>`
- Command mode extensible for future commands

> **Note:** Prior to command mode implementation, export uses `Ctrl+s` with a `[SAVE]` prompt.

## Change Log

| Date | Change |
|------|--------|
| 2026-02-20 | Copy to clipboard (y/Y with format selection, OSC 52) |
| 2026-02-22 | Command mode `:w` export |
