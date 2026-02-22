# Help Dialog

## Overview

A scrollable help overlay displaying all available keybindings, version information, and project links.


## Design

### Trigger

`?` — opens the help overlay as a centered popup.

### Content

- Complete keybinding reference table (all shortcuts with descriptions)
- Grouped by category: Navigation, Search, Filter, Bookmarks, Highlight, Copy/Export, Misc
- Version number (from `Cargo.toml`)
- GitHub repository URL: `https://github.com/r12f/scouty`

### Behavior

- Scrollable when content exceeds popup height
- `Esc` or `?` to close
- j/k/↑/↓ and PageUp/PageDown for scrolling

## Change Log

| Date | Change |
|------|--------|
| 2026-02-20 | Help dialog with keybinding reference |
