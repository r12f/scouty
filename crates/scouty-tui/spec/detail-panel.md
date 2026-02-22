# Detail Panel

## Overview

The detail panel displays the full content of the currently selected log record in a left-right split layout: log content on the left, structured fields on the right.


## Design

### Layout

- **Left side (~70% width)**: Full log content (`raw` field), with word wrap. Scrollable if content exceeds panel height.
- **Right side (~30% width)**: Structured field table (Field | Value), fixed-width field column, value fills remaining space. Truncated (no wrap) for long values.
- Vertical separator between left and right areas.
- Left title: "Log Content"; Right title: "Fields"

### Fields Displayed (Right Side)

Timestamp, Level, Source, Hostname, Container, Context, Function, Component, Process, PID, TID, plus all metadata KV pairs. Only non-empty fields shown.

Level shows `-` when empty. Source always shown even if hidden in table columns.

### Behavior

- `Enter` toggles panel open/close
- Panel follows cursor — updates when selected row changes
- Panel height adapts to field count + border (not fixed percentage)
- Narrow window (< 80 cols): right side collapses or switches to single-column

## Change Log

| Date | Change |
|------|--------|
| 2026-02-20 | Initial detail panel (single column) |
| 2026-02-22 | Left-right split layout redesign |
