# Default Theme: Replace ANSI 16 Colors with RGB - High-Level Spec

## Background & Goals

Terminal emulators (iTerm2, Windows Terminal, Alacritty, etc.) ship with color themes that remap ANSI 16 colors (e.g., `Red`, `Blue`, `White`) to arbitrary values. This means scouty's UI colors change unpredictably based on the user's terminal theme. RGB true colors (`#RRGGBB` / `Color::Rgb(r,g,b)`) are rendered exactly as specified, immune to terminal remapping.

Most of scouty's default theme already uses RGB colors. A few places still use ANSI 16 colors that need migration.

## Problem Statement

Users with custom terminal themes (Solarized, Dracula, Gruvbox, etc.) see incorrect or unreadable colors in scouty's UI because some default theme values use ANSI 16 colors that get remapped.

## User Stories

- As a user with a custom terminal color theme, I want scouty's colors to look consistent regardless of my terminal settings

## Requirements Breakdown

### P0 — Must Have

- [ ] **Replace all ANSI 16 colors in the default theme with RGB equivalents** (dependency: none)
  - Audit all `Color::Black`, `Color::Red`, `Color::Green`, `Color::Yellow`, `Color::Blue`, `Color::Magenta`, `Color::Cyan`, `Color::White`, `Color::Gray`, `Color::DarkGray` in `theme.rs` defaults
  - Replace each with a visually equivalent `Color::Rgb(r, g, b)` value
  - Suggested mapping (can be adjusted for visual coherence):
    - `Black` → `#0D1117`
    - `Red` → `#FF6B6B` (matches existing error color)
    - `Green` → `#6BCB77` (matches existing notice color)
    - `Yellow` → `#FFD93D` (matches existing warn color)
    - `Blue` → `#4FC3F7` (matches existing info color)
    - `Magenta` → `#CE93D8`
    - `Cyan` → `#4DD0E1`
    - `White` → `#D4D4D4`
    - `DarkGray` → `#5C5C5C` (matches existing trace color)

- [ ] **New theme field: `density_tick`** — subtle tick marks every 10 columns on density chart X-axis (dependency: none)

  | Theme | `density_tick` | Rationale |
  |-------|---------------|-----------|
  | **Default** (dark blue) | `#3B4252` | Matches border color, subtle against `#1B2838` bg |
  | **Ocean** | `#2A4A5A` | Muted teal, between ocean `density_normal` and bg |
  | **Forest** | `#2A4A2A` | Dark green, consistent with forest green palette |
  | **Solarized** | `#073642` | Solarized base02, standard subtle element color |
  | **Sakura** | `#4A2040` | Dark plum, matches sakura pink/plum palette |

  The tick should be noticeably dimmer than `density_label` in each theme.

### P1 — Should Have

- [ ] **Document in help/README**: Users creating custom themes should use `#RRGGBB` format for terminal-independent colors

## Functional Requirements

- Default theme values in `theme.rs` must use only RGB true colors (`Color::Rgb`)
- User-defined themes (YAML) already support `#RRGGBB` — no changes needed there
- `Color::Reset` stays as-is (it means "use terminal default", which is intentional)
- All built-in themes (default, ocean, forest, solarized, sakura) must include `density_tick`

## Acceptance Criteria

- [ ] No `Color::Black/Red/Green/Yellow/Blue/Magenta/Cyan/White/Gray/DarkGray` in default theme definition (except `Color::Reset`)
- [ ] Visual appearance is equivalent to current defaults on a standard dark terminal
- [ ] Users with Solarized/Dracula/Gruvbox terminal themes see consistent scouty colors
- [ ] All built-in themes include `density_tick` with per-theme colors as specified above

## Out of Scope

- Changing the theme config format
- Light theme variant (separate feature)

## Open Questions

None
