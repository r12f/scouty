# Theme

## Overview

Centralized color and style management for the TUI. All UI colors are defined in a single `Theme` struct, enabling consistent styling and customization via `~/.scouty/themes/` YAML files.

## Design

### Theme Struct

A `Theme` struct in `theme.rs` holds all color definitions, grouped by UI area:

```
Theme
├── description       # one-line theme description (shown in --theme-list and --theme-dump)
├── log_levels        # Fatal, Error, Warn, Notice, Info, Debug, Trace
├── table             # header bg/fg/bold, selected row, alternating row bg, separator (color + char)
├── status_bar        # line1 bg/fg, line2 bg/fg, mode label, density chart, density label, density_tick, position, cursor marker
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
  density_tick: { fg: "#3B4252" }              # Subtle tick marks every 10 columns
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

### Theme Description Field

Each built-in theme includes a `description` field for display in `--theme-list`. Descriptions are short (one line), mentioning style inspiration and primary colors.

| Name | Description |
|------|-------------|
| mizuiro | Clear aqua: deep navy base with water blue and sky blue (default) |
| landmine | Jirai Kei: black base with pink and red accents |
| amai | Sweet Lolita: dark rose base with candy pink and lavender |
| maid | Classic maid: black and white high contrast with wine red |
| gyaru | Shibuya bold: dark bronze base with gold and hot pink |
| dopamine | Rainbow maximalist: warm black base with neon pink, sunflower yellow, electric blue |

### CLI: `--theme-list`

Prints all available themes (built-in + user custom from `~/.scouty/themes/`) and exits.

```
$ scouty --theme-list
Built-in themes:
  mizuiro      Clear aqua: deep navy base with water blue and sky blue (default)
  landmine     Jirai Kei: black base with pink and red accents
  amai         Sweet Lolita: dark rose base with candy pink and lavender
  maid         Classic maid: black and white high contrast with wine red
  gyaru        Shibuya bold: dark bronze base with gold and hot pink
  dopamine     Rainbow maximalist: warm black base with neon pink, sunflower yellow, electric blue

Custom themes (~/.scouty/themes/):
  my-custom    (loaded from my-custom.yaml)
