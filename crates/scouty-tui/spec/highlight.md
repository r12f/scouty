# Highlight Rules

## Overview

Custom highlight rules allow users to visually mark multiple patterns simultaneously with different colors, independent of search and filtering.


## Design

### Quick Highlight (`h`)

- Status bar input: `[HIGHLIGHT] pattern█`
- Supports regex (same as search)
- System auto-assigns color from a rotating palette: red, green, blue, yellow, magenta, cyan...
- Multiple rules active simultaneously
- Highlights render in the Log column of the log table
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
