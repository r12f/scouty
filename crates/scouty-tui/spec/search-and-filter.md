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
- **Cursor position after filter**: cursor stays on the same log record if it still exists in filtered results; if that record is filtered out, cursor moves to the nearest preceding record that remains visible (i.e., the last visible record before the original cursor position). Only if no preceding records remain does the cursor go to the first row.
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

### Log Level Quick Filter (`l`)

Press `l` to open a level selector overlay:

```
┌─ Level Filter ──────────┐
│                         │
│  1. ALL (no filter)     │
│  2. TRACE+              │
│  3. DEBUG+              │
│  4. INFO+               │
│  5. NOTICE+             │
│  6. WARN+               │
│  7. ERROR+              │
│  8. FATAL only          │
│                         │
│  Current: ALL           │
└─────────────────────────┘
```

**Behavior:**
- Press `1`-`8` to instantly apply level filter (overlay closes immediately)
- Or use `↑`/`↓`/`j`/`k` to navigate, `Enter` to confirm
- `Esc` closes without changing
- Level filter is additive — it combines with other active filters
- Applied filter shown in filter manager as a level filter entry
- Selecting a new level replaces the previous level filter (not stacked)
- Level mappings (severity low → high: Trace < Debug < Info < Notice < Warn < Error < Fatal):
  - `1` ALL: no level filter
  - `2` TRACE+: all levels shown (same as ALL, explicit)
  - `3` DEBUG+: TRACE excluded
  - `4` INFO+: TRACE, DEBUG excluded
  - `5` NOTICE+: TRACE, DEBUG, INFO excluded
  - `6` WARN+: TRACE, DEBUG, INFO, NOTICE excluded
  - `7` ERROR+: TRACE, DEBUG, INFO, NOTICE, WARN excluded
  - `8` FATAL only: only FATAL shown
- Current active level shown in the overlay

### Filter Presets (Save/Load)

Filter presets are stored in `~/.scouty/filters/` as YAML files.

**Save preset** — in filter manager (`F`), press `s`:
```
┌─ Save Filter Preset ────────────────┐
│                                     │
│  Name: my-error-filters█            │
│                                     │
│         [Enter] Save  [Esc] Cancel  │
└─────────────────────────────────────┘
```

**Load preset** — in filter manager (`F`), press `l`:
```
┌─ Load Filter Preset ────────────────┐
│                                     │
│  > my-error-filters                 │
│    network-debug                    │
│    production-noise                 │
│                                     │
│  [Enter] Load  [d] Delete  [Esc]   │
└─────────────────────────────────────┘
```

**Behavior:**
- Save: saves all current active filters (exclude + include + level) as a named preset
- File format: `~/.scouty/filters/<name>.yaml`
- Load: replaces all current filters with the preset's filters
- Delete: `d` key deletes the selected preset (with confirmation)
- Preset YAML format:
  ```yaml
  # ~/.scouty/filters/my-error-filters.yaml
  filters:
    - type: include
      expression: 'level == "Error" OR level == "Fatal"'
    - type: exclude
      expression: 'component == "healthcheck"'
  level_filter: 6    # WARN+ (0=ALL, 1=ALL, 2=TRACE+, 3=DEBUG+, 4=INFO+, 5=NOTICE+, 6=WARN+, 7=ERROR+, 8=FATAL)
  ```
- Empty filter list: show "No presets found" in load dialog
- Name collision on save: overwrite with confirmation prompt

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
| 2026-02-23 | Filter preserves cursor position (stay on same record or nearest preceding) |
| 2026-02-24 | Log level quick filter (l key, 1-5 selection) |
| 2026-02-24 | Filter presets save/load in filter manager (s/l keys, stored in ~/.scouty/filters/) |
| 2026-02-24 | Level filter expanded to all 7 levels: TRACE+/DEBUG+/INFO+/NOTICE+/WARN+/ERROR+/FATAL only (1-8) |
