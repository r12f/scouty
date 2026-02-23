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

### Built-in Presets — Color Reference

> Each cell shows `fg / bg` hex values. ![color](https://via.placeholder.com/12/HEX/HEX) swatch for quick visual reference.

#### Log Levels

| Area | default | dark | light | solarized |
|------|---------|------|-------|-----------|
| FATAL fg | ![](https://via.placeholder.com/12/FF0000/FF0000) `red` bold | ![](https://via.placeholder.com/12/CC0000/CC0000) `#CC0000` bold | ![](https://via.placeholder.com/12/CC0000/CC0000) `#CC0000` bold | ![](https://via.placeholder.com/12/DC322F/DC322F) `#DC322F` bold |
| ERROR fg | ![](https://via.placeholder.com/12/FF6B6B/FF6B6B) `#FF6B6B` | ![](https://via.placeholder.com/12/CC6666/CC6666) `#CC6666` | ![](https://via.placeholder.com/12/CC0000/CC0000) `#CC0000` | ![](https://via.placeholder.com/12/DC322F/DC322F) `#DC322F` |
| WARN fg | ![](https://via.placeholder.com/12/FFD93D/FFD93D) `#FFD93D` | ![](https://via.placeholder.com/12/CCAA33/CCAA33) `#CCAA33` | ![](https://via.placeholder.com/12/B58900/B58900) `#B58900` | ![](https://via.placeholder.com/12/B58900/B58900) `#B58900` |
| NOTICE fg | ![](https://via.placeholder.com/12/6BCB77/6BCB77) `#6BCB77` | ![](https://via.placeholder.com/12/5A9A65/5A9A65) `#5A9A65` | ![](https://via.placeholder.com/12/2AA198/2AA198) `#2AA198` | ![](https://via.placeholder.com/12/2AA198/2AA198) `#2AA198` |
| INFO fg | ![](https://via.placeholder.com/12/4FC3F7/4FC3F7) `#4FC3F7` | ![](https://via.placeholder.com/12/6699CC/6699CC) `#6699CC` | ![](https://via.placeholder.com/12/268BD2/268BD2) `#268BD2` | ![](https://via.placeholder.com/12/268BD2/268BD2) `#268BD2` |
| DEBUG fg | ![](https://via.placeholder.com/12/8B8B8B/8B8B8B) `#8B8B8B` | ![](https://via.placeholder.com/12/666666/666666) `#666666` | ![](https://via.placeholder.com/12/888888/888888) `#888888` | ![](https://via.placeholder.com/12/839496/839496) `#839496` |
| TRACE fg | ![](https://via.placeholder.com/12/5C5C5C/5C5C5C) `#5C5C5C` | ![](https://via.placeholder.com/12/444444/444444) `#444444` | ![](https://via.placeholder.com/12/AAAAAA/AAAAAA) `#AAAAAA` | ![](https://via.placeholder.com/12/657B83/657B83) `#657B83` |

#### Table

| Area | default | dark | light | solarized |
|------|---------|------|-------|-----------|
| Header fg | ![](https://via.placeholder.com/12/B8C4CE/B8C4CE) `#B8C4CE` bold | ![](https://via.placeholder.com/12/999999/999999) `#999999` bold | ![](https://via.placeholder.com/12/333333/333333) `#333333` bold | ![](https://via.placeholder.com/12/586E75/586E75) `#586E75` bold |
| Header bg | ![](https://via.placeholder.com/12/1E2A38/1E2A38) `#1E2A38` | ![](https://via.placeholder.com/12/1A1A1A/1A1A1A) `#1A1A1A` | ![](https://via.placeholder.com/12/E8E8E8/E8E8E8) `#E8E8E8` | ![](https://via.placeholder.com/12/073642/073642) `#073642` |
| Selected bg | ![](https://via.placeholder.com/12/2A3F55/2A3F55) `#2A3F55` | ![](https://via.placeholder.com/12/2A2A2A/2A2A2A) `#2A2A2A` | ![](https://via.placeholder.com/12/D0E4F7/D0E4F7) `#D0E4F7` | ![](https://via.placeholder.com/12/073642/073642) `#073642` |
| Alternating bg | ![](https://via.placeholder.com/12/0D1117/0D1117) `#0D1117` | ![](https://via.placeholder.com/12/111111/111111) `#111111` | ![](https://via.placeholder.com/12/F8F8F8/F8F8F8) `#F8F8F8` | ![](https://via.placeholder.com/12/002B36/002B36) `#002B36` |
| Separator fg | ![](https://via.placeholder.com/12/3B4252/3B4252) `#3B4252` | ![](https://via.placeholder.com/12/333333/333333) `#333333` | ![](https://via.placeholder.com/12/CCCCCC/CCCCCC) `#CCCCCC` | ![](https://via.placeholder.com/12/586E75/586E75) `#586E75` |

#### Status Bar

| Area | default | dark | light | solarized |
|------|---------|------|-------|-----------|
| Line 1 fg | ![](https://via.placeholder.com/12/D4D4D4/D4D4D4) `#D4D4D4` | ![](https://via.placeholder.com/12/AAAAAA/AAAAAA) `#AAAAAA` | ![](https://via.placeholder.com/12/333333/333333) `#333333` | ![](https://via.placeholder.com/12/839496/839496) `#839496` |
| Line 1 bg | ![](https://via.placeholder.com/12/1B2838/1B2838) `#1B2838` | ![](https://via.placeholder.com/12/1A1A1A/1A1A1A) `#1A1A1A` | ![](https://via.placeholder.com/12/E0E0E0/E0E0E0) `#E0E0E0` | ![](https://via.placeholder.com/12/073642/073642) `#073642` |
| Line 2 fg | ![](https://via.placeholder.com/12/A0A0A0/A0A0A0) `#A0A0A0` | ![](https://via.placeholder.com/12/777777/777777) `#777777` | ![](https://via.placeholder.com/12/555555/555555) `#555555` | ![](https://via.placeholder.com/12/657B83/657B83) `#657B83` |
| Line 2 bg | ![](https://via.placeholder.com/12/0D1117/0D1117) `#0D1117` | ![](https://via.placeholder.com/12/0D0D0D/0D0D0D) `#0D0D0D` | ![](https://via.placeholder.com/12/F0F0F0/F0F0F0) `#F0F0F0` | ![](https://via.placeholder.com/12/002B36/002B36) `#002B36` |
| Mode label fg/bg | ![](https://via.placeholder.com/12/1B2838/1B2838) `#1B2838` / ![](https://via.placeholder.com/12/4FC3F7/4FC3F7) `#4FC3F7` bold | ![](https://via.placeholder.com/12/0D0D0D/0D0D0D) `#0D0D0D` / ![](https://via.placeholder.com/12/6699CC/6699CC) `#6699CC` bold | ![](https://via.placeholder.com/12/FFFFFF/FFFFFF) `white` / ![](https://via.placeholder.com/12/268BD2/268BD2) `#268BD2` bold | ![](https://via.placeholder.com/12/FDF6E3/FDF6E3) `#FDF6E3` / ![](https://via.placeholder.com/12/268BD2/268BD2) `#268BD2` bold |
| Density chart fg | ![](https://via.placeholder.com/12/4FC3F7/4FC3F7) `#4FC3F7` | ![](https://via.placeholder.com/12/6699CC/6699CC) `#6699CC` | ![](https://via.placeholder.com/12/268BD2/268BD2) `#268BD2` | ![](https://via.placeholder.com/12/268BD2/268BD2) `#268BD2` |
| Density label fg | ![](https://via.placeholder.com/12/6B7B8D/6B7B8D) `#6B7B8D` | ![](https://via.placeholder.com/12/555555/555555) `#555555` | ![](https://via.placeholder.com/12/888888/888888) `#888888` | ![](https://via.placeholder.com/12/657B83/657B83) `#657B83` |
| Position fg | ![](https://via.placeholder.com/12/E8E8E8/E8E8E8) `#E8E8E8` | ![](https://via.placeholder.com/12/CCCCCC/CCCCCC) `#CCCCCC` | ![](https://via.placeholder.com/12/222222/222222) `#222222` | ![](https://via.placeholder.com/12/93A1A1/93A1A1) `#93A1A1` |
| Cursor marker fg | ![](https://via.placeholder.com/12/FFD93D/FFD93D) `#FFD93D` | ![](https://via.placeholder.com/12/CCAA33/CCAA33) `#CCAA33` | ![](https://via.placeholder.com/12/B58900/B58900) `#B58900` | ![](https://via.placeholder.com/12/B58900/B58900) `#B58900` |

#### Search & Dialogs

| Area | default | dark | light | solarized |
|------|---------|------|-------|-----------|
| Match fg/bg | `black` / ![](https://via.placeholder.com/12/FFD93D/FFD93D) `#FFD93D` | `black` / ![](https://via.placeholder.com/12/CCAA33/CCAA33) `#CCAA33` | `black` / ![](https://via.placeholder.com/12/FFD93D/FFD93D) `#FFD93D` | `black` / ![](https://via.placeholder.com/12/B58900/B58900) `#B58900` |
| Current match fg/bg | `black` / ![](https://via.placeholder.com/12/FF8C42/FF8C42) `#FF8C42` | `black` / ![](https://via.placeholder.com/12/CC7733/CC7733) `#CC7733` | `black` / ![](https://via.placeholder.com/12/FF8C42/FF8C42) `#FF8C42` | `black` / ![](https://via.placeholder.com/12/CB4B16/CB4B16) `#CB4B16` |
| Dialog border fg | ![](https://via.placeholder.com/12/4FC3F7/4FC3F7) `#4FC3F7` | ![](https://via.placeholder.com/12/6699CC/6699CC) `#6699CC` | ![](https://via.placeholder.com/12/268BD2/268BD2) `#268BD2` | ![](https://via.placeholder.com/12/268BD2/268BD2) `#268BD2` |
| Input prompt fg | ![](https://via.placeholder.com/12/FFD93D/FFD93D) `#FFD93D` | ![](https://via.placeholder.com/12/CCAA33/CCAA33) `#CCAA33` | ![](https://via.placeholder.com/12/B58900/B58900) `#B58900` | ![](https://via.placeholder.com/12/B58900/B58900) `#B58900` |

#### General & Accents

| Area | default | dark | light | solarized |
|------|---------|------|-------|-----------|
| Accent fg | ![](https://via.placeholder.com/12/4FC3F7/4FC3F7) `#4FC3F7` | ![](https://via.placeholder.com/12/6699CC/6699CC) `#6699CC` | ![](https://via.placeholder.com/12/268BD2/268BD2) `#268BD2` | ![](https://via.placeholder.com/12/268BD2/268BD2) `#268BD2` |
| Border fg | ![](https://via.placeholder.com/12/3B4252/3B4252) `#3B4252` | ![](https://via.placeholder.com/12/333333/333333) `#333333` | ![](https://via.placeholder.com/12/CCCCCC/CCCCCC) `#CCCCCC` | ![](https://via.placeholder.com/12/586E75/586E75) `#586E75` |
| Muted fg | ![](https://via.placeholder.com/12/6B7B8D/6B7B8D) `#6B7B8D` | ![](https://via.placeholder.com/12/555555/555555) `#555555` | ![](https://via.placeholder.com/12/999999/999999) `#999999` | ![](https://via.placeholder.com/12/657B83/657B83) `#657B83` |

#### Highlight Palette (color rotation order)

| # | default | dark | light | solarized |
|---|---------|------|-------|-----------|
| 1 | ![](https://via.placeholder.com/12/FF6B6B/FF6B6B) `#FF6B6B` | ![](https://via.placeholder.com/12/CC6666/CC6666) `#CC6666` | ![](https://via.placeholder.com/12/DC322F/DC322F) `#DC322F` | ![](https://via.placeholder.com/12/DC322F/DC322F) `#DC322F` |
| 2 | ![](https://via.placeholder.com/12/6BCB77/6BCB77) `#6BCB77` | ![](https://via.placeholder.com/12/5A9A65/5A9A65) `#5A9A65` | ![](https://via.placeholder.com/12/2AA198/2AA198) `#2AA198` | ![](https://via.placeholder.com/12/2AA198/2AA198) `#2AA198` |
| 3 | ![](https://via.placeholder.com/12/4FC3F7/4FC3F7) `#4FC3F7` | ![](https://via.placeholder.com/12/6699CC/6699CC) `#6699CC` | ![](https://via.placeholder.com/12/268BD2/268BD2) `#268BD2` | ![](https://via.placeholder.com/12/268BD2/268BD2) `#268BD2` |
| 4 | ![](https://via.placeholder.com/12/FFD93D/FFD93D) `#FFD93D` | ![](https://via.placeholder.com/12/CCAA33/CCAA33) `#CCAA33` | ![](https://via.placeholder.com/12/B58900/B58900) `#B58900` | ![](https://via.placeholder.com/12/B58900/B58900) `#B58900` |
| 5 | ![](https://via.placeholder.com/12/CE93D8/CE93D8) `#CE93D8` | ![](https://via.placeholder.com/12/9977AA/9977AA) `#9977AA` | ![](https://via.placeholder.com/12/6C71C4/6C71C4) `#6C71C4` | ![](https://via.placeholder.com/12/6C71C4/6C71C4) `#6C71C4` |
| 6 | ![](https://via.placeholder.com/12/4DD0E1/4DD0E1) `#4DD0E1` | ![](https://via.placeholder.com/12/5599AA/5599AA) `#5599AA` | ![](https://via.placeholder.com/12/2AA198/2AA198) `#2AA198` | ![](https://via.placeholder.com/12/2AA198/2AA198) `#2AA198` |

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
| 2026-02-23 | Replace text descriptions with color swatch tables for all 4 built-in presets |
