# Highlight Rules

## Overview

Custom highlight rules allow users to visually mark multiple patterns simultaneously with different colors, independent of search and filtering.


## Design

### Quick Highlight (`h`)

- Status bar input: `[HIGHLIGHT] pattern█`
- Supports regex (same as search)
- System auto-assigns color from a rotating palette: red, green, blue, yellow, magenta, cyan...
- Multiple rules active simultaneously
- Highlights render as **full-row background color** in the log table (the entire row is colored, not just matching text)
- Overlapping matches: later-added rule takes priority
- **Highlights are purely visual** — they do not affect filtering

### Highlight Manager (`H`)

- Overlay listing all highlight rules
- Each entry: color swatch + regex pattern + match count
- j/k navigate, d delete, Space enable/disable, Enter close
- Design consistent with Filter Manager (`F`)

## Change Log

| Date | Change |
|------|--------|
| 2026-02-22 | Custom highlight rules with manager dialog |
| 2026-02-23 | Highlight renders full row, not just matching text |
