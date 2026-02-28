# TUI Overview

## Overview

`scouty-tui` is an interactive terminal log viewer built on the `scouty` core library. It provides table-based log browsing, filtering, searching, and analysis through a component-based architecture using `ratatui` + `crossterm`.


## Design

### Layout

```
┌──────────────────────────────────┬────────────────────────────────────────┐
│ Time                             │ Log                                    │
├──────────────────────────────────┼────────────────────────────────────────┤
│                                  │                                        │
│  (Log Table — scrollable main area)                                       │
├─── ▾ Detail ── Region ───────────┴────────────────────────────────────────┤
│ [Log Content]                          │ [Fields]                          │
│ (Panel Area — collapsible)             │ Timestamp: ...                    │
├────────────────────────────────────────┴──────────────────────────────────-┤
│ ▁▂▃▅▇█▇▅▃▂▁▁▂▃▄▅▆▇ │ 1,234/5,678 (Total: 10,000)  ← Line 1: density   │
│ [VIEW] /: Search │ f: Filter │ ?: Help                  ← Line 2: status  │
└───────────────────────────────────────────────────────────────────────────-┘
```

> **Note:** Default columns are **Time** and **Log** only. Additional columns (Level, ProcessName, Pid, Tid, Component, etc.) can be toggled via the `c` column selector. Panel area is collapsible — see [panel-system.md](panel-system.md) for details.

### UI Architecture

See [ui-architecture.md](ui-architecture.md) for the full design.

**Window Stack:** A stack of windows; the topmost window always owns focus and receives all keyboard input. The Main Window sits at the bottom; overlay windows (Help, Filter Manager, etc.) are pushed on top.

**Widget Tree:** Each window contains a tree of widgets. Keyboard input goes to the focused widget first; if unhandled, it bubbles up to the parent, then grandparent, until the root window handles it or discards it.

### UiComponent Trait (Legacy)

> **Note:** Being replaced by the `Window` + `Widget` traits defined in [ui-architecture.md](ui-architecture.md).

```rust
trait UiComponent {
    fn render(&self, frame: &mut Frame, area: Rect);
    fn on_up(&mut self) {}
    fn on_down(&mut self) {}
    fn on_page_up(&mut self) {}
    fn on_page_down(&mut self) {}
    fn on_toggle(&mut self) {}        // Space
    fn on_confirm(&mut self) {}       // Enter
    fn on_cancel(&mut self) {}        // Esc
    fn on_char(&mut self, c: char) {}
    fn on_key(&mut self, key: KeyEvent) {}
}
```

Framework dispatches key events to the active component — components never directly match `KeyEvent`.

### File Structure

```
crates/scouty-tui/src/ui/
├── mod.rs              # UiComponent trait + dispatch logic
├── windows/            # Pop-up windows (open/close lifecycle)
│   ├── field_filter_window.rs
│   ├── filter_manager_window.rs
│   ├── column_selector_window.rs
│   ├── copy_format_window.rs
│   ├── goto_line_window.rs
│   ├── help_window.rs
│   ├── highlight_manager_window.rs
│   └── stats_window.rs
└── widgets/            # Persistent components
    ├── log_table_widget.rs
    ├── detail_panel_widget.rs
    ├── region_panel_widget.rs
    ├── panel_manager.rs
    ├── search_input_widget.rs
    ├── filter_input_widget.rs
    └── status_bar_widget.rs
```

- **Window** — pop-up overlay with open/close lifecycle, named `XxxWindow`
- **Widget** — always present in layout, named `XxxWidget`

### Event Dispatch Flow

> **Note:** Being replaced by window stack + widget tree bubbling. See [ui-architecture.md](ui-architecture.md).

```
KeyEvent arrives
    ├─ WindowStack.top() receives input
    ├─ Focused widget in that window handles it? → done
    └─ Bubble to parent → … → root window
```

### Component Communication

Components notify App via return values or callbacks. App updates shared state (LogStoreView, etc.) and triggers dependent refreshes.

### Keybinding Summary

#### Global (always active)

| Key | Function |
|-----|----------|
| `Esc` | Close current overlay / cancel input |
| `q` | Quit |
| `?` | Help |

#### Log Table View (main view, no overlay open)

| Key | Function |
|-----|----------|
| `j`/`k` | Move up/down one row |
| `Ctrl+j`/`Ctrl+k` | Page up/down |
| `g`/`G` | First/last row |
| `Ctrl+G` | Go to line number |
| `Enter` | Toggle detail panel |
| `/` | Search (regex) |
| `n`/`N` | Next/prev search match |
| `f` | Filter expression input |
| `-`/`=` | Quick exclude/include text |
| `_`/`+` | Field exclude/include dialog |
| `F` | Filter manager |
| `l` | Log level quick filter (1-8) |
| `c` | Column selector |
| `y`/`Y` | Copy raw / format selection |
| `s` | Save/export dialog (path + format) |
| `d`/`D` | Cycle / select density chart source (level/highlight) |
| `h`/`H` | Add highlight / highlight manager |
| `m` | Toggle bookmark |
| `'`/`"` | Next/prev bookmark |
| `M` | Bookmark manager |
| `S` | Stats summary |
| `r` / `R` | Region panel / jump to next region start |
| `Tab`/`Shift+Tab` | Cycle focus: Log Table → Detail → Region → Stats → Log Table (expand if collapsed) |
| `z` | Toggle panel maximize/restore |
| `]`/`[` | Relative time jump (forward/backward) |
| `Ctrl+]` | Toggle follow mode |

#### Dialog Navigation (shared across all overlays/dialogs)

| Key | Function |
|-----|----------|
| `j`/`k`/`↑`/`↓` | Move selection |
| `PageUp`/`PageDown` | Page through options |
| `Space` | Toggle selection (multi-select dialogs) |
| `Enter` | Confirm |
| `Esc` | Cancel / close |

## Change Log

| Date | Change |
|------|--------|
| 2026-02-20 | TUI log viewer full interaction design |
| 2026-02-21 | Architecture refactor to UiComponent trait + windows/widgets structure |
| 2026-02-26 | Panel system: collapsible panels with tab bar, Ctrl+arrow focus/switch, maximize |
| 2026-02-28 | UI architecture redesign: window stack + widget tree with event bubbling (see ui-architecture.md) |
