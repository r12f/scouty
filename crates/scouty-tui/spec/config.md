# Configuration System

## Overview

Centralized configuration for scouty-tui: keybindings, theme selection, and general settings. Configuration is loaded from multiple sources with layered override semantics — later sources override earlier ones, allowing system-wide defaults, per-user customization, and per-invocation CLI overrides.

## Design

### Directory Structure

```
/etc/scouty/
├── config.yaml          # System-wide configuration (admin-managed)
└── themes/              # System-wide custom themes

~/.scouty/
├── config.yaml          # Per-user configuration
└── themes/              # Per-user custom theme files (see theme spec)
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

  # Copy & Export (saving via "s" dialog)
  copy_raw: "y"
  copy_format: "Y"
  save_export: "s"

  # Level & Density
  level_filter: "l"
  density_cycle: "d"
  density_select: "D"

  # General
  help: "?"
  quit: "q"
```

### Loading Order

Configuration is loaded in layers. Each layer merges on top of the previous one (field-level override, not full replacement). Later layers take priority:

1. **Built-in defaults** (compiled into binary, lowest priority)
2. **System config** — `/etc/scouty/config.yaml` (if exists)
3. **User config** — `~/.scouty/config.yaml` (if exists)
4. **Local config** — `./scouty.yaml` in current working directory (if exists) — project-level overrides
5. **CLI flags** (highest priority)
   - `--theme <name>` overrides `theme`
   - `--config <path>` loads an additional config file after local config (overrides all file-based configs)
   - File arguments override `default_paths`

**Merge semantics:**
- Scalar values (string, number, bool): later layer replaces earlier
- Maps (keybindings, general): deep merge — only specified keys are overridden, unspecified keys keep previous value
- Lists (default_paths): later layer **replaces** the entire list (not appended)
- A key explicitly set to `null` or empty resets it to built-in default

**Example — system admin sets company defaults, user overrides theme:**

`/etc/scouty/config.yaml`:
```yaml
theme: "corporate"
default_paths:
  - "/var/log/app/*.log"
general:
  follow_on_pipe: false
```

`~/.scouty/config.yaml`:
```yaml
theme: "solarized"
keybindings:
  quit: "ctrl+q"
```

Result: theme=solarized (user wins), default_paths=[/var/log/app/*.log] (from system), follow_on_pipe=false (from system), quit=ctrl+q (user override), all other keybindings=built-in defaults.

**Example — project-level local config:**

`./scouty.yaml` (in project root, checked into repo):
```yaml
default_paths:
  - "./logs/*.log"
general:
  follow_on_pipe: true
```

This overrides user and system configs for anyone running scouty in this directory. Useful for per-project log paths and parser settings.

6. Resolve file paths:
   - If CLI file arguments provided → use CLI files (ignore `default_paths`)
   - If no CLI arguments and `default_paths` configured → expand globs and open matching files
   - If no CLI arguments and no `default_paths` → fall back to platform defaults (see cli.md)
   - Glob patterns that match no files are silently skipped
   - If all paths resolve to no files → friendly error + usage hint
6. Resolve theme (see theme spec for theme file format and built-in presets):
   - Theme search order: `~/.scouty/themes/` → `/etc/scouty/themes/` → built-in presets
   - Built-in name → use built-in theme
   - Custom theme file found → load and merge over default theme
   - Otherwise → warn and fall back to default

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

- [ ] Built-in defaults → `/etc/scouty/config.yaml` → `~/.scouty/config.yaml` → `./scouty.yaml` → CLI flags, layered override
- [ ] Each layer does field-level deep merge (not full replacement)
- [ ] Lists (e.g., `default_paths`) are replaced entirely by later layers, not appended
- [ ] Missing config files silently skipped (no error)
- [ ] `--config <path>` loads additional config after local config
- [ ] `./scouty.yaml` in cwd loaded as local/project config
- [ ] Local config priority: above user config, below CLI flags
- [ ] All keybindings configurable via config file
- [ ] Theme selected by name from config or `--theme` CLI flag
- [ ] Theme search: `~/.scouty/themes/` → `/etc/scouty/themes/` → built-in
- [ ] Partial config files work (missing fields use defaults)
- [ ] Invalid config: warn to stderr and continue with defaults
- [ ] All existing tests pass
- [ ] `default_paths` in config loaded when no CLI files specified
- [ ] Glob patterns expanded correctly (*, **, ?)
- [ ] CLI file arguments take priority over `default_paths`
- [ ] Non-matching globs silently skipped

## Change Log

| Date | Change |
|------|--------|
| 2026-02-22 | Initial configuration system design |
| 2026-02-22 | Moved theme details to theme.md, kept references |
| 2026-02-23 | Added default_paths with glob support |
| 2026-02-23 | Multi-profile config: built-in → /etc/scouty → ~/.scouty → CLI layered override |
| 2026-02-24 | Added ./scouty.yaml local/project config (priority below CLI, above user config) |
