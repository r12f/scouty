# Column Width Adjustment - High-Level Spec

## Background & Goals

The Column Selector dialog (`c` key) currently only supports toggling column visibility. Column widths are auto-computed with hardcoded maximums, which often truncates important content (e.g., long component names like `SAI_OBJECT_TYPE_ROUTE_ENTRY`). Users need the ability to manually adjust column widths.

## Problem Statement

- Column widths are auto-computed with fixed max limits — users cannot widen important columns
- Long values in Component, Context, Function columns get truncated
- No way to customize the balance between metadata columns and the Log (message) column

## User Stories

- As a log analyst, I want to adjust individual column widths in the Column Selector dialog, so I can see full values for the columns I care about
- As a log analyst, I want the Log (message) column to always fill remaining screen width, so screen space is never wasted

## Requirements Breakdown

### P0 — Must Have

- [ ] **Width adjustment via h/l or Left/Right keys** in the Column Selector dialog (dependency: none)
  - When cursor is on a column, `h`/Left decreases width, `l`/Right increases width
  - Step size: 1 character per keypress (or a reasonable increment)
  - Each column has a minimum width (e.g., label length) to prevent collapsing to zero
  - Width changes take effect immediately (live preview in the log table behind the dialog)

- [ ] **Log column always fills remaining screen width** (dependency: none)
  - The Log (message) column width = total available width − sum of all other visible column widths
  - Log column width is not manually adjustable — it auto-expands/contracts
  - Log column is always the last (rightmost) column

- [ ] **Display current width in the Column Selector** (dependency: none)
  - Each column row shows its current width value (e.g., `[x] Component   20`)
  - Log column shows "auto" or "fill" instead of a number

### P1 — Should Have

- [ ] **Persist user-set column widths** (dependency: config system)
  - User-adjusted widths survive dialog close (within the same session)
  - Once a width is manually set, auto-compute no longer overrides it for that column
  - Reset to auto-compute: a dedicated key (e.g., `r` on the selected column) clears the manual override

## Functional Requirements

### Interaction Design

```
┌─ Columns (c) ──────────────────────┐
│ Toggle (Space) / Width (h/l)       │
│                                    │
│ [x] Time         23                │
│ [x] Level         5                │
│ [ ] Hostname      -                │
│ [x] Component    20  ← cursor     │
│ [x] Function     12               │
│ [x] Context      15               │
│ [x] Log         fill               │
│                                    │
│ r: Reset width  Esc: Close         │
└────────────────────────────────────┘
```

- **Space/Enter**: toggle visibility (existing behavior)
- **h/Left**: decrease width by 1
- **l/Right**: increase width by 1
- **r**: reset column width to auto-computed value
- Disabled columns show `-` for width (not adjustable when hidden)
- Log column shows `fill` (not adjustable)

### Width Constraints

| Column | Min Width | Default Max Width |
|--------|-----------|-------------------|
| Time | 19 | 23 |
| Level | 3 | 5 |
| Hostname | 4 | 20 |
| Container | 4 | 20 |
| ProcessName | 4 | 20 |
| Pid | 3 | 8 |
| Tid | 3 | 8 |
| Component | 4 | 30 |
| Function | 4 | 20 |
| Context | 4 | 40 |
| Source | 4 | 30 |
| Log | fill | fill |

Users can set widths beyond the default max — the max only applies to auto-compute.

## Non-Functional Requirements

- **Performance**: Width changes should re-render the log table immediately with no perceptible lag
- **No config file changes**: Width overrides are in-memory for P0; persistence to config file is P1

## Acceptance Criteria

- [ ] h/l or Left/Right keys adjust the selected column's width in the Column Selector
- [ ] Log column always fills remaining screen width
- [ ] Column Selector shows current width for each column
- [ ] Width cannot go below minimum (column label length or defined minimum)
- [ ] Disabled columns cannot be width-adjusted
- [ ] Width changes are reflected immediately in the log table

## Out of Scope

- Drag-to-resize columns (TUI limitation)
- Per-column alignment options
- Column reordering (separate feature)

## Open Questions

None
