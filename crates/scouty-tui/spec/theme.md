# Theme

## Overview

Centralized color and style management for the TUI. All UI colors are defined in a single `Theme` struct, enabling consistent styling and future theme customization.

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

### Color Categories

| Category | Elements | Current Problem |
|----------|----------|-----------------|
| Log levels | Row text color by severity | OK but could be richer |
| Table | Header, selection, alternation | Header is plain DarkGray |
| Status bar | Background, mode labels, density | Dull Rgb(20,20,40) |
| Dialogs | Borders, titles, selection | Gray borders, black bg |
| Detail panel | Field names vs values | No differentiation |
| Input fields | Prompt, cursor, error | Inconsistent across modes |
| General | Accents, borders, muted text | Everything defaults to gray |

### Default Theme

Vibrant dark theme with blue/cyan accents:

- **Borders & accents**: Cyan / Blue (not gray)
- **Status bar**: Dark teal background
- **Log levels**: FATAL red bold, ERROR red, WARN gold, NOTICE cyan, INFO green, DEBUG gray, TRACE dark gray
- **Selected row**: Blue highlight background
- **Alternating rows**: Subtle dark shade difference
- **Dialogs**: Cyan borders, blue title, clear selection highlight
- **Input prompts**: Yellow labels, white text

### Integration

- `Theme` is created once at startup and passed by reference to all render functions
- All `Color::*` literals replaced with `theme.field` access
- Widgets accept `&Theme` parameter in their render methods

### Future Extensibility

- Built-in presets: `default`, `dark`, `light`, `solarized`
- CLI flag: `--theme <name>`
- Custom theme file: YAML/TOML with color overrides
- Auto-detect terminal color depth (16 / 256 / true color)

## Change Log

| Date | Change |
|------|--------|
| 2026-02-22 | Initial theme system design |
