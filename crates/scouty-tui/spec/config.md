# Configuration System

## Overview

Centralized configuration for scouty-tui: keybindings, theme selection, and general settings. Configuration will be loaded at startup from `~/.scouty/config.yaml`, with sensible defaults when no config exists.

## Design

### Directory Structure

```
~/.scouty/
├── config.yaml          # Main configuration file
└── themes/              # Custom theme files (see theme spec)
    ├── my-theme.yaml
    └── solarized.yaml
```

### Configuration File (`config.yaml`)

```yaml
# Theme selection (see theme.md for theme file format)
theme: "default"            # Built-in: "default", "dark", "light", "solarized"
                            # Or custom: name of a file in ~/.scouty/themes/ (without .yaml)

# Default paths to open when no files are specified on the command line
# Supports multiple entries and glob patterns (e.g., *.log, /var/log/*.log)
# Entries are processed in order; globs are expanded at runtime
# If CLI arguments are provided, this setting is ignored
default_paths:
  - "/var/log/syslog"
  - "/var/log/*.log"
  # - "/home/me/logs/**/*.log"   # recursive glob example

# General settings
general:
  follow_on_pipe: true      # Auto-enable follow mode for stdin/pipe input
  detail_panel_ratio: 0.3   # Detail panel height ratio (0.0 - 1.0)

# Keybindings (normal mode)
# Format: "<key>" or "<modifier>+<key>"
# Multiple keys per action: use a list ["j", "down"]
# Supported modifiers: ctrl, alt
# Special keys: enter, esc, backspace, delete, home, end, pageup, pagedown, up, down, left, right, tab
keybindings:
  # Navigation
  move_down: ["j", "down"]
  move_up: ["k", "up"]
  page_down: "pagedown"
  page_up: "pageup"
  scroll_to_top: "g"
  scroll_to_bottom: "G"
  goto_line: "ctrl+g"
  jump_forward: "]"
  jump_backward: "["
  toggle_follow: "ctrl+]"

  # Search & Filter
  search: "/"
  next_match: "n"
  prev_match: "N"
  filter: "f"
  quick_exclude: "-"
  quick_include: "="
  field_exclude: "_"
  field_include: "+"
  filter_manager: "F"

  # Display
  toggle_detail: "enter"
  column_selector: "c"
  stats: "S"

  # Highlight
  add_highlight: "h"
  highlight_manager: "H"

  # Bookmarks
  toggle_bookmark: "m"
  next_bookmark: "'"
  prev_bookmark: '"'
  bookmark_manager: "M"

  # Copy & Export (saving via ":w" in command mode)
  copy_raw: "y"
  copy_format: "Y"
  quick_export: "ctrl+s"

  # General
  help: "?"
  quit: "q"
```

### Loading Order

1. Load built-in default config (compiled into binary)
2. If `~/.scouty/config.yaml` exists, merge user config over defaults (partial overrides OK)
3. Resolve file paths:
   - If CLI arguments provided → use CLI files (ignore `default_paths`)
   - If no CLI arguments and `default_paths` configured → expand globs and open matching files
   - If no CLI arguments and no `default_paths` → fall back to platform defaults (see cli.md)
   - Glob patterns that match no files are silently skipped
   - If all paths resolve to no files → friendly error + usage hint
4. Resolve theme (see theme spec for theme file format and built-in presets):
   - Built-in name → use built-in theme
   - `~/.scouty/themes/<name>.yaml` exists → load and merge over default theme
   - Otherwise → warn and fall back to default
5. CLI flags override config:
   - `--theme <name>` overrides `theme` in config

### Keybinding Resolution

- Config defines `action → key` or `action → [keys]` mapping
- At startup, invert to `key → action` lookup table
- Duplicate key detection: warn if same key maps to multiple actions
- Unknown action names: warn and skip
- Missing actions: use built-in defaults

### Implementation Notes

- `serde` + `serde_yaml` for config parsing (already a dependency)
- `Config` struct with `#[serde(default)]` — partial configs just work
- Keybinding table built once at startup, used in main event loop dispatch
- Invalid config: warn to stderr and continue with defaults

## Acceptance Criteria

- [ ] `~/.scouty/config.yaml` loaded at startup if present
- [ ] All keybindings configurable via config file
- [ ] Theme selected by name from config or `--theme` CLI flag
- [ ] Partial config files work (missing fields use defaults)
- [ ] Invalid config: warn to stderr and continue with defaults
- [ ] All existing tests pass
- [ ] `default_paths` in config loaded when no CLI files specified
- [ ] Glob patterns expanded correctly (*, **, ?)
- [ ] CLI arguments take priority over `default_paths`
- [ ] Non-matching globs silently skipped

## Change Log

| Date | Change |
|------|--------|
| 2026-02-22 | Initial configuration system design |
| 2026-02-22 | Moved theme details to theme.md, kept references |
| 2026-02-23 | Added default_paths with glob support |
