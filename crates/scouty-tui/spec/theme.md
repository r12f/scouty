# Theme

## Overview

Centralized color and style management for the TUI. All UI colors are defined in a single `Theme` struct, enabling consistent styling and customization via `~/.scouty/themes/` YAML files.

## Design

### Theme Struct

A `Theme` struct in `theme.rs` holds all color definitions, grouped by UI area:

```
Theme
├── log_levels        # Fatal, Error, Warn, Notice, Info, Debug, Trace
├── table             # header bg/fg/bold, selected row, alternating row bg, separator (color + char)
├── status_bar        # line1 bg/fg, line2 bg/fg, mode label, density chart, density label, position, cursor marker
├── search            # match highlight, current match
├── filter            # active indicator, error text
├── dialog            # border, title, selected/unselected items, muted
├── detail_panel      # field name, field value, separator
├── panel_tab         # focused tab, unfocused tab, tab bar background
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
  header: { fg: "#1B2838", bg: "#4FC3F7", bold: true }   # Accent color bg — matches panel_tab.focused style when log table has focus
  header_unfocused: { fg: "#6B7B8D", bg: "#1B2838" }     # Muted/gray — when focus is on a panel (matches panel_tab.unfocused style)
  selected: { bg: "#2A3F55" }       # Steel blue highlight — visible but not harsh
  alternating: { bg: "#0D1117" }    # Very subtle dark shade (GitHub dark style)
  separator: { fg: "#3B4252", char: "│" }  # Muted separator — color and character are both themeable

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

panel_tab:
  focused: { fg: "#1B2838", bg: "#4FC3F7", bold: true }   # Accent color bg — panel has keyboard focus
  unfocused: { fg: "#6B7B8D", bg: "#1B2838" }             # Muted/gray — panel does not have focus
  bar_bg: { bg: "#0D1117" }                                # Tab bar background

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

> Each cell shows `fg / bg` hex values with a color swatch for quick visual reference.

#### Log Levels

| Area | default | dark | light | solarized | landmine |
|------|---------|------|-------|-----------|----------|
| FATAL fg | ![](https://placehold.co/16x16/FF0000/FF0000.png) `red` bold | ![](https://placehold.co/16x16/CC0000/CC0000.png) `#CC0000` bold | ![](https://placehold.co/16x16/CC0000/CC0000.png) `#CC0000` bold | ![](https://placehold.co/16x16/DC322F/DC322F.png) `#DC322F` bold | ![](https://placehold.co/16x16/FF3366/FF3366.png) `#FF3366` bold |
| ERROR fg | ![](https://placehold.co/16x16/FF6B6B/FF6B6B.png) `#FF6B6B` | ![](https://placehold.co/16x16/CC6666/CC6666.png) `#CC6666` | ![](https://placehold.co/16x16/CC0000/CC0000.png) `#CC0000` | ![](https://placehold.co/16x16/DC322F/DC322F.png) `#DC322F` | ![](https://placehold.co/16x16/E8577E/E8577E.png) `#E8577E` |
| WARN fg | ![](https://placehold.co/16x16/FFD93D/FFD93D.png) `#FFD93D` | ![](https://placehold.co/16x16/CCAA33/CCAA33.png) `#CCAA33` | ![](https://placehold.co/16x16/B58900/B58900.png) `#B58900` | ![](https://placehold.co/16x16/B58900/B58900.png) `#B58900` | ![](https://placehold.co/16x16/F5A0C0/F5A0C0.png) `#F5A0C0` |
| NOTICE fg | ![](https://placehold.co/16x16/6BCB77/6BCB77.png) `#6BCB77` | ![](https://placehold.co/16x16/5A9A65/5A9A65.png) `#5A9A65` | ![](https://placehold.co/16x16/2AA198/2AA198.png) `#2AA198` | ![](https://placehold.co/16x16/2AA198/2AA198.png) `#2AA198` | ![](https://placehold.co/16x16/D4A0B9/D4A0B9.png) `#D4A0B9` |
| INFO fg | ![](https://placehold.co/16x16/4FC3F7/4FC3F7.png) `#4FC3F7` | ![](https://placehold.co/16x16/6699CC/6699CC.png) `#6699CC` | ![](https://placehold.co/16x16/268BD2/268BD2.png) `#268BD2` | ![](https://placehold.co/16x16/268BD2/268BD2.png) `#268BD2` | ![](https://placehold.co/16x16/C8C8C8/C8C8C8.png) `#C8C8C8` |
| DEBUG fg | ![](https://placehold.co/16x16/8B8B8B/8B8B8B.png) `#8B8B8B` | ![](https://placehold.co/16x16/666666/666666.png) `#666666` | ![](https://placehold.co/16x16/888888/888888.png) `#888888` | ![](https://placehold.co/16x16/839496/839496.png) `#839496` | ![](https://placehold.co/16x16/6B5B6B/6B5B6B.png) `#6B5B6B` |
| TRACE fg | ![](https://placehold.co/16x16/5C5C5C/5C5C5C.png) `#5C5C5C` | ![](https://placehold.co/16x16/444444/444444.png) `#444444` | ![](https://placehold.co/16x16/AAAAAA/AAAAAA.png) `#AAAAAA` | ![](https://placehold.co/16x16/657B83/657B83.png) `#657B83` | ![](https://placehold.co/16x16/4A3A4A/4A3A4A.png) `#4A3A4A` |

#### Table

| Area | default | dark | light | solarized | landmine |
|------|---------|------|-------|-----------|----------|
| Header fg | ![](https://placehold.co/16x16/1B2838/1B2838.png) `#1B2838` bold | ![](https://placehold.co/16x16/0D0D0D/0D0D0D.png) `#0D0D0D` bold | ![](https://placehold.co/16x16/FFFFFF/FFFFFF.png) `white` bold | ![](https://placehold.co/16x16/FDF6E3/FDF6E3.png) `#FDF6E3` bold | ![](https://placehold.co/16x16/0D060B/0D060B.png) `#0D060B` bold |
| Header bg | ![](https://placehold.co/16x16/4FC3F7/4FC3F7.png) `#4FC3F7` | ![](https://placehold.co/16x16/6699CC/6699CC.png) `#6699CC` | ![](https://placehold.co/16x16/268BD2/268BD2.png) `#268BD2` | ![](https://placehold.co/16x16/268BD2/268BD2.png) `#268BD2` | ![](https://placehold.co/16x16/E8577E/E8577E.png) `#E8577E` |
| Header unfocused fg/bg | ![](https://placehold.co/16x16/6B7B8D/6B7B8D.png) `#6B7B8D` / ![](https://placehold.co/16x16/1B2838/1B2838.png) `#1B2838` | ![](https://placehold.co/16x16/555555/555555.png) `#555555` / ![](https://placehold.co/16x16/1A1A1A/1A1A1A.png) `#1A1A1A` | ![](https://placehold.co/16x16/999999/999999.png) `#999999` / ![](https://placehold.co/16x16/E8E8E8/E8E8E8.png) `#E8E8E8` | ![](https://placehold.co/16x16/657B83/657B83.png) `#657B83` / ![](https://placehold.co/16x16/073642/073642.png) `#073642` | ![](https://placehold.co/16x16/6B4A5E/6B4A5E.png) `#6B4A5E` / ![](https://placehold.co/16x16/1A0A14/1A0A14.png) `#1A0A14` |
| Selected bg | ![](https://placehold.co/16x16/2A3F55/2A3F55.png) `#2A3F55` | ![](https://placehold.co/16x16/2A2A2A/2A2A2A.png) `#2A2A2A` | ![](https://placehold.co/16x16/D0E4F7/D0E4F7.png) `#D0E4F7` | ![](https://placehold.co/16x16/073642/073642.png) `#073642` | ![](https://placehold.co/16x16/2D1028/2D1028.png) `#2D1028` |
| Alternating bg | ![](https://placehold.co/16x16/0D1117/0D1117.png) `#0D1117` | ![](https://placehold.co/16x16/111111/111111.png) `#111111` | ![](https://placehold.co/16x16/F8F8F8/F8F8F8.png) `#F8F8F8` | ![](https://placehold.co/16x16/002B36/002B36.png) `#002B36` | ![](https://placehold.co/16x16/0D060B/0D060B.png) `#0D060B` |
| Separator fg | ![](https://placehold.co/16x16/3B4252/3B4252.png) `#3B4252` | ![](https://placehold.co/16x16/333333/333333.png) `#333333` | ![](https://placehold.co/16x16/CCCCCC/CCCCCC.png) `#CCCCCC` | ![](https://placehold.co/16x16/586E75/586E75.png) `#586E75` | ![](https://placehold.co/16x16/4A2040/4A2040.png) `#4A2040` |
| Separator char | `│` | `│` | `│` | `│` | `♡` |

#### Status Bar

| Area | default | dark | light | solarized | landmine |
|------|---------|------|-------|-----------|----------|
| Line 1 fg | ![](https://placehold.co/16x16/D4D4D4/D4D4D4.png) `#D4D4D4` | ![](https://placehold.co/16x16/AAAAAA/AAAAAA.png) `#AAAAAA` | ![](https://placehold.co/16x16/333333/333333.png) `#333333` | ![](https://placehold.co/16x16/839496/839496.png) `#839496` | ![](https://placehold.co/16x16/D4A0B9/D4A0B9.png) `#D4A0B9` |
| Line 1 bg | ![](https://placehold.co/16x16/1B2838/1B2838.png) `#1B2838` | ![](https://placehold.co/16x16/1A1A1A/1A1A1A.png) `#1A1A1A` | ![](https://placehold.co/16x16/E0E0E0/E0E0E0.png) `#E0E0E0` | ![](https://placehold.co/16x16/073642/073642.png) `#073642` | ![](https://placehold.co/16x16/1A0A14/1A0A14.png) `#1A0A14` |
| Line 2 fg | ![](https://placehold.co/16x16/A0A0A0/A0A0A0.png) `#A0A0A0` | ![](https://placehold.co/16x16/777777/777777.png) `#777777` | ![](https://placehold.co/16x16/555555/555555.png) `#555555` | ![](https://placehold.co/16x16/657B83/657B83.png) `#657B83` | ![](https://placehold.co/16x16/8A6A7E/8A6A7E.png) `#8A6A7E` |
| Line 2 bg | ![](https://placehold.co/16x16/0D1117/0D1117.png) `#0D1117` | ![](https://placehold.co/16x16/0D0D0D/0D0D0D.png) `#0D0D0D` | ![](https://placehold.co/16x16/F0F0F0/F0F0F0.png) `#F0F0F0` | ![](https://placehold.co/16x16/002B36/002B36.png) `#002B36` | ![](https://placehold.co/16x16/0D060B/0D060B.png) `#0D060B` |
| Mode label fg/bg | ![](https://placehold.co/16x16/1B2838/1B2838.png) `#1B2838` / ![](https://placehold.co/16x16/4FC3F7/4FC3F7.png) `#4FC3F7` bold | ![](https://placehold.co/16x16/0D0D0D/0D0D0D.png) `#0D0D0D` / ![](https://placehold.co/16x16/6699CC/6699CC.png) `#6699CC` bold | ![](https://placehold.co/16x16/FFFFFF/FFFFFF.png) `white` / ![](https://placehold.co/16x16/268BD2/268BD2.png) `#268BD2` bold | ![](https://placehold.co/16x16/FDF6E3/FDF6E3.png) `#FDF6E3` / ![](https://placehold.co/16x16/268BD2/268BD2.png) `#268BD2` bold | ![](https://placehold.co/16x16/0D060B/0D060B.png) `#0D060B` / ![](https://placehold.co/16x16/E8577E/E8577E.png) `#E8577E` bold |
| Density chart fg | ![](https://placehold.co/16x16/4FC3F7/4FC3F7.png) `#4FC3F7` | ![](https://placehold.co/16x16/6699CC/6699CC.png) `#6699CC` | ![](https://placehold.co/16x16/268BD2/268BD2.png) `#268BD2` | ![](https://placehold.co/16x16/268BD2/268BD2.png) `#268BD2` | ![](https://placehold.co/16x16/E8577E/E8577E.png) `#E8577E` |
| Density label fg | ![](https://placehold.co/16x16/6B7B8D/6B7B8D.png) `#6B7B8D` | ![](https://placehold.co/16x16/555555/555555.png) `#555555` | ![](https://placehold.co/16x16/888888/888888.png) `#888888` | ![](https://placehold.co/16x16/657B83/657B83.png) `#657B83` | ![](https://placehold.co/16x16/6B4A5E/6B4A5E.png) `#6B4A5E` |
| Position fg | ![](https://placehold.co/16x16/E8E8E8/E8E8E8.png) `#E8E8E8` | ![](https://placehold.co/16x16/CCCCCC/CCCCCC.png) `#CCCCCC` | ![](https://placehold.co/16x16/222222/222222.png) `#222222` | ![](https://placehold.co/16x16/93A1A1/93A1A1.png) `#93A1A1` | ![](https://placehold.co/16x16/F5D0E0/F5D0E0.png) `#F5D0E0` |
| Cursor marker fg | ![](https://placehold.co/16x16/FFD93D/FFD93D.png) `#FFD93D` | ![](https://placehold.co/16x16/CCAA33/CCAA33.png) `#CCAA33` | ![](https://placehold.co/16x16/B58900/B58900.png) `#B58900` | ![](https://placehold.co/16x16/B58900/B58900.png) `#B58900` | ![](https://placehold.co/16x16/FF3366/FF3366.png) `#FF3366` |

#### Search & Dialogs

| Area | default | dark | light | solarized | landmine |
|------|---------|------|-------|-----------|----------|
| Match fg/bg | `black` / ![](https://placehold.co/16x16/FFD93D/FFD93D.png) `#FFD93D` | `black` / ![](https://placehold.co/16x16/CCAA33/CCAA33.png) `#CCAA33` | `black` / ![](https://placehold.co/16x16/FFD93D/FFD93D.png) `#FFD93D` | `black` / ![](https://placehold.co/16x16/B58900/B58900.png) `#B58900` | `black` / ![](https://placehold.co/16x16/F5A0C0/F5A0C0.png) `#F5A0C0` |
| Current match fg/bg | `black` / ![](https://placehold.co/16x16/FF8C42/FF8C42.png) `#FF8C42` | `black` / ![](https://placehold.co/16x16/CC7733/CC7733.png) `#CC7733` | `black` / ![](https://placehold.co/16x16/FF8C42/FF8C42.png) `#FF8C42` | `black` / ![](https://placehold.co/16x16/CB4B16/CB4B16.png) `#CB4B16` | `black` / ![](https://placehold.co/16x16/FF3366/FF3366.png) `#FF3366` |
| Dialog border fg | ![](https://placehold.co/16x16/4FC3F7/4FC3F7.png) `#4FC3F7` | ![](https://placehold.co/16x16/6699CC/6699CC.png) `#6699CC` | ![](https://placehold.co/16x16/268BD2/268BD2.png) `#268BD2` | ![](https://placehold.co/16x16/268BD2/268BD2.png) `#268BD2` | ![](https://placehold.co/16x16/E8577E/E8577E.png) `#E8577E` |
| Input prompt fg | ![](https://placehold.co/16x16/FFD93D/FFD93D.png) `#FFD93D` | ![](https://placehold.co/16x16/CCAA33/CCAA33.png) `#CCAA33` | ![](https://placehold.co/16x16/B58900/B58900.png) `#B58900` | ![](https://placehold.co/16x16/B58900/B58900.png) `#B58900` | ![](https://placehold.co/16x16/F5A0C0/F5A0C0.png) `#F5A0C0` |

#### Panel Tab

| Area | default | dark | light | solarized | landmine |
|------|---------|------|-------|-----------|----------|
| Focused fg/bg | ![](https://placehold.co/16x16/1B2838/1B2838.png) `#1B2838` / ![](https://placehold.co/16x16/4FC3F7/4FC3F7.png) `#4FC3F7` bold | ![](https://placehold.co/16x16/0D0D0D/0D0D0D.png) `#0D0D0D` / ![](https://placehold.co/16x16/6699CC/6699CC.png) `#6699CC` bold | ![](https://placehold.co/16x16/FFFFFF/FFFFFF.png) `white` / ![](https://placehold.co/16x16/268BD2/268BD2.png) `#268BD2` bold | ![](https://placehold.co/16x16/FDF6E3/FDF6E3.png) `#FDF6E3` / ![](https://placehold.co/16x16/268BD2/268BD2.png) `#268BD2` bold | ![](https://placehold.co/16x16/0D060B/0D060B.png) `#0D060B` / ![](https://placehold.co/16x16/E8577E/E8577E.png) `#E8577E` bold |
| Unfocused fg/bg | ![](https://placehold.co/16x16/6B7B8D/6B7B8D.png) `#6B7B8D` / ![](https://placehold.co/16x16/1B2838/1B2838.png) `#1B2838` | ![](https://placehold.co/16x16/555555/555555.png) `#555555` / ![](https://placehold.co/16x16/1A1A1A/1A1A1A.png) `#1A1A1A` | ![](https://placehold.co/16x16/999999/999999.png) `#999999` / ![](https://placehold.co/16x16/E8E8E8/E8E8E8.png) `#E8E8E8` | ![](https://placehold.co/16x16/657B83/657B83.png) `#657B83` / ![](https://placehold.co/16x16/073642/073642.png) `#073642` | ![](https://placehold.co/16x16/6B4A5E/6B4A5E.png) `#6B4A5E` / ![](https://placehold.co/16x16/1A0A14/1A0A14.png) `#1A0A14` |
| Bar bg | ![](https://placehold.co/16x16/0D1117/0D1117.png) `#0D1117` | ![](https://placehold.co/16x16/0D0D0D/0D0D0D.png) `#0D0D0D` | ![](https://placehold.co/16x16/F0F0F0/F0F0F0.png) `#F0F0F0` | ![](https://placehold.co/16x16/002B36/002B36.png) `#002B36` | ![](https://placehold.co/16x16/0D060B/0D060B.png) `#0D060B` |

#### General & Accents

| Area | default | dark | light | solarized | landmine |
|------|---------|------|-------|-----------|----------|
| Accent fg | ![](https://placehold.co/16x16/4FC3F7/4FC3F7.png) `#4FC3F7` | ![](https://placehold.co/16x16/6699CC/6699CC.png) `#6699CC` | ![](https://placehold.co/16x16/268BD2/268BD2.png) `#268BD2` | ![](https://placehold.co/16x16/268BD2/268BD2.png) `#268BD2` | ![](https://placehold.co/16x16/E8577E/E8577E.png) `#E8577E` |
| Border fg | ![](https://placehold.co/16x16/3B4252/3B4252.png) `#3B4252` | ![](https://placehold.co/16x16/333333/333333.png) `#333333` | ![](https://placehold.co/16x16/CCCCCC/CCCCCC.png) `#CCCCCC` | ![](https://placehold.co/16x16/586E75/586E75.png) `#586E75` | ![](https://placehold.co/16x16/3D1A30/3D1A30.png) `#3D1A30` |
| Muted fg | ![](https://placehold.co/16x16/6B7B8D/6B7B8D.png) `#6B7B8D` | ![](https://placehold.co/16x16/555555/555555.png) `#555555` | ![](https://placehold.co/16x16/999999/999999.png) `#999999` | ![](https://placehold.co/16x16/657B83/657B83.png) `#657B83` | ![](https://placehold.co/16x16/6B4A5E/6B4A5E.png) `#6B4A5E` |

#### Highlight Palette (color rotation order)

| # | default | dark | light | solarized | landmine |
|---|---------|------|-------|-----------|----------|
| 1 | ![](https://placehold.co/16x16/FF6B6B/FF6B6B.png) `#FF6B6B` | ![](https://placehold.co/16x16/CC6666/CC6666.png) `#CC6666` | ![](https://placehold.co/16x16/DC322F/DC322F.png) `#DC322F` | ![](https://placehold.co/16x16/DC322F/DC322F.png) `#DC322F` | ![](https://placehold.co/16x16/FF3366/FF3366.png) `#FF3366` |
| 2 | ![](https://placehold.co/16x16/6BCB77/6BCB77.png) `#6BCB77` | ![](https://placehold.co/16x16/5A9A65/5A9A65.png) `#5A9A65` | ![](https://placehold.co/16x16/2AA198/2AA198.png) `#2AA198` | ![](https://placehold.co/16x16/2AA198/2AA198.png) `#2AA198` | ![](https://placehold.co/16x16/F5A0C0/F5A0C0.png) `#F5A0C0` |
| 3 | ![](https://placehold.co/16x16/4FC3F7/4FC3F7.png) `#4FC3F7` | ![](https://placehold.co/16x16/6699CC/6699CC.png) `#6699CC` | ![](https://placehold.co/16x16/268BD2/268BD2.png) `#268BD2` | ![](https://placehold.co/16x16/268BD2/268BD2.png) `#268BD2` | ![](https://placehold.co/16x16/D4A0B9/D4A0B9.png) `#D4A0B9` |
| 4 | ![](https://placehold.co/16x16/FFD93D/FFD93D.png) `#FFD93D` | ![](https://placehold.co/16x16/CCAA33/CCAA33.png) `#CCAA33` | ![](https://placehold.co/16x16/B58900/B58900.png) `#B58900` | ![](https://placehold.co/16x16/B58900/B58900.png) `#B58900` | ![](https://placehold.co/16x16/E8577E/E8577E.png) `#E8577E` |
| 5 | ![](https://placehold.co/16x16/CE93D8/CE93D8.png) `#CE93D8` | ![](https://placehold.co/16x16/9977AA/9977AA.png) `#9977AA` | ![](https://placehold.co/16x16/6C71C4/6C71C4.png) `#6C71C4` | ![](https://placehold.co/16x16/6C71C4/6C71C4.png) `#6C71C4` | ![](https://placehold.co/16x16/F5D0E0/F5D0E0.png) `#F5D0E0` |
| 6 | ![](https://placehold.co/16x16/4DD0E1/4DD0E1.png) `#4DD0E1` | ![](https://placehold.co/16x16/5599AA/5599AA.png) `#5599AA` | ![](https://placehold.co/16x16/2AA198/2AA198.png) `#2AA198` | ![](https://placehold.co/16x16/2AA198/2AA198.png) `#2AA198` | ![](https://placehold.co/16x16/8A6A7E/8A6A7E.png) `#8A6A7E` |

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
| 2026-02-24 | Added `landmine` theme (地雷系 Jirai Kei: black + pink); separator char now themeable |
| 2026-02-28 | Added `panel_tab` theme section: focused (accent), unfocused (gray) styles for all 5 presets |
| 2026-02-28 | Added `table.header_unfocused`: table header goes gray when focus moves to panel |
| 2026-02-28 | Table header focused color now matches `panel_tab.focused` (accent bg) for consistent focus visual language |
