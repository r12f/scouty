# TUI Overview

## Overview

`scouty-tui` is an interactive terminal log viewer built on the `scouty` core library. It provides table-based log browsing, filtering, searching, and analysis through a component-based architecture using `ratatui` + `crossterm`.


## Design

### Layout

```
┌────────┬───────┬─────────────┬───────┬───────┬───────────┬─────────────────┐
│ Time   │ Level │ ProcessName │ Pid   │ Tid   │ Component │ Log             │
├────────┼───────┼─────────────┼───────┼───────┼───────────┼─────────────────┤
│        │ ERROR │             │       │       │           │                 │
│  (Log Table — scrollable main area)                                       │
├────────┴───────┴─────────────┴───────┴───────┴───────────┴─────────────────┤
│ [Log Content]                          │ [Fields]                          │
│ (Detail Panel — left/right split)      │ Timestamp: ...                    │
├────────────────────────────────────────┴──────────────────────────────────-┤
│ ▁▂▃▅▇█▇▅▃▂▁▁▂▃▄▅▆▇ │ 1,234/5,678 (Total: 10,000)  ← Line 1: density   │
│ [VIEW] /: Search │ f: Filter │ ?: Help                  ← Line 2: status  │
└───────────────────────────────────────────────────────────────────────────-┘
```

### UiComponent Trait

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
    ├── search_input_widget.rs
    ├── filter_input_widget.rs
    └── status_bar_widget.rs
```

- **Window** — pop-up overlay with open/close lifecycle, named `XxxWindow`
- **Widget** — always present in layout, named `XxxWidget`

### Event Dispatch Flow

```
KeyEvent arrives
    ├─ Global shortcut? (q exit) → handle directly
    ├─ Active window? → dispatch to window's UiComponent callbacks
    └─ No active window → dispatch to focused widget
```

### Component Communication

Components notify App via return values or callbacks. App updates shared state (LogStoreView, etc.) and triggers dependent refreshes.

### Keybinding Summary

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
| `c` | Column selector |
| `y`/`Y` | Copy raw / format selection |
| `h`/`H` | Add highlight / highlight manager |
| `m` | Toggle bookmark |
| `'`/`"` | Next/prev bookmark |
| `M` | Bookmark manager |
| `S` | Stats summary |
| `]`/`[` | Relative time jump (forward/backward) |
| `Ctrl+]` | Toggle follow mode |
| `Ctrl+s` | Quick export filtered records to auto-named file |
| `Esc` | Close current overlay |
| `q` | Quit |
| `?` | Help |

## Change Log

| Date | Change |
|------|--------|
| 2026-02-20 | TUI log viewer full interaction design |
| 2026-02-21 | Architecture refactor to UiComponent trait + windows/widgets structure |
