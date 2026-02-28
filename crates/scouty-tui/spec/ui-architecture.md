# UI Architecture — Window Stack + Widget Tree

## Overview

Redesign the TUI UI layer into a two-level architecture:
1. **Window Stack** — a stack of windows; the topmost window always owns focus
2. **Widget Tree** — each window contains a tree of widgets; keyboard input flows from the focused widget upward via event bubbling

This replaces the current ad-hoc dispatch (`if active_window → else if panel_focus → else log_table`) with a structured, extensible model.

## Background & Goals

**Why:** The current event dispatch is a flat `if-else` chain that grows with every new window/panel. Adding new panels or dialogs requires touching the central dispatch logic, which is fragile and hard to reason about.

**Goals:**
- Keyboard input always goes to the topmost window's focused widget — no leaking to lower layers
- Event bubbling provides a clean "handled or not" pattern — no need for a central dispatcher to know every widget's shortcuts
- Adding a new window or widget requires zero changes to the dispatch framework
- Windows and widgets are composable and self-contained

## Design

### Window Stack

```
┌─────────────────────────────────┐
│  Help Window (topmost = focus)  │  ← KeyEvent goes here
├─────────────────────────────────┤
│  Main Window (background)       │  ← Does NOT receive input
└─────────────────────────────────┘
```

- The stack is a `Vec<Box<dyn Window>>`, ordered bottom to top
- **Only the topmost window receives keyboard input** — input is never forwarded to lower windows
- Push a window → it gets focus. Pop a window → focus returns to the one below
- The **Main Window** (log table + panels + status bar) is always at the bottom of the stack and is never popped
- Overlay windows (Help, Column Selector, Filter Manager, etc.) are pushed on top when opened

```rust
struct WindowStack {
    windows: Vec<Box<dyn Window>>,
}

impl WindowStack {
    fn push(&mut self, window: Box<dyn Window>);
    fn pop(&mut self) -> Option<Box<dyn Window>>;
    fn top(&self) -> &dyn Window;
    fn top_mut(&mut self) -> &mut dyn Window;
}
```

### Window Trait

```rust
trait Window {
    fn name(&self) -> &str;
    fn render(&self, frame: &mut Frame, area: Rect);
    fn handle_key(&mut self, event: KeyEvent) -> WindowAction;
}

enum WindowAction {
    Handled,          // Input consumed, no further action
    Close,            // Close this window (pop from stack)
    Open(Box<dyn Window>),  // Push a new window on top
    Unhandled,        // Window didn't handle it (for future use)
}
```

### Widget Tree (within each Window)

Each window contains a **tree of widgets** with a single root (the window itself):

```
Main Window (root)
├── LogTableWidget (focusable)
├── PanelManager
│   ├── DetailPanelWidget (focusable)
│   ├── RegionPanelWidget (focusable)
│   └── StatsPanelWidget (focusable)
├── SearchInputWidget (focusable, shown when active)
├── FilterInputWidget (focusable, shown when active)
└── StatusBarWidget (not focusable)
```

- Each widget has a list of child widgets
- The tree tracks which widget currently has focus
- **Keyboard input is sent to the focused widget first**
- If the focused widget handles it → done (stop propagation)
- If not handled → bubble up to parent widget → continue until root
- If root doesn't handle it either → `WindowAction::Unhandled`

```rust
trait Widget {
    fn children(&self) -> &[Box<dyn Widget>];
    fn render(&self, frame: &mut Frame, area: Rect);
    fn handle_key(&mut self, event: KeyEvent) -> KeyAction;
    fn is_focusable(&self) -> bool;
}

enum KeyAction {
    Handled,    // Consumed, stop propagation
    Unhandled,  // Not consumed, bubble to parent
}
```

### Focus Management

```rust
struct FocusManager {
    /// Path from root to the currently focused widget (indices into children)
    focus_path: Vec<usize>,
}
```

- `Tab` / `Shift+Tab` are handled by the **parent** widget that manages focus among its children (e.g., Main Window cycles between LogTable and PanelManager children)
- `Esc` on overlay windows → `WindowAction::Close` (pop from stack)
- Focus transfer shortcuts (`Ctrl+↓`, `Ctrl+↑`) are handled by Main Window, which updates `focus_path`

### Event Flow Example

**User presses `j` while Detail Panel has focus:**

```
1. WindowStack.top() → Main Window (no overlay open)
2. Main Window's focused widget → DetailPanelWidget
3. DetailPanelWidget.handle_key('j') → Handled (scroll down)
4. Done — no bubbling needed
```

**User presses `q` while Detail Panel has focus:**

```
1. WindowStack.top() → Main Window
2. Main Window's focused widget → DetailPanelWidget
3. DetailPanelWidget.handle_key('q') → Unhandled (Detail doesn't use 'q')
4. Bubble to parent → PanelManager.handle_key('q') → Unhandled
5. Bubble to root → MainWindow.handle_key('q') → Handled (quit app)
```

**User presses `Esc` while Help Window is open:**

```
1. WindowStack.top() → Help Window
2. HelpWindow.handle_key(Esc) → WindowAction::Close
3. WindowStack.pop() → Help Window removed
4. Focus returns to Main Window
```

### Rendering

All windows in the stack are rendered bottom-to-top:
1. Main Window renders full screen
2. Overlay windows render on top (centered popup, etc.)

This gives natural visual layering — lower windows are visible behind overlays.

### Migration Path

The existing windows and widgets map cleanly to this architecture:

| Current | New |
|---------|-----|
| `main.rs` giant match | `MainWindow.handle_key()` + widget bubbling |
| `windows/*.rs` (Help, etc.) | Overlay `Window` implementations, pushed onto stack |
| `widgets/log_table_widget.rs` | `Widget` child of Main Window |
| `widgets/panel_manager.rs` | `Widget` with Detail/Region/Stats as children |
| `widgets/search_input_widget.rs` | `Widget` child of Main Window (focusable when active) |

## Acceptance Criteria

- [ ] `WindowStack` manages window lifecycle (push/pop/render)
- [ ] Only topmost window receives keyboard input
- [ ] Each window has a widget tree with focus tracking
- [ ] Keyboard events bubble from focused widget → parent → root
- [ ] `Tab`/`Shift+Tab` cycle focus within Main Window's widget tree
- [ ] Overlay windows (Help, Column Selector, etc.) pushed/popped correctly
- [ ] Existing keybindings work identically after migration
- [ ] No regression in test suite

## Out of Scope

- Mouse input handling (future)
- Multi-window split views (future)
- Async/background window updates (current sync model is sufficient)

## Open Questions

(None)

## Change Log

| Date | Change |
|------|--------|
| 2026-02-28 | Initial spec — window stack + widget tree architecture |
