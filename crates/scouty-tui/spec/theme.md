# Theme

## Overview

Centralized color and style management for the TUI. All UI colors are defined in a single `Theme` struct, enabling consistent styling and customization via `~/.scouty/themes/` YAML files.

## Design

### Theme Struct

A `Theme` struct in `theme.rs` holds all color definitions, grouped by UI area:

```
Theme
├── log_levels        # Fatal, Error, Warn, Notice, Info, Debug, Trace
├── table             # header bg/fg, selected row, alternating row bg
├── status_bar        # line1/line2 bg, mode label, density chart
├── search            # match highlight, current match
├── filter            # active indicator, error text
├── dialog            # border, title, selected/unselected items
├── detail_panel      # field name, field value, separator
├── input             # prompt, cursor, text, background
├── highlight_palette # color rotation for user highlight rules
└── general           # primary accent, secondary accent, muted, border
```

### Theme File Format (`~/.scouty/themes/<name>.yaml`)

```yaml
log_levels:
  fatal: { fg: "red", bold: true }
  error: { fg: "red" }
  warn: { fg: "#FFD700" }           # Gold
  notice: { fg: "cyan" }
  info: { fg: "#00CC66" }           # Rich green
  debug: { fg: "gray" }
  trace: { fg: "dark_gray" }

table:
  header: { fg: "white", bg: "#1A1A2E" }
  selected: { bg: "#16213E" }
  alternating: { bg: "#0F0F1A" }

status_bar:
  line1: { bg: "#1A1A2E" }
  line2: { bg: "#16213E" }
  mode_label: { fg: "black", bg: "cyan" }
  density_chart: { fg: "cyan" }
  position: { fg: "white" }

search:
  match: { fg: "black", bg: "yellow" }
  current_match: { fg: "black", bg: "#FF6600" }

dialog:
  border: { fg: "cyan" }
  title: { fg: "white", bold: true }
  selected: { fg: "white", bg: "#16213E" }
  text: { fg: "white" }
  muted: { fg: "dark_gray" }

detail_panel:
  field_name: { fg: "cyan" }
  field_value: { fg: "white" }
  separator: { fg: "dark_gray" }

input:
  prompt: { fg: "yellow" }
  text: { fg: "white" }
  cursor: { fg: "white" }
  error: { fg: "red" }
  background: { bg: "#1A1A2E" }

highlight_palette:
  - "red"
  - "#00CC66"
  - "#3399FF"
  - "yellow"
  - "magenta"
  - "cyan"

general:
  border: { fg: "#333366" }
  accent: { fg: "cyan" }
  muted: { fg: "dark_gray" }
```

### Color Value Formats

- **Named colors**: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `gray`, `dark_gray`
- **Hex RGB**: `"#RRGGBB"` (e.g., `"#FF6600"`)
- **256-color index**: `color(123)` (terminal 256-color palette)

### Default Theme

Vibrant dark theme with blue/cyan accents:

- **Borders & accents**: Cyan / Blue (not gray)
- **Status bar**: Dark teal background
- **Log levels**: FATAL red bold, ERROR red, WARN gold, NOTICE cyan, INFO green, DEBUG gray, TRACE dark gray
- **Selected row**: Blue highlight background
- **Alternating rows**: Subtle dark shade difference
- **Dialogs**: Cyan borders, blue title, clear selection highlight
- **Input prompts**: Yellow labels, white text

### Built-in Presets

- `default` — vibrant dark (described above)
- `dark` — muted dark
- `light` — light background
- `solarized` — solarized color scheme

### Integration

- `Theme` is created once at startup and passed by reference to all render functions
- All `Color::*` literals replaced with `theme.field` access
- Widgets accept `&Theme` parameter in their render methods
- Theme selected via `config.yaml` or `--theme` CLI flag (see config spec)

## Change Log

| Date | Change |
|------|--------|
| 2026-02-22 | Initial theme system design |
| 2026-02-22 | Moved theme file format and color details from config spec |