```

Implementation notes:
- `description` is a field on `Theme` struct (serialized/deserialized via serde)
- Built-in themes set description in their constructors
- Custom themes can set `description:` in their YAML file
- `--theme-dump <name>` output includes the description field
- `--theme-list` reads description from Theme for both built-in and custom themes
- Print to stdout, then `std::process::exit(0)`

### Color Value Formats

- **Named colors**: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `gray`, `dark_gray`
- **Hex RGB**: `"#RRGGBB"` (e.g., `"#FF6600"`)
- **256-color index**: `color(123)` (terminal 256-color palette)

### Built-in Presets — Color Reference

> Each cell shows `fg / bg` hex values with a color swatch for quick visual reference.

#### Log Levels

| Area | mizuiro | landmine | amai | maid | gyaru | dopamine |
|------|------|------|------|------|------|------|
| FATAL fg | ![](https://placehold.co/16x16/E87461/E87461.png) `#E87461` bold | ![](https://placehold.co/16x16/FF3366/FF3366.png) `#FF3366` bold| ![](https://placehold.co/16x16/FF6B8A/FF6B8A.png) `#FF6B8A` bold | ![](https://placehold.co/16x16/C43060/C43060.png) `#C43060` bold | ![](https://placehold.co/16x16/FF2499/FF2499.png) `#FF2499` bold | ![](https://placehold.co/16x16/FF0000/FF0000.png) `#FF0000` bold |
| ERROR fg | ![](https://placehold.co/16x16/CF6B5E/CF6B5E.png) `#CF6B5E` | ![](https://placehold.co/16x16/E8577E/E8577E.png) `#E8577E` | ![](https://placehold.co/16x16/E85A7A/E85A7A.png) `#E85A7A` | ![](https://placehold.co/16x16/8B2252/8B2252.png) `#8B2252` | ![](https://placehold.co/16x16/FF69B4/FF69B4.png) `#FF69B4` | ![](https://placehold.co/16x16/FF6B9D/FF6B9D.png) `#FF6B9D` |
| WARN fg | ![](https://placehold.co/16x16/E8A74E/E8A74E.png) `#E8A74E` | ![](https://placehold.co/16x16/F5A0C0/F5A0C0.png) `#F5A0C0` | ![](https://placehold.co/16x16/FFE8A0/FFE8A0.png) `#FFE8A0` | ![](https://placehold.co/16x16/D4A050/D4A050.png) `#D4A050` | ![](https://placehold.co/16x16/FFE040/FFE040.png) `#FFE040` | ![](https://placehold.co/16x16/FFD700/FFD700.png) `#FFD700` |
| NOTICE fg | ![](https://placehold.co/16x16/7BC8F6/7BC8F6.png) `#7BC8F6` | ![](https://placehold.co/16x16/D4A0B9/D4A0B9.png) `#D4A0B9` | ![](https://placehold.co/16x16/C8A2C8/C8A2C8.png) `#C8A2C8` | ![](https://placehold.co/16x16/B0A8B9/B0A8B9.png) `#B0A8B9` | ![](https://placehold.co/16x16/FFD700/FFD700.png) `#FFD700` | ![](https://placehold.co/16x16/4DA6FF/4DA6FF.png) `#4DA6FF` |
| INFO fg | ![](https://placehold.co/16x16/C8D6E5/C8D6E5.png) `#C8D6E5` | ![](https://placehold.co/16x16/C8C8C8/C8C8C8.png) `#C8C8C8` | ![](https://placehold.co/16x16/E8D0DA/E8D0DA.png) `#E8D0DA` | ![](https://placehold.co/16x16/F0EDE8/F0EDE8.png) `#F0EDE8` | ![](https://placehold.co/16x16/FFF0D4/FFF0D4.png) `#FFF0D4` | ![](https://placehold.co/16x16/FFF5E6/FFF5E6.png) `#FFF5E6` |
| DEBUG fg | ![](https://placehold.co/16x16/4A7B9D/4A7B9D.png) `#4A7B9D` | ![](https://placehold.co/16x16/6B5B6B/6B5B6B.png) `#6B5B6B` | ![](https://placehold.co/16x16/8A6A7E/8A6A7E.png) `#8A6A7E` | ![](https://placehold.co/16x16/6B6B80/6B6B80.png) `#6B6B80` | ![](https://placehold.co/16x16/A67C52/A67C52.png) `#A67C52` | ![](https://placehold.co/16x16/6B4858/6B4858.png) `#6B4858` |
| TRACE fg | ![](https://placehold.co/16x16/3B5A7A/3B5A7A.png) `#3B5A7A` | ![](https://placehold.co/16x16/4A3A4A/4A3A4A.png) `#4A3A4A` | ![](https://placehold.co/16x16/5A3A4E/5A3A4E.png) `#5A3A4E` | ![](https://placehold.co/16x16/3A3A4E/3A3A4E.png) `#3A3A4E` | ![](https://placehold.co/16x16/5A4830/5A4830.png) `#5A4830` | ![](https://placehold.co/16x16/4A2838/4A2838.png) `#4A2838` |

#### Table

| Area | mizuiro | landmine | amai | maid | gyaru | dopamine |
|------|------|------|------|------|------|------|
| Header fg | ![](https://placehold.co/16x16/0A1628/0A1628.png) `#0A1628` bold | ![](https://placehold.co/16x16/0D060B/0D060B.png) `#0D060B` bold| ![](https://placehold.co/16x16/140A10/140A10.png) `#140A10` bold | ![](https://placehold.co/16x16/0D0D1A/0D0D1A.png) `#0D0D1A` bold | ![](https://placehold.co/16x16/1A1208/1A1208.png) `#1A1208` bold | ![](https://placehold.co/16x16/1A1418/1A1418.png) `#1A1418` bold |
| Header bg | ![](https://placehold.co/16x16/5BA4CF/5BA4CF.png) `#5BA4CF` | ![](https://placehold.co/16x16/E8577E/E8577E.png) `#E8577E` | ![](https://placehold.co/16x16/FFC8D6/FFC8D6.png) `#FFC8D6` | ![](https://placehold.co/16x16/F0EDE8/F0EDE8.png) `#F0EDE8` | ![](https://placehold.co/16x16/FFD700/FFD700.png) `#FFD700` | ![](https://placehold.co/16x16/B388FF/B388FF.png) `#B388FF` |
| Header unfocused fg/bg | ![](https://placehold.co/16x16/4A7B9D/4A7B9D.png) `#4A7B9D` / ![](https://placehold.co/16x16/0F2038/0F2038.png) `#0F2038` | ![](https://placehold.co/16x16/6B4A5E/6B4A5E.png) `#6B4A5E` / ![](https://placehold.co/16x16/1A0A14/1A0A14.png) `#1A0A14` | ![](https://placehold.co/16x16/B08A9E/B08A9E.png) `#B08A9E` / ![](https://placehold.co/16x16/3D2540/3D2540.png) `#3D2540` | ![](https://placehold.co/16x16/6B6B80/6B6B80.png) `#6B6B80` / ![](https://placehold.co/16x16/1A1A2E/1A1A2E.png) `#1A1A2E` | ![](https://placehold.co/16x16/A67C52/A67C52.png) `#A67C52` / ![](https://placehold.co/16x16/2A1F14/2A1F14.png) `#2A1F14` | ![](https://placehold.co/16x16/A89098/A89098.png) `#A89098` / ![](https://placehold.co/16x16/241820/241820.png) `#241820` |
| Selected bg | ![](https://placehold.co/16x16/162E48/162E48.png) `#162E48` | ![](https://placehold.co/16x16/2D1028/2D1028.png) `#2D1028` | ![](https://placehold.co/16x16/3A1830/3A1830.png) `#3A1830` | ![](https://placehold.co/16x16/1E1E38/1E1E38.png) `#1E1E38` | ![](https://placehold.co/16x16/3A2818/3A2818.png) `#3A2818` | ![](https://placehold.co/16x16/2E1828/2E1828.png) `#2E1828` |
| Alternating bg | ![](https://placehold.co/16x16/0A1628/0A1628.png) `#0A1628` | ![](https://placehold.co/16x16/0D060B/0D060B.png) `#0D060B` | ![](https://placehold.co/16x16/140A10/140A10.png) `#140A10` | ![](https://placehold.co/16x16/0D0D1A/0D0D1A.png) `#0D0D1A` | ![](https://placehold.co/16x16/1A1208/1A1208.png) `#1A1208` | ![](https://placehold.co/16x16/1A1418/1A1418.png) `#1A1418` |
| Separator fg | ![](https://placehold.co/16x16/1E3A50/1E3A50.png) `#1E3A50` | ![](https://placehold.co/16x16/4A2040/4A2040.png) `#4A2040` | ![](https://placehold.co/16x16/4A2E40/4A2E40.png) `#4A2E40` | ![](https://placehold.co/16x16/2A2A3E/2A2A3E.png) `#2A2A3E` | ![](https://placehold.co/16x16/3A2A18/3A2A18.png) `#3A2A18` | ![](https://placehold.co/16x16/3A1828/3A1828.png) `#3A1828` |
| Separator char | `│` | `♡` | `♡` | `│` | `│` | `│` |

#### Status Bar

| Area | mizuiro | landmine | amai | maid | gyaru | dopamine |
|------|------|------|------|------|------|------|
| Line 1 fg | ![](https://placehold.co/16x16/8BA4B8/8BA4B8.png) `#8BA4B8` | ![](https://placehold.co/16x16/D4A0B9/D4A0B9.png) `#D4A0B9` | ![](https://placehold.co/16x16/E8B8C8/E8B8C8.png) `#E8B8C8` | ![](https://placehold.co/16x16/B0A8B9/B0A8B9.png) `#B0A8B9` | ![](https://placehold.co/16x16/C68642/C68642.png) `#C68642` | ![](https://placehold.co/16x16/A89098/A89098.png) `#A89098` |
| Line 1 bg | ![](https://placehold.co/16x16/0F2038/0F2038.png) `#0F2038` | ![](https://placehold.co/16x16/1A0A14/1A0A14.png) `#1A0A14` | ![](https://placehold.co/16x16/3D2540/3D2540.png) `#3D2540` | ![](https://placehold.co/16x16/1A1A2E/1A1A2E.png) `#1A1A2E` | ![](https://placehold.co/16x16/2A1F14/2A1F14.png) `#2A1F14` | ![](https://placehold.co/16x16/241820/241820.png) `#241820` |
| Line 2 fg | ![](https://placehold.co/16x16/4A7B9D/4A7B9D.png) `#4A7B9D` | ![](https://placehold.co/16x16/8A6A7E/8A6A7E.png) `#8A6A7E` | ![](https://placehold.co/16x16/B08A9E/B08A9E.png) `#B08A9E` | ![](https://placehold.co/16x16/6B6B80/6B6B80.png) `#6B6B80` | ![](https://placehold.co/16x16/A67C52/A67C52.png) `#A67C52` | ![](https://placehold.co/16x16/6B4858/6B4858.png) `#6B4858` |
| Line 2 bg | ![](https://placehold.co/16x16/0A1628/0A1628.png) `#0A1628` | ![](https://placehold.co/16x16/0D060B/0D060B.png) `#0D060B` | ![](https://placehold.co/16x16/140A10/140A10.png) `#140A10` | ![](https://placehold.co/16x16/0D0D1A/0D0D1A.png) `#0D0D1A` | ![](https://placehold.co/16x16/1A1208/1A1208.png) `#1A1208` | ![](https://placehold.co/16x16/1A1418/1A1418.png) `#1A1418` |
| Mode label fg/bg | ![](https://placehold.co/16x16/0A1628/0A1628.png) `#0A1628` / ![](https://placehold.co/16x16/5BA4CF/5BA4CF.png) `#5BA4CF` bold | ![](https://placehold.co/16x16/0D060B/0D060B.png) `#0D060B` / ![](https://placehold.co/16x16/E8577E/E8577E.png) `#E8577E` bold| ![](https://placehold.co/16x16/140A10/140A10.png) `#140A10` / ![](https://placehold.co/16x16/FFC8D6/FFC8D6.png) `#FFC8D6` bold | ![](https://placehold.co/16x16/0D0D1A/0D0D1A.png) `#0D0D1A` / ![](https://placehold.co/16x16/F0EDE8/F0EDE8.png) `#F0EDE8` bold | ![](https://placehold.co/16x16/1A1208/1A1208.png) `#1A1208` / ![](https://placehold.co/16x16/FFD700/FFD700.png) `#FFD700` bold | ![](https://placehold.co/16x16/1A1418/1A1418.png) `#1A1418` / ![](https://placehold.co/16x16/FFD700/FFD700.png) `#FFD700` bold |
| Density chart fg | ![](https://placehold.co/16x16/5BA4CF/5BA4CF.png) `#5BA4CF` | ![](https://placehold.co/16x16/E8577E/E8577E.png) `#E8577E` | ![](https://placehold.co/16x16/FFC8D6/FFC8D6.png) `#FFC8D6` | ![](https://placehold.co/16x16/B0A8B9/B0A8B9.png) `#B0A8B9` | ![](https://placehold.co/16x16/C68642/C68642.png) `#C68642` | ![](https://placehold.co/16x16/FF6B9D/FF6B9D.png) `#FF6B9D` |
| Density label fg | ![](https://placehold.co/16x16/3B5A7A/3B5A7A.png) `#3B5A7A` | ![](https://placehold.co/16x16/6B4A5E/6B4A5E.png) `#6B4A5E` | ![](https://placehold.co/16x16/7A5A6E/7A5A6E.png) `#7A5A6E` | ![](https://placehold.co/16x16/3A3A4E/3A3A4E.png) `#3A3A4E` | ![](https://placehold.co/16x16/6B5A28/6B5A28.png) `#6B5A28` | ![](https://placehold.co/16x16/6B4858/6B4858.png) `#6B4858` |
| Density tick fg | ![](https://placehold.co/16x16/1E3A50/1E3A50.png) `#1E3A50` | ![](https://placehold.co/16x16/4A2040/4A2040.png) `#4A2040` | ![](https://placehold.co/16x16/4A2E40/4A2E40.png) `#4A2E40` | ![](https://placehold.co/16x16/2A2A3E/2A2A3E.png) `#2A2A3E` | ![](https://placehold.co/16x16/3A2A18/3A2A18.png) `#3A2A18` | ![](https://placehold.co/16x16/3A1828/3A1828.png) `#3A1828` |
| Position fg | ![](https://placehold.co/16x16/A8D8EA/A8D8EA.png) `#A8D8EA` | ![](https://placehold.co/16x16/F5D0E0/F5D0E0.png) `#F5D0E0` | ![](https://placehold.co/16x16/D4B2D4/D4B2D4.png) `#D4B2D4` | ![](https://placehold.co/16x16/F0EDE8/F0EDE8.png) `#F0EDE8` | ![](https://placehold.co/16x16/FFD700/FFD700.png) `#FFD700` | ![](https://placehold.co/16x16/B388FF/B388FF.png) `#B388FF` |
| Cursor marker fg | ![](https://placehold.co/16x16/7BC8F6/7BC8F6.png) `#7BC8F6` | ![](https://placehold.co/16x16/FF3366/FF3366.png) `#FF3366` | ![](https://placehold.co/16x16/FF6B8A/FF6B8A.png) `#FF6B8A` | ![](https://placehold.co/16x16/C43060/C43060.png) `#C43060` | ![](https://placehold.co/16x16/FF2499/FF2499.png) `#FF2499` | |

#### Search & Dialogs

| Area | mizuiro | landmine | amai | maid | gyaru | dopamine |
|------|------|------|------|------|------|------|
| Match fg/bg | ![](https://placehold.co/16x16/0A1628/0A1628.png) `#0A1628` / ![](https://placehold.co/16x16/A8D8EA/A8D8EA.png) `#A8D8EA` | `black` / ![](https://placehold.co/16x16/F5A0C0/F5A0C0.png) `#F5A0C0` | ![](https://placehold.co/16x16/140A10/140A10.png) `#140A10` / ![](https://placehold.co/16x16/C8A2C8/C8A2C8.png) `#C8A2C8` | ![](https://placehold.co/16x16/0D0D1A/0D0D1A.png) `#0D0D1A` / ![](https://placehold.co/16x16/B0A8B9/B0A8B9.png) `#B0A8B9` | ![](https://placehold.co/16x16/1A1208/1A1208.png) `#1A1208` / ![](https://placehold.co/16x16/C68642/C68642.png) `#C68642` | ![](https://placehold.co/16x16/1A1418/1A1418.png) `#1A1418` / ![](https://placehold.co/16x16/B388FF/B388FF.png) `#B388FF` |
| Current match fg/bg | ![](https://placehold.co/16x16/0A1628/0A1628.png) `#0A1628` / ![](https://placehold.co/16x16/7BC8F6/7BC8F6.png) `#7BC8F6` | `black` / ![](https://placehold.co/16x16/FF3366/FF3366.png) `#FF3366` | ![](https://placehold.co/16x16/140A10/140A10.png) `#140A10` / ![](https://placehold.co/16x16/FFC8D6/FFC8D6.png) `#FFC8D6` | ![](https://placehold.co/16x16/0D0D1A/0D0D1A.png) `#0D0D1A` / ![](https://placehold.co/16x16/F0EDE8/F0EDE8.png) `#F0EDE8` | ![](https://placehold.co/16x16/1A1208/1A1208.png) `#1A1208` / ![](https://placehold.co/16x16/FFD700/FFD700.png) `#FFD700` | ![](https://placehold.co/16x16/1A1418/1A1418.png) `#1A1418` / ![](https://placehold.co/16x16/FFD700/FFD700.png) `#FFD700` |
| Dialog border fg | ![](https://placehold.co/16x16/5BA4CF/5BA4CF.png) `#5BA4CF` | ![](https://placehold.co/16x16/E8577E/E8577E.png) `#E8577E` | ![](https://placehold.co/16x16/FFC8D6/FFC8D6.png) `#FFC8D6` | ![](https://placehold.co/16x16/6B6B80/6B6B80.png) `#6B6B80` | ![](https://placehold.co/16x16/C68642/C68642.png) `#C68642` | ![](https://placehold.co/16x16/FF6B9D/FF6B9D.png) `#FF6B9D` |
| Input prompt fg | ![](https://placehold.co/16x16/7BC8F6/7BC8F6.png) `#7BC8F6` | ![](https://placehold.co/16x16/F5A0C0/F5A0C0.png) `#F5A0C0` | ![](https://placehold.co/16x16/C8A2C8/C8A2C8.png) `#C8A2C8` | ![](https://placehold.co/16x16/F0EDE8/F0EDE8.png) `#F0EDE8` | ![](https://placehold.co/16x16/FFD700/FFD700.png) `#FFD700` | |

#### Panel Tab

| Area | mizuiro | landmine | amai | maid | gyaru | dopamine |
|------|------|------|------|------|------|------|
| Focused fg/bg | ![](https://placehold.co/16x16/0A1628/0A1628.png) `#0A1628` / ![](https://placehold.co/16x16/5BA4CF/5BA4CF.png) `#5BA4CF` bold | ![](https://placehold.co/16x16/0D060B/0D060B.png) `#0D060B` / ![](https://placehold.co/16x16/E8577E/E8577E.png) `#E8577E` bold| ![](https://placehold.co/16x16/140A10/140A10.png) `#140A10` / ![](https://placehold.co/16x16/FFC8D6/FFC8D6.png) `#FFC8D6` bold | ![](https://placehold.co/16x16/0D0D1A/0D0D1A.png) `#0D0D1A` / ![](https://placehold.co/16x16/F0EDE8/F0EDE8.png) `#F0EDE8` bold | ![](https://placehold.co/16x16/1A1208/1A1208.png) `#1A1208` / ![](https://placehold.co/16x16/FFD700/FFD700.png) `#FFD700` bold | ![](https://placehold.co/16x16/1A1418/1A1418.png) `#1A1418` / ![](https://placehold.co/16x16/FFD700/FFD700.png) `#FFD700` bold |
| Unfocused fg/bg | ![](https://placehold.co/16x16/4A7B9D/4A7B9D.png) `#4A7B9D` / ![](https://placehold.co/16x16/0F2038/0F2038.png) `#0F2038` | ![](https://placehold.co/16x16/6B4A5E/6B4A5E.png) `#6B4A5E` / ![](https://placehold.co/16x16/1A0A14/1A0A14.png) `#1A0A14` | ![](https://placehold.co/16x16/B08A9E/B08A9E.png) `#B08A9E` / ![](https://placehold.co/16x16/3D2540/3D2540.png) `#3D2540` | ![](https://placehold.co/16x16/6B6B80/6B6B80.png) `#6B6B80` / ![](https://placehold.co/16x16/1A1A2E/1A1A2E.png) `#1A1A2E` | ![](https://placehold.co/16x16/A67C52/A67C52.png) `#A67C52` / ![](https://placehold.co/16x16/2A1F14/2A1F14.png) `#2A1F14` | ![](https://placehold.co/16x16/A89098/A89098.png) `#A89098` / ![](https://placehold.co/16x16/241820/241820.png) `#241820` |
| Bar bg | ![](https://placehold.co/16x16/0A1628/0A1628.png) `#0A1628` | ![](https://placehold.co/16x16/0D060B/0D060B.png) `#0D060B` | ![](https://placehold.co/16x16/140A10/140A10.png) `#140A10` | ![](https://placehold.co/16x16/0D0D1A/0D0D1A.png) `#0D0D1A` | ![](https://placehold.co/16x16/1A1208/1A1208.png) `#1A1208` | ![](https://placehold.co/16x16/1A1418/1A1418.png) `#1A1418` |

#### General & Accents

| Area | mizuiro | landmine | amai | maid | gyaru | dopamine |
|------|------|------|------|------|------|------|
| Accent fg | ![](https://placehold.co/16x16/5BA4CF/5BA4CF.png) `#5BA4CF` | ![](https://placehold.co/16x16/E8577E/E8577E.png) `#E8577E` | ![](https://placehold.co/16x16/FFC8D6/FFC8D6.png) `#FFC8D6` | ![](https://placehold.co/16x16/8B2252/8B2252.png) `#8B2252` | ![](https://placehold.co/16x16/FFD700/FFD700.png) `#FFD700` | ![](https://placehold.co/16x16/FFD700/FFD700.png) `#FFD700` |
| Border fg | ![](https://placehold.co/16x16/1E3A50/1E3A50.png) `#1E3A50` | ![](https://placehold.co/16x16/3D1A30/3D1A30.png) `#3D1A30` | ![](https://placehold.co/16x16/4A2E40/4A2E40.png) `#4A2E40` | ![](https://placehold.co/16x16/2A2A3E/2A2A3E.png) `#2A2A3E` | ![](https://placehold.co/16x16/3A2A18/3A2A18.png) `#3A2A18` | ![](https://placehold.co/16x16/3A1828/3A1828.png) `#3A1828` |
| Muted fg | ![](https://placehold.co/16x16/3B5A7A/3B5A7A.png) `#3B5A7A` | ![](https://placehold.co/16x16/6B4A5E/6B4A5E.png) `#6B4A5E` | ![](https://placehold.co/16x16/7A5A6E/7A5A6E.png) `#7A5A6E` | ![](https://placehold.co/16x16/3A3A4E/3A3A4E.png) `#3A3A4E` | ![](https://placehold.co/16x16/5A4830/5A4830.png) `#5A4830` | ![](https://placehold.co/16x16/6B4858/6B4858.png) `#6B4858` |

#### Highlight Palette (color rotation order)

| # | mizuiro | landmine | amai | maid | gyaru | |
|------|------|------|------|------|------|------|
| 1 | ![](https://placehold.co/16x16/7BC8F6/7BC8F6.png) `#7BC8F6` | ![](https://placehold.co/16x16/FF3366/FF3366.png) `#FF3366` | ![](https://placehold.co/16x16/FF6B8A/FF6B8A.png) `#FF6B8A` | ![](https://placehold.co/16x16/F0EDE8/F0EDE8.png) `#F0EDE8` | ![](https://placehold.co/16x16/FF2499/FF2499.png) `#FF2499` | |
| 2 | ![](https://placehold.co/16x16/5BA4CF/5BA4CF.png) `#5BA4CF` | ![](https://placehold.co/16x16/F5A0C0/F5A0C0.png) `#F5A0C0` | ![](https://placehold.co/16x16/FFC8D6/FFC8D6.png) `#FFC8D6` | ![](https://placehold.co/16x16/B0A8B9/B0A8B9.png) `#B0A8B9` | ![](https://placehold.co/16x16/FFD700/FFD700.png) `#FFD700` | |
| 3 | ![](https://placehold.co/16x16/A8D8EA/A8D8EA.png) `#A8D8EA` | ![](https://placehold.co/16x16/D4A0B9/D4A0B9.png) `#D4A0B9` | ![](https://placehold.co/16x16/C8A2C8/C8A2C8.png) `#C8A2C8` | ![](https://placehold.co/16x16/C43060/C43060.png) `#C43060` | ![](https://placehold.co/16x16/C68642/C68642.png) `#C68642` | |
| 4 | ![](https://placehold.co/16x16/2E6B9E/2E6B9E.png) `#2E6B9E` | ![](https://placehold.co/16x16/E8577E/E8577E.png) `#E8577E` | ![](https://placehold.co/16x16/98D8C8/98D8C8.png) `#98D8C8` | ![](https://placehold.co/16x16/8B2252/8B2252.png) `#8B2252` | ![](https://placehold.co/16x16/FF69B4/FF69B4.png) `#FF69B4` | |
| 5 | ![](https://placehold.co/16x16/D4EEF6/D4EEF6.png) `#D4EEF6` | ![](https://placehold.co/16x16/F5D0E0/F5D0E0.png) `#F5D0E0` | ![](https://placehold.co/16x16/6B8EC2/6B8EC2.png) `#6B8EC2` | ![](https://placehold.co/16x16/6880A0/6880A0.png) `#6880A0` | ![](https://placehold.co/16x16/FFE040/FFE040.png) `#FFE040` | |
| 6 | ![](https://placehold.co/16x16/4A7B9D/4A7B9D.png) `#4A7B9D` | ![](https://placehold.co/16x16/8A6A7E/8A6A7E.png) `#8A6A7E` | ![](https://placehold.co/16x16/FFE8A0/FFE8A0.png) `#FFE8A0` | ![](https://placehold.co/16x16/6B6B80/6B6B80.png) `#6B6B80` | ![](https://placehold.co/16x16/8B6914/8B6914.png) `#8B6914` | |

### Integration

- `Theme` is created once at startup and passed by reference to all render functions
- All `Color::*` literals replaced with `theme.field` access
- Widgets accept `&Theme` parameter in their render methods
- Theme selected via `config.yaml` or `--theme` CLI flag (see config spec)


## Change Log

| Date | Change |
|------|------|
| 2026-02-22 | Initial theme system design |
| 2026-02-22 | Moved theme file format and color details from config spec |
| 2026-02-23 | Redesign default theme: distinct status bar lines, softer colors, clear visual hierarchy |
| 2026-02-23 | Replace text descriptions with color swatch tables for all 4 built-in presets |
| 2026-02-24 | Added `landmine` theme (Jirai Kei: black + pink); separator char now themeable |
| 2026-02-28 | Added `panel_tab` theme section: focused (accent), unfocused (gray) styles for all 5 presets |
| 2026-02-28 | Added `table.header_unfocused`: table header goes gray when focus moves to panel |
| 2026-02-28 | Table header focused color now matches `panel_tab.focused` (accent bg) for consistent focus visual language |
| 2026-03-02 | Replace all ANSI 16 colors in default theme with RGB equivalents for terminal theme independence |
| 2026-03-02 | Added `density_tick` field to StatusBarTheme for density chart tick marks |
| 2026-03-02 | Added 4 new fashion-inspired themes: mizuiro, amai, maid, gyaru |
| 2026-03-02 | Tuned amai theme: darker log bg (#140A10), brighter highlights (header #FFC8D6, status bar #3D2540) |
| 2026-03-02 | Added `--theme-list` CLI command and per-theme `description` field |
| 2026-03-02 | `description` is now a Theme struct field, visible in `--theme-dump` YAML output |
| 2026-03-02 | Renamed `--generate-theme` to `--theme-dump` |
| 2026-03-02 | Removed default/dark/light/solarized themes; mizuiro is now the default theme |
| 2026-03-02 | Added dopamine theme: rainbow maximalist with neon pink, sunflower yellow, electric blue |
