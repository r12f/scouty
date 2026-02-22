# Filter Expression Syntax

## Overview

The filter engine evaluates expressions against LogRecord fields to include or exclude records. It supports a rich expression language with field comparisons, logical operators, and string matching.

## Current Status

✅ Implemented

## Design

### Expression Syntax

```
<field> <op> <value> [AND|OR <more filters>]
```

### Supported Operators

| Category | Operators |
|----------|-----------|
| Comparison | `=`, `!=`, `>`, `>=`, `<`, `<=` |
| Logical | `AND`, `OR`, `NOT` |
| String matching | `contains`, `starts_with`, `ends_with`, `regex` |
| Grouping | Parentheses `()` for precedence control |

### Examples

```
level = "Error"
(level = "Error" OR level = "Fatal") AND component = "auth"
message contains "timeout" AND source starts_with "/var/log"
timestamp > "2025-05-17T18:42:00" AND timestamp < "2025-05-17T19:00:00"
hostname = "BSL-0101" AND container = "pmon"
```

### Addressable Fields

All LogRecord fields including: `timestamp`, `level`, `source`, `hostname`, `container`, `context`, `function`, `pid`, `tid`, `component`, `process_name`, `message`, and any metadata keys.

### Filter Actions

Each filter has an action that takes effect when matched:
- **Exclude** — remove matching records
- **Include** — keep matching records

### Evaluation Order

1. Check all **Exclude** filters first — if any match, record is excluded
2. Then check **Include** filters:
   - If no Include filters exist → include all (not excluded)
   - If Include filters exist → only include matched records

### Time Range Filters

Quick time-based filtering via `Ctrl+-` / `Ctrl+=` dialogs:
- "Before this log" → `timestamp < "YYYY-MM-DDTHH:MM:SS.ffffff"`
- "After this log" → `timestamp > "YYYY-MM-DDTHH:MM:SS.ffffff"`
- Current row is **included** (boundary inclusive)
- Combine both to create a time window
- Time filters stack with field filters via AND logic

## Change Log

| Date | Change |
|------|--------|
| 2026-02-18 | Initial filter expression design |
| 2026-02-22 | Time range filter via Exclude/Include field dialog |
