//! UI Architecture — Window Stack + Widget Tree framework.
//!
//! Phase 1: Core types only, no migration of existing code.
//! See spec: `crates/scouty-tui/spec/ui-architecture.md`

// Phase 1: framework is additive-only, not yet used by existing code.
#![allow(dead_code)]

#[cfg(test)]
#[path = "framework_tests.rs"]
mod framework_tests;

use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::Frame;

// ── Key/Window Actions ──────────────────────────────────────────────

/// Result of a widget handling a key event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// Input consumed — stop propagation.
    Handled,
    /// Input not consumed — bubble to parent.
    Unhandled,
}

/// Result of a window handling a key event.
pub enum WindowAction {
    /// Input consumed, no further action.
    Handled,
    /// Close this window (pop from stack).
    Close,
    /// Push a new window on top of the stack.
    Open(Box<dyn Window>),
    /// Window didn't handle it.
    Unhandled,
}

impl std::fmt::Debug for WindowAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Handled => write!(f, "Handled"),
            Self::Close => write!(f, "Close"),
            Self::Open(w) => write!(f, "Open({})", w.name()),
            Self::Unhandled => write!(f, "Unhandled"),
        }
    }
}

impl PartialEq for WindowAction {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Eq for WindowAction {}

// ── Widget Trait ─────────────────────────────────────────────────────

/// A composable UI element within a window.
///
/// Widgets form a tree; keyboard input flows from focused leaf → root
/// via event bubbling.
pub trait Widget {
    /// Child widgets (empty for leaf widgets).
    fn children(&self) -> &[Box<dyn Widget>];

    /// Mutable access to child widgets (for dispatching events).
    fn children_mut(&mut self) -> &mut [Box<dyn Widget>];

    /// Render this widget into the given area.
    fn render(&self, frame: &mut Frame, area: Rect);

    /// Handle a key event. Return `Handled` to stop propagation.
    fn handle_key(&mut self, event: KeyEvent) -> KeyAction;

    /// Whether this widget can receive focus.
    fn is_focusable(&self) -> bool;

    /// Display name for debugging/tracing.
    fn name(&self) -> &str {
        "Widget"
    }

    /// Return shortcut hints this widget handles.
    /// Each entry is `("key", "description")`.
    /// Status bar collects hints by walking focus path: focused → parent → root.
    fn shortcut_hints(&self) -> Vec<(&str, &str)> {
        Vec::new()
    }
}

// ── Window Trait ─────────────────────────────────────────────────────

/// A top-level UI surface. Windows are managed by the WindowStack.
pub trait Window {
    /// Display name (for tracing and debugging).
    fn name(&self) -> &str;

    /// Render this window into the given area.
    fn render(&self, frame: &mut Frame, area: Rect);

    /// Handle a key event. The window is responsible for dispatching
    /// to its widget tree (via FocusManager) and bubbling.
    fn handle_key(&mut self, event: KeyEvent) -> WindowAction;

    /// Return shortcut hints for the current focus state.
    /// Default: empty (windows that don't use widget trees override this).
    fn shortcut_hints(&self) -> Vec<(&str, &str)> {
        Vec::new()
    }
}

// ── WindowStack ─────────────────────────────────────────────────────

/// Manages a stack of windows. The topmost window owns focus and input.
pub struct WindowStack {
    windows: Vec<Box<dyn Window>>,
}

impl WindowStack {
    /// Create a new stack with a base window (the main window).
    pub fn new(base: Box<dyn Window>) -> Self {
        Self {
            windows: vec![base],
        }
    }

    /// Push a window onto the stack (it becomes the topmost/focused).
    pub fn push(&mut self, window: Box<dyn Window>) {
        tracing::info!(window = window.name(), "WindowStack: pushed window");
        self.windows.push(window);
    }

