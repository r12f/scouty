# Configuration System

## Overview

Centralized configuration for scouty-tui: keybindings, theme selection, and general settings. Configuration is loaded at startup from `~/.scouty/config.yaml`, with sensible defaults when no config exists.

## Design

### Directory Structure

```
~/.scouty/
├── config.yaml          # Main configuration file
└── themes/              # Custom theme files
    ├── my-theme.yaml
    └── solarized.yaml
```

### Configuration File (`config.yaml`)

```yaml
# Theme selection
theme: "default"            # Built-in: "default", "dark", "light", "solarized"
                            # Or custom: name of a file in ~/.scouty/themes/ (without .yaml)

# General settings
general:
  follow_on_pipe: true      # Auto-enable follow mode for stdin/pipe input
  detail_panel_ratio: 0.3   # Detail panel width ratio (0.0 - 1.0)

# Keybindings (normal mode)
# Format: "<key>" or "<modifier>+<key>"
# Supported modifiers: ctrl, shift, alt
# Special keys: enter, esc, backspace, delete, home, end, pageup, pagedown, up, down, left, right, tab
keybindings:
  # Navigation
  move_down: "j"
  move_up: "k"
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
  prev_bookmark: "\""
  bookmark_manager: "M"

  # Copy & Export
  copy_raw: "y"
  copy_format: "Y"
  save_file: "ctrl+s"      # Or ":w" in command mode

  # General
  help: "?"
  quit: "q"
```

### Theme File Format (`~/.scouty/themes/<name>.yaml`)

```yaml
# Log level colors
log_levels:
  fatal: { fg: "red", bold: true }
  error: { fg: "red" }
  warn: { fg: "#FFD700" }           # Gold
  notice: { fg: "cyan" }
  info: { fg: "#00CC66" }           # Rich green
  debug: { fg: "gray" }
  trace: { fg: "dark_gray" }

# Table
table:
  header: { fg: "white", bg: "#1A1A2E" }
  selected: { bg: "#16213E" }
  alternating: { bg: "#0F0F1A" }

# Status bar
status_bar:
  line1: { bg: "#1A1A2E" }
  line2: { bg: "#16213E" }
  mode_label: { fg: "black", bg: "cyan" }
  density_chart: { fg: "cyan" }
  position: { fg: "white" }

# Search
search:
  match: { fg: "black", bg: "yellow" }
  current_match: { fg: "black", bg: "#FF6600" }

# Dialogs & Windows
dialog:
  border: { fg: "cyan" }
  title: { fg: "white", bold: true }
  selected: { fg: "white", bg: "#16213E" }
  text: { fg: "white" }
  muted: { fg: "dark_gray" }

# Detail panel
detail_panel:
  field_name: { fg: "cyan" }
  field_value: { fg: "white" }
  separator: { fg: "dark_gray" }

# Input fields
input:
  prompt: { fg: "yellow" }
  text: { fg: "white" }
  cursor: { fg: "white" }
  error: { fg: "red" }
  bg: { bg: "#1A1A2E" }

# Highlight palette (auto-rotation for user highlight rules)
highlight_palette:
  - "red"
  - "#00CC66"
  - "#3399FF"
  - "yellow"
  - "magenta"
  - "cyan"

# General
general:
  border: { fg: "#333366" }
  accent: { fg: "cyan" }
  muted: { fg: "dark_gray" }
```

### Color Value Formats

- **Named colors**: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `gray`, `dark_gray`
- **Hex RGB**: `"#RRGGBB"` (e.g., `"#FF6600"`)
- **256-color index**: `color(123)` (terminal 256-color palette)

### Loading Order

1. Load built-in default config (compiled into binary)
2. If `~/.scouty/config.yaml` exists, merge user config over defaults (partial overrides OK)
3. Resolve theme:
   - If `theme` is a built-in name → use built-in theme
   - If `~/.scouty/themes/<name>.yaml` exists → load and merge over built-in default theme
   - Otherwise → warn and fall back to default
4. CLI flags override config file values:
   - `--theme <name>` overrides `theme` in config
   - Future: `--bind key=action` for one-off overrides

### Keybinding Resolution

- Config defines a map of `action → key`
- At startup, invert to `key → action` lookup table
- Duplicate key detection: warn if same key maps to multiple actions
- Unknown action names: warn and skip
- Missing actions: use built-in defaults

### Implementation Notes

- Use `serde` + `serde_yaml` for config parsing (already a dependency for parser config)
- `Config` struct with `#[serde(default)]` for all fields — partial configs just work
- `Theme` struct loaded from config, passed by `&Theme` to all render functions
- Keybinding table built once at startup, used in main event loop dispatch

## Acceptance Criteria

- [ ] `~/.scouty/config.yaml` loaded at startup if present
- [ ] All keybindings configurable via config file
- [ ] Theme selected by name from config or `--theme` CLI flag
- [ ] Custom themes loadable from `~/.scouty/themes/`
- [ ] Partial config files work (missing fields use defaults)
- [ ] Invalid config: warn to stderr and continue with defaults
- [ ] All existing tests pass

## Change Log

| Date | Change |
|------|--------|
| 2026-02-22 | Initial configuration system design |
