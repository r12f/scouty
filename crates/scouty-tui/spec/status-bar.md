# Status Bar

## Overview

The status bar occupies the bottom 2 lines, providing data overview (density chart + position) on line 1 and interactive state (mode/input/messages) on line 2.


## Design

### Line 1: Data Overview

- **Left**: Braille density chart (Unicode U+2800-U+28FF), adaptive width
- **Right**: `1,234/5,678 (Total: 10,000)`
- Density chart shows filtered log time distribution; each braille char = 2×4 dot matrix
- Current cursor position highlighted in the density chart (yellow)

### Density Chart Time-Per-Column Label

Display a time-per-column label to the left of the density chart on line 1, showing the time span each braille column represents.

**Format:** `[█=Xs]` where X is dynamically computed as `(max_ts - min_ts) / num_buckets`, then snapped up to the nearest standard interval.

**Time snapping:** The raw computed duration per column is rounded **up** to the nearest value in the following progression:
- Seconds: 0, 5, 15, 30
- Minutes: 0, 5, 15, 30
- Hours: 1, 2, 6, 12, 24

For example: raw 3s → snap to 5s, raw 8s → snap to 15s, raw 20s → snap to 30s, raw 40s → snap to 1m, raw 3m → snap to 5m, raw 8m → snap to 15m, raw 45m → snap to 1h.

The number of buckets should be adjusted accordingly so that the chart covers the full time range with the snapped interval.

**Unit auto-selection:**
- < 1s → ms (e.g., `[█=500ms]`)
- 1s to < 60s → s (e.g., `[█=5s]`)
- 1m to < 60m → m (e.g., `[█=2m]`)
- ≥ 60m → h (e.g., `[█=1h]`)

**Layout:** `[█=5s]⣿⣷⣶⣤⣀⣿⣷⣶  42/100 (Total: 500)`

**Behavior:**
- Label width is deducted from density chart available width
- Hidden when no data or only one timestamp exists
- Visually distinguishable from chart (e.g., dimmer color)

**Acceptance criteria:**
- [ ] Label displayed left of density chart in `[█=time]` format
- [ ] Time unit dynamically computed and auto-selected (ms/s/m/h)
- [ ] Time per column snapped up to nearest standard interval (5/15/30 for s and m)
- [ ] Label width deducted from chart width, no layout overflow
- [ ] Hidden when insufficient data

### Density Chart Level/Highlight Selector

The density chart can be switched to show only a specific log level or highlight group, making it easy to visually locate error clusters or highlighted regions.

**Trigger:** Press `d` to cycle through density chart modes, or `D` to open a selector overlay:

```
┌─ Density Chart Source ──────────────┐
│                                     │
│  > All records (default)            │
│    FATAL only                       │
│    ERROR only                       │
│    WARN only                        │
│    INFO only                        │
│    ── Highlights ──                 │
│    Highlight #1: "timeout"          │
│    Highlight #2: "connection"       │
│                                     │
│         [Enter] Select  [Esc] Close │
└─────────────────────────────────────┘
```

**Behavior:**
- Default: density chart shows all filtered records (current behavior)
- When a level is selected: chart only counts records of that level → spikes = error clusters
- When a highlight is selected: chart only counts records matching that highlight rule
- Current selection shown in the density label: `[█=5s ERROR]` or `[█=5s "timeout"]`
- `d` quick-cycles: All → ERROR → WARN → current highlights → All
- `D` opens full selector overlay with all options
- Selection persists until changed or reset
- Cursor marker still works in filtered density view

**Mode A — Default (shortcut hints):**

Shortcut hints are **context-sensitive** — they update based on current focus:

**Log Table focus (default):**
```
[VIEW] /: Search │ f: Filter │ -: Exclude │ =: Include │ Enter: Detail │ ?: Help
```

**Detail Panel focus:**
```
[DETAIL] Tab: Next Tab │ Ctrl+↑: Back │ z: Maximize │ Esc: Close │ ?: Help
```

**Region Panel focus:**
```
[REGION] j/k: Navigate │ Tab: Next Tab │ Ctrl+↑: Back │ z: Maximize │ Esc: Close
```

When focus switches (e.g., `Ctrl+↓` to enter panel, `Tab` to switch panel, `Ctrl+↑` to return to log table), the status bar line 2 immediately updates to show the relevant shortcuts for the new context.

**Mode B — Input mode:**
```
[SEARCH] pattern█    [FILTER] level == "Error"█    [GOTO] 1234█
```
Triggered by `/`, `f`, `-`, `=`, `Ctrl+G`, `h`, `:`. Esc/Enter exits input.

**Mode C — Temporary status message:**
```
Copied to clipboard    Filter applied: 5,678 records    No matches found
```
Auto-clears after 3 seconds or on next keypress.

### Mode Label Colors

- `[VIEW]` — default
- `[FOLLOW]` — green/cyan
- `[SEARCH]`/`[FILTER]`/`[EXCLUDE]`/`[INCLUDE]`/`[GOTO]` — yellow

### Density Chart Caching

The density chart is **cached** and only recomputed when:
- Filter conditions change (filtered_indices version change)
- Window width changes (bucket count changes)

Navigation (j/k/PageUp/PageDown/g/G) does **not** trigger recomputation. Only the cursor highlight position is updated per frame (O(log N) binary search on cached time range).

## Performance

- Navigation must be < 16ms/frame (60fps target)
- Cache overhead: < 1KB (buckets array + braille string)

## Change Log

| Date | Change |
|------|--------|
| 2026-02-20 | Initial single-line status bar with density chart |
| 2026-02-22 | Two-line redesign with mode/input/message states |
| 2026-02-22 | Density chart caching for navigation performance |
| 2026-02-22 | Add time-per-column label spec for density chart |
| 2026-02-23 | Label format changed from [Xs/█] to [█=Xs] |
| 2026-02-23 | Time per column snaps up to standard intervals (5/15/30 for s and m) |
| 2026-02-24 | Density chart level/highlight selector (d/D keys) |
| 2026-02-28 | Status bar line 2 shows context-sensitive shortcuts based on current focus (log table / detail panel / region panel) |
