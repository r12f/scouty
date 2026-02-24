# Detail Panel

## Overview

The detail panel displays the full content of the currently selected log record in a left-right split layout: log content on the left, structured fields on the right. When the log record has parser-provided structured expansion, the left side renders an interactive tree view.


## Design

### Layout

- **Left side (~70% width)**: Structured expansion tree (if `expanded` is present) or full log content (`raw` field) with word wrap. Scrollable.
- **Right side (~30% width)**: Structured field table (Field | Value), fixed-width field column, value fills remaining space. Truncated (no wrap) for long values.
- Vertical separator between left and right areas.
- Left title: "Expanded" (when tree) or "Log Content" (when raw); Right title: "Fields"

### Structured Expansion Tree (Left Side)

When `LogRecord.expanded` is populated by the parser, the left side renders an interactive tree:

```
▼ Operation: SET
▼ Table: ROUTE_TABLE
▼ Key: Vrf1:10.0.0.0/24
▼ Attributes
    nexthop: 10.1.1.1
    ifname: Ethernet0
```

JSON example:
```
▼ Payload
    service: auth
    msg: login failed
  ▼ details
      user: alice
      ip: 10.0.0.1
```

**Tree behavior:**
- `▼` / `▶` indicators for collapsible nodes (KeyValue and List types)
- `j`/`k` to navigate tree nodes
- `Enter` or `l` to expand/collapse a node
- `h` to collapse current node (or go to parent if already collapsed)
- `H` to collapse all; `L` to expand all
- Indentation: 2 spaces per nesting level
- Leaf nodes (Text values): `label: value` on one line
- Long values: truncated with `…`, full value shown on hover/select
- When `expanded` is not populated: falls back to raw text display (current behavior)

### Quick Filter from Expanded Fields

When navigating the tree, press `f` on a leaf node to create a filter from that field:

- **KeyValue leaf**: creates filter `key == "value"` (using the field's key path)
- Status bar shows: `Filter added: details.user == "alice"`
- Works with existing filter system (additive)

This gives users a fast path from browsing structured logs to filtering by any field.

### Fields Displayed (Right Side)

Timestamp, Level, Source, Hostname, Container, Context, Function, Component, Process, PID, TID, plus all metadata KV pairs. Only non-empty fields shown.

Level shows `-` when empty. Source always shown even if hidden in table columns.

### Behavior

- `Enter` toggles panel open/close (when panel is closed)
- When panel is open and expanded tree is shown, `Enter` toggles tree nodes
- Panel follows cursor — updates when selected row changes
- Panel height adapts to field count + border (not fixed percentage)
- Narrow window (< 80 cols): right side collapses or switches to single-column
- `Tab` switches focus between left (tree/content) and right (fields) sides

## Change Log

| Date | Change |
|------|--------|
| 2026-02-20 | Initial detail panel (single column) |
| 2026-02-22 | Left-right split layout redesign |
| 2026-02-24 | Structured expansion tree rendering with interactive navigation and quick filter |
