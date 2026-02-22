# Search and Filter

## Overview

Interactive search and filtering in the TUI, providing regex search with match navigation and multiple filter interaction modes.


## Design

### Search (`/`)

- Opens search input in status bar
- Supports regex matching
- Searches within current filtered results (not full dataset)
- Matching rows highlighted (yellow background)
- `n` / `N` — next / previous match
- `Esc` — close search (highlights persist until next search or clear)

### Filter Expression Input (`f`)

- Opens input with field name hints
- Full expression syntax (see `crates/scouty/spec/filter.md`)
- Enter → creates pending LogStoreView → replaces active view on completion
- Esc cancels

### Quick Exclude (`-`) / Quick Include (`=`)

- Text input; adds exclude/include filter for records containing that text

### Field Filter Dialog (`_` / `+`)

Shared dialog component, differing only in initial action (exclude vs include):

```
┌─ Exclude Filter ─────────────────────┐
│ Condition: [OR] / AND                │
│                                      │
│ [ ] Before 2025-05-17 18:42:03.233   │
│ [ ] After  2025-05-17 18:42:03.233   │
│ [x] Level      = ERROR               │
│ [ ] Source     = /var/log/syslog      │
│ [ ] Process    = myapp               │
│ ...                                  │
│         [Apply]  [Cancel]            │
└──────────────────────────────────────┘
```

- Based on currently selected row's field values
- All fields shown (including metadata KV pairs)
- Multi-select with OR (default) or AND condition
- **Time range options** at top: "Before this log" / "After this log"
- j/k/↑/↓ navigation, PageUp/PageDown, Space toggle, Enter apply

### Filter Manager (`F`)

- Lists all active filters (exclude/include)
- Add new, delete individual, clear all
- j/k navigation, PageUp/PageDown

### Time Range Filtering

Via the field filter dialog's time options:
- Exclude "Before this log" → `timestamp < "..."` (current row included)
- Exclude "After this log" → `timestamp > "..."` (current row included)
- Combine both for a time window
- Visible and manageable in Filter Manager

## Change Log

| Date | Change |
|------|--------|
| 2026-02-20 | Search, filter expression, quick exclude/include, field dialog, filter manager |
| 2026-02-22 | Time range options in field filter dialog |
