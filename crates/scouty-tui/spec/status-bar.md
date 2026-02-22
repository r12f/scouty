# Status Bar

## Overview

The status bar occupies the bottom 2 lines, providing data overview (density chart + position) on line 1 and interactive state (mode/input/messages) on line 2.

## Current Status

✅ Implemented

## Design

### Line 1: Data Overview

- **Left**: Braille density chart (Unicode U+2800-U+28FF), adaptive width
- **Right**: `1,234/5,678 (Total: 10,000)`
- Density chart shows filtered log time distribution; each braille char = 2×4 dot matrix
- Current cursor position highlighted in the density chart (yellow)

### Line 2: Interactive State (Three Mutually Exclusive Modes)

**Mode A — Default (shortcut hints):**
```
[VIEW] /: Search │ f: Filter │ -: Exclude │ =: Include │ Enter: Detail │ ?: Help
```

**Mode B — Input mode:**
```
[SEARCH] pattern█    [FILTER] level = "Error"█    [GOTO] 1234█
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
