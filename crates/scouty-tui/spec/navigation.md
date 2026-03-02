# Navigation

## Overview

Advanced navigation features beyond basic j/k scrolling: bookmarks, relative time jumps, and go-to-line.


## Design

### Bookmarks

- `m` — toggle bookmark on current row
- `'` (single quote) — jump to next bookmark (cyclic)
- `"` (double quote) — jump to previous bookmark (cyclic)
- `M` — open bookmark manager dialog (list all, j/k navigate, Enter jump, d delete)
- Bookmarks stored in memory, associated with original record indices
- Bookmarks survive filter changes (tied to records, not filtered positions)
- Visual indicator on bookmarked rows (left bar or line number highlight)
- Status bar shows bookmark count: `Bookmarks: 3`

### Relative Time Jump

- `]` — jump forward (time increases): `[JUMP+] 5m█`
- `[` — jump backward (time decreases): `[JUMP-] 5m█`
- Supported formats: `Nms` (milliseconds), `Ns` (seconds), `Nm` (minutes), `Nh` (hours), `Nd` (days)
- Combined formats (P1): `1h30m`, `2m30s`
- Enter confirms: binary search (O(log N)) for nearest row to target timestamp
- Status bar feedback: `Jumped +5m to 2025-05-17 18:47:03`
- Out of range: jump to first/last row with notification

### Go to Line (`Ctrl+G`)

- Input dialog for line number
- Jumps to specified line in filtered view

### Follow Mode (`Ctrl+]`)

- Auto-scrolls to bottom, stays at newest record
- Status bar shows `[FOLLOW]`
- Manual scroll up auto-exits follow mode

## Change Log

| Date | Change |
|------|--------|
| 2026-02-20 | Basic navigation (j/k, g/G, Ctrl+G, follow mode) |
| 2026-02-22 | Bookmarks with manager dialog |
| 2026-02-22 | Relative time jump (`]`/`[`) |
