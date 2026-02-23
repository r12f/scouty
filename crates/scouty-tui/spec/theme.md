# Theme

## Overview

Centralized color and style management for the TUI. All UI colors are defined in a single `Theme` struct, enabling consistent styling and customization via `~/.scouty/themes/` YAML files.

## Design

### Theme Struct

A `Theme` struct in `theme.rs` holds all color definitions, grouped by UI area:

```
Theme
├── log_levels        # Fatal, Error, Warn, Notice, Info, Debug, Trace
├── table             # header bg/fg/bold, selected row, alternating row bg, separator
├── status_bar        # line1 bg/fg, line2 bg/fg, mode label, density chart, density label, position, cursor marker
├── search            # match highlight, current match
├── filter            # active indicator, error text
├── dialog            # border, title, selected/unselected items, muted
├── detail_panel      # field name, field value, separator
├── input             # prompt, cursor, text, background, error
├── highlight_palette # color rotation for user highlight rules
└── general           # accent, muted, border
```

### Theme File Format (`~/.scouty/themes/<name>.yaml`)

```yaml
log_levels:
  fatal: { fg: "red", bold: true }
  error: { fg: "#FF6B6B" }          # Soft red, easy on the eyes
  warn: { fg: "#FFD93D" }           # Warm yellow
  notice: { fg: "#6BCB77" }         # Soft green (distinct from info)
  info: { fg: "#4FC3F7" }           # Light blue
  debug: { fg: "#8B8B8B" }          # Medium gray
  trace: { fg: "#5C5C5C" }          # Dark gray

table:
  header: { fg: "#B8C4CE", bg: "#1E2A38", bold: true }   # Light steel text on dark slate — clearly distinct from rows
  selected: { bg: "#2A3F55" }       # Steel blue highlight — visible but not harsh
  alternating: { bg: "#0D1117" }    # Very subtle dark shade (GitHub dark style)
  separator: { fg: "#3B4252" }      # Muted separator, visible but not distracting

status_bar:
  line1: { fg: "#D4D4D4", bg: "#1B2838" }    # Density chart line: dark navy, light text
  line2: { fg: "#A0A0A0", bg: "#0D1117" }    # Mode/shortcut line: near-black, dimmer text — clearly different from line1
  mode_label: { fg: "#1B2838", bg: "#4FC3F7", bold: true }  # Light blue badge, dark text — pops out
  density_chart: { fg: "#4FC3F7" }            # Light blue braille, matches accent
  density_label: { fg: "#6B7B8D" }            # Dimmer than chart — visually secondary
  position: { fg: "#E8E8E8" }                 # Bright white for record count
  cursor_marker: { fg: "#FFD93D" }            # Yellow cursor position in density chart

search:
  match: { fg: "black", bg: "#FFD93D" }       # Yellow highlight
  current_match: { fg: "black", bg: "#FF8C42" }  # Orange for current match

dialog:
  border: { fg: "#4FC3F7" }         # Light blue border
  title: { fg: "white", bold: true }
  selected: { fg: "white", bg: "#2A3F55" }
  text: { fg: "#D4D4D4" }
  muted: { fg: "#6B7B8D" }

detail_panel:
  field_name: { fg: "#4FC3F7" }     # Light blue labels
  field_value: { fg: "#D4D4D4" }
  separator: { fg: "#3B4252" }

input:
  prompt: { fg: "#FFD93D" }         # Yellow prompt label
  text: { fg: "white" }
  cursor: { fg: "white" }
  error: { fg: "#FF6B6B" }
  background: { bg: "#1B2838" }

highlight_palette:
  - "#FF6B6B"     # Soft red
  - "#6BCB77"     # Green
  - "#4FC3F7"     # Light blue
  - "#FFD93D"     # Yellow
  - "#CE93D8"     # Lavender
  - "#4DD0E1"     # Teal

general:
  border: { fg: "#3B4252" }         # Nord-inspired muted border
  accent: { fg: "#4FC3F7" }         # Light blue accent
  muted: { fg: "#6B7B8D" }          # Readable muted text
```

### Color Value Formats

- **Named colors**: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `gray`, `dark_gray`
- **Hex RGB**: `"#RRGGBB"` (e.g., `"#FF6600"`)
- **256-color index**: `color(123)` (terminal 256-color palette)

### Default Theme

Clean dark theme with blue accents and clear visual hierarchy:

- **Status bar**: Two lines with **distinct** backgrounds — line 1 (density chart) is dark navy `#1B2838`, line 2 (mode/shortcuts) is near-black `#0D1117`. Mode label is a bold light-blue badge that pops out.
- **Table header**: Light steel text on dark slate with bold — clearly distinguishable from data rows
- **Selected row**: Steel blue highlight `#2A3F55` — visible but not eye-straining
- **Log levels**: Soft, pastel-leaning colors — red for errors, warm yellow for warn, light blue for info, soft green for notice
- **Borders & accents**: Light blue `#4FC3F7` (not pure cyan — easier on the eyes)
- **Dialogs**: Light blue borders, clear selection highlight
- **Input prompts**: Yellow labels, white text
- **Muted text**: `#6B7B8D` — readable, not invisible

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
| 2026-02-23 | Redesign default theme: distinct status bar lines, softer colors, clear visual hierarchy |
