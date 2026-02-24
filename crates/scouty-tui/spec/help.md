# Help Dialog

## Overview

A scrollable help overlay displaying all available keybindings, version information, and project links.


## Design

### Trigger

`?` — opens the help overlay as a centered popup.

### Content

- Complete keybinding reference table grouped by context and category (see below)
- Version number (from `Cargo.toml`)
- GitHub repository URL: `https://github.com/r12f/scouty`

### Keybinding Reference

The help dialog shows keybindings organized by **context** (where they are active) and **category** (what they do). Keybindings marked "Log Table View" only appear when the main log table is focused (no overlay/dialog open).

#### Global (always active)

| Key | Function |
|-----|----------|
| `Esc` | Close current overlay / cancel input |
| `q` | Quit |
| `?` | Help |

#### Log Table View — Navigation

| Key | Function |
|-----|----------|
| `j` / `k` | Move up/down one row |
| `Ctrl+j` / `Ctrl+k` | Page up/down |
| `g` / `G` | First/last row |
| `Ctrl+G` | Go to line number |
| `]` / `[` | Relative time jump forward/backward |
| `Ctrl+]` | Toggle follow mode |
| `Enter` | Toggle detail panel |

#### Log Table View — Search & Filter

| Key | Function |
|-----|----------|
| `/` | Search (regex) |
| `n` / `N` | Next/prev search match |
| `f` | Filter expression input |
| `-` / `=` | Quick exclude/include text |
| `_` / `+` | Field exclude/include dialog |
| `F` | Filter manager |
| `l` | Log level quick filter (1-8) |

#### Log Table View — Display & Analysis

| Key | Function |
|-----|----------|
| `c` | Column selector |
| `d` / `D` | Cycle / select density chart source |
| `h` / `H` | Add highlight / highlight manager |
| `S` | Stats summary |

#### Log Table View — Bookmarks

| Key | Function |
|-----|----------|
| `m` | Toggle bookmark |
| `'` / `"` | Next/prev bookmark |
| `M` | Bookmark manager |

#### Log Table View — Copy & Export

| Key | Function |
|-----|----------|
| `y` / `Y` | Copy raw / format selection |
| `s` | Save/export dialog |

#### Dialog Navigation (shared across all dialogs)

| Key | Function |
|-----|----------|
| `j` / `k` / `↑` / `↓` | Move selection |
| `PageUp` / `PageDown` | Page through options |
| `Space` | Toggle selection (multi-select dialogs) |
| `Enter` | Confirm |
| `Esc` | Cancel / close |

### Behavior

- Scrollable when content exceeds popup height
- `Esc` or `?` to close
- j/k/↑/↓ and PageUp/PageDown for scrolling

## Change Log

| Date | Change |
|------|--------|
| 2026-02-20 | Help dialog with keybinding reference |
| 2026-02-24 | Explicit keybinding groups by context (global vs log table view vs dialog) |