    /// Pop the topmost window. Returns `None` if only the base remains.
    pub fn pop(&mut self) -> Option<Box<dyn Window>> {
        if self.windows.len() <= 1 {
            tracing::warn!("WindowStack: cannot pop base window");
            return None;
        }
        let w = self.windows.pop();
        if let Some(ref w) = w {
            tracing::info!(window = w.name(), "WindowStack: popped window");
        }
        w
    }

    /// Reference to the topmost window.
    pub fn top(&self) -> &dyn Window {
        self.windows
            .last()
            .expect("stack must never be empty")
            .as_ref()
    }

    /// Mutable reference to the topmost window.
    pub fn top_mut(&mut self) -> &mut dyn Window {
        self.windows
            .last_mut()
            .expect("stack must never be empty")
            .as_mut()
    }

    /// Number of windows in the stack.
    pub fn len(&self) -> usize {
        self.windows.len()
    }

    /// Whether the stack has only the base window.
    pub fn is_base_only(&self) -> bool {
        self.windows.len() == 1
    }

    /// Render all windows bottom-to-top.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        for window in &self.windows {
            window.render(frame, area);
        }
    }

    /// Dispatch a key event to the topmost window.
    /// Handles `Close` and `Open` actions automatically.
    pub fn handle_key(&mut self, event: KeyEvent) -> WindowAction {
        let action = self.top_mut().handle_key(event);
        match action {
            WindowAction::Close => {
                self.pop();
                WindowAction::Handled
            }
            WindowAction::Open(window) => {
                self.push(window);
                WindowAction::Handled
            }
            other => other,
        }
    }
}

// ── FocusManager ────────────────────────────────────────────────────

/// Tracks which widget in a tree has focus.
///
/// `focus_path` is a list of child indices from the root to the focused widget.
/// E.g., `[1, 0]` means root's child at index 1, then that widget's child at index 0.
pub struct FocusManager {
    /// Path from root to focused widget (child indices).
    focus_path: Vec<usize>,
}

impl FocusManager {
    /// Create a new FocusManager with no initial focus (empty path = root).
    /// Call `focus_next()` to move focus to the first focusable widget.
    pub fn new() -> Self {
        Self {
            focus_path: Vec::new(),
        }
    }

    /// Create with an explicit initial path.
    pub fn with_path(path: Vec<usize>) -> Self {
        Self { focus_path: path }
    }

    /// Current focus path.
    pub fn path(&self) -> &[usize] {
        &self.focus_path
    }

    /// Set the focus path directly.
    pub fn set_path(&mut self, path: Vec<usize>) {
        tracing::debug!(path = ?path, "FocusManager: focus path changed");
        self.focus_path = path;
    }

    /// Advance focus to the next focusable widget (Tab).
    /// Returns true if focus changed.
    pub fn next(&mut self, root: &dyn Widget) -> bool {
        let focusables = Self::collect_focusable_paths(root);
        if focusables.is_empty() {
            return false;
        }
        let current_idx = focusables.iter().position(|p| p == &self.focus_path);
        let next_idx = match current_idx {
            Some(i) => (i + 1) % focusables.len(),
            None => 0,
        };
        let new_path = focusables[next_idx].clone();
        if new_path != self.focus_path {
            tracing::debug!(from = ?self.focus_path, to = ?new_path, "FocusManager: Tab → next");
            self.focus_path = new_path;
            true
        } else {
            false
        }
    }

    /// Move focus to the previous focusable widget (Shift+Tab).
    /// Returns true if focus changed.
    pub fn prev(&mut self, root: &dyn Widget) -> bool {
        let focusables = Self::collect_focusable_paths(root);
        if focusables.is_empty() {
            return false;
        }
        let current_idx = focusables.iter().position(|p| p == &self.focus_path);
        let prev_idx = match current_idx {
            Some(i) => (i + focusables.len() - 1) % focusables.len(),
            None => focusables.len() - 1,
        };
        let new_path = focusables[prev_idx].clone();
        if new_path != self.focus_path {
            tracing::debug!(from = ?self.focus_path, to = ?new_path, "FocusManager: Shift+Tab → prev");
            self.focus_path = new_path;
            true
        } else {
            false
        }
    }

    /// Dispatch a key event through the widget tree using event bubbling.
    ///
    /// Sends the event to the focused widget first. If unhandled, bubbles
    /// up through each ancestor until handled or the root is reached.
    pub fn dispatch_key(&self, root: &mut dyn Widget, event: KeyEvent) -> KeyAction {
        // Build the chain of widgets from root to focused leaf
        // We process from leaf to root (event bubbling)
        self.dispatch_at_depth(root, event, 0)
    }

    /// Recursively dispatch: walk to focused leaf, then bubble up.
    fn dispatch_at_depth(
        &self,
        widget: &mut dyn Widget,
        event: KeyEvent,
        depth: usize,
    ) -> KeyAction {
        // If we haven't reached the end of the focus path, go deeper
        if depth < self.focus_path.len() {
            let child_idx = self.focus_path[depth];
            let children = widget.children_mut();
            if child_idx < children.len() {
                let result = self.dispatch_at_depth(children[child_idx].as_mut(), event, depth + 1);
                if result == KeyAction::Handled {
                    return KeyAction::Handled;
                }
            }
        }
        // Either we're at the focused widget, or a child didn't handle it — try this widget
        let result = widget.handle_key(event);
        if result == KeyAction::Handled {
            tracing::debug!(widget = widget.name(), depth, "key handled by widget");
        }
        result
    }

    /// Collect all focusable widget paths in depth-first order.
    fn collect_focusable_paths(root: &dyn Widget) -> Vec<Vec<usize>> {
        let mut result = Vec::new();
        Self::collect_recursive(root, &mut Vec::new(), &mut result);
        result
    }

    fn collect_recursive(
        widget: &dyn Widget,
        current_path: &mut Vec<usize>,
        result: &mut Vec<Vec<usize>>,
    ) {
        // Check this widget (skip root — root's focusability is at path [])
        if !current_path.is_empty() && widget.is_focusable() {
            result.push(current_path.clone());
        }
        // Recurse into children
        for (i, child) in widget.children().iter().enumerate() {
            current_path.push(i);
            Self::collect_recursive(child.as_ref(), current_path, result);
            current_path.pop();
        }
    }
    /// Collect shortcut hints by walking from focused widget up to root.
    /// Returns hints in order: focused widget first, then parents up to root.
    pub fn collect_hints(&self, root: &dyn Widget) -> Vec<(String, String)> {
        let mut hints = Vec::new();
        let mut widgets_on_path: Vec<&dyn Widget> = Vec::new();

        // Walk down the focus path to collect widgets
        let mut current: &dyn Widget = root;
        widgets_on_path.push(current);
        for &idx in &self.focus_path {
            let children = current.children();
            if idx < children.len() {
                current = children[idx].as_ref();
                widgets_on_path.push(current);
            } else {
                break;
            }
        }

        // Collect hints from focused (deepest) to root
        for widget in widgets_on_path.iter().rev() {
            for (k, v) in widget.shortcut_hints() {
                hints.push((k.to_string(), v.to_string()));
            }
        }

        hints
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

// ── TabbedContainer ─────────────────────────────────────────────────

/// Configuration for a single tab in a `TabbedContainer`.
pub struct TabEntry {
    /// Display name shown in the tab bar.
    pub name: String,
    /// Optional shortcut key character (e.g. 'd' for Detail).
    pub shortcut: Option<char>,
    /// The child widget for this tab.
    pub widget: Box<dyn Widget>,
}

/// A generic container that manages multiple child widgets as tabs.
///
/// - Renders a tab bar with names + active highlight
/// - Only the active tab's widget is rendered in the content area
/// - `Ctrl+←`/`Ctrl+→` switch tabs
/// - Delegates other keys to the active tab's child widget
pub struct TabbedContainer {
    pub tabs: Vec<TabEntry>,
    pub active: usize,
}

impl TabbedContainer {
    pub fn new(tabs: Vec<TabEntry>) -> Self {
        Self { tabs, active: 0 }
    }

    /// Switch to the next tab (wrapping).
    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active = (self.active + 1) % self.tabs.len();
            tracing::debug!(
                active = self.active,
                name = self.tabs[self.active].name,
                "TabbedContainer: next_tab"
            );
        }
    }

    /// Switch to the previous tab (wrapping).
    pub fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active = if self.active == 0 {
                self.tabs.len() - 1
            } else {
                self.active - 1
            };
            tracing::debug!(
                active = self.active,
                name = self.tabs[self.active].name,
                "TabbedContainer: prev_tab"
            );
        }
    }

    /// Switch to a tab by index.
    pub fn set_active(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active = index;
        }
    }

    /// The name of the currently active tab.
    pub fn active_name(&self) -> &str {
        self.tabs.get(self.active).map_or("", |t| &t.name)
    }

    /// Number of tabs.
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }
}

impl Widget for TabbedContainer {
    fn name(&self) -> &str {
        "TabbedContainer"
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        // We can't return &[Box<dyn Widget>] from TabEntry vec directly,
        // so delegate to children_mut pattern. For focus enumeration,
        // the FocusManager will need the active child only.
        &[]
    }

    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] {
        &mut []
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        // Tab bar takes 1 line at top, content below.
        if area.height < 2 || self.tabs.is_empty() {
            return;
        }

        let tab_bar_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };
        let content_area = Rect {
            x: area.x,
            y: area.y + 1,
            width: area.width,
            height: area.height - 1,
        };

        // Render tab bar
        use ratatui::style::{Color, Modifier, Style};
        use ratatui::text::{Line, Span};
        use ratatui::widgets::Paragraph;

        let mut spans = Vec::new();
        for (i, tab) in self.tabs.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" │ "));
            }
            let style = if i == self.active {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if i == self.active { "▸ " } else { "  " };
            spans.push(Span::styled(format!("{}{}", prefix, tab.name), style));
        }
        frame.render_widget(Paragraph::new(Line::from(spans)), tab_bar_area);

        // Render active tab content
        if let Some(tab) = self.tabs.get(self.active) {
            tab.widget.render(frame, content_area);
        }
    }

    fn handle_key(&mut self, event: KeyEvent) -> KeyAction {
        use crossterm::event::{KeyCode, KeyModifiers};

        // Ctrl+Right → next tab
        if event.modifiers.contains(KeyModifiers::CONTROL) && event.code == KeyCode::Right {
            self.next_tab();
            return KeyAction::Handled;
        }
        // Ctrl+Left → prev tab
        if event.modifiers.contains(KeyModifiers::CONTROL) && event.code == KeyCode::Left {
            self.prev_tab();
            return KeyAction::Handled;
        }

        // Shortcut keys for direct tab activation
        if let KeyCode::Char(c) = event.code {
            if event.modifiers.is_empty() || event.modifiers == KeyModifiers::SHIFT {
                for (i, tab) in self.tabs.iter().enumerate() {
                    if tab.shortcut == Some(c) {
                        self.active = i;
                        tracing::debug!(tab = tab.name, "TabbedContainer: shortcut activated");
                        return KeyAction::Handled;
                    }
                }
            }
        }

        // Delegate to active tab's widget
        if let Some(tab) = self.tabs.get_mut(self.active) {
            return tab.widget.handle_key(event);
        }

        KeyAction::Unhandled
    }

    fn is_focusable(&self) -> bool {
        true
    }

    fn shortcut_hints(&self) -> Vec<(&str, &str)> {
        vec![("Tab/S-Tab", "Switch")]
    }
}
