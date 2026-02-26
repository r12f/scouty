//! Panel system — unified bottom panel framework.
//!
//! Panels render below the log table, sharing a tab bar.

#[cfg(test)]
#[path = "panel_tests.rs"]
mod panel_tests;

/// Trait for panel implementations.
///
/// Each panel provides metadata (name, shortcut, height strategy) and
/// rendering. Panels that need access to shared application state use
/// `render_with_app` for rendering while still implementing the trait
/// for registration and dispatch.
pub trait Panel {
    /// Display name for the tab bar (e.g. "Detail").
    fn name(&self) -> &str;

    /// Keyboard shortcut to open this panel directly (e.g. Some('\r') for Enter).
    /// Returns `None` if no direct shortcut.
    fn shortcut(&self) -> Option<char>;

    /// Default height strategy for this panel.
    fn default_height(&self) -> PanelHeight;

    /// Whether this panel has content available to display.
    fn is_available(&self) -> bool;

    /// Called when the log table cursor changes to a new index.
    fn on_log_cursor_changed(&mut self, index: usize);
}

/// Height strategy for a panel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelHeight {
    /// Fit content (dynamic), capped by max ratio.
    FitContent,
    /// Fixed percentage of terminal height.
    Percentage(u16),
}

/// Identifies a registered panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelId {
    Detail,
    Region,
}

impl PanelId {
    /// Display name for the tab bar.
    pub fn name(self) -> &'static str {
        match self {
            PanelId::Detail => "Detail",
            PanelId::Region => "Region",
        }
    }

    /// All panels in tab order.
    pub fn all() -> &'static [PanelId] {
        &[PanelId::Detail, PanelId::Region]
    }

    /// Next panel in tab order.
    pub fn next(self) -> PanelId {
        match self {
            PanelId::Detail => PanelId::Region,
            PanelId::Region => PanelId::Detail,
        }
    }

    /// Previous panel in tab order.
    pub fn prev(self) -> PanelId {
        match self {
            PanelId::Detail => PanelId::Region,
            PanelId::Region => PanelId::Detail,
        }
    }

    /// Default height strategy.
    pub fn default_height(self) -> PanelHeight {
        match self {
            PanelId::Detail => PanelHeight::FitContent,
            PanelId::Region => PanelHeight::Percentage(40),
        }
    }
}

/// Focus layer within the panel system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelFocus {
    /// Focus is on the log table (default).
    LogTable,
    /// Focus is on the panel content.
    PanelContent,
}

/// Panel system state stored in App.
#[derive(Debug, Clone)]
pub struct PanelState {
    /// Currently selected panel tab.
    pub active: PanelId,
    /// Whether the panel area is expanded (showing content).
    pub expanded: bool,
    /// Current focus layer.
    pub focus: PanelFocus,
    /// Whether the panel is maximized (fills log table + panel area).
    pub maximized: bool,
}

impl Default for PanelState {
    fn default() -> Self {
        Self {
            active: PanelId::Detail,
            expanded: false,
            focus: PanelFocus::LogTable,
            maximized: false,
        }
    }
}

impl PanelState {
    /// Open a specific panel and focus it.
    pub fn open(&mut self, panel: PanelId) {
        self.active = panel;
        self.expanded = true;
        self.focus = PanelFocus::PanelContent;
    }

    /// Close the panel (collapse).
    pub fn close(&mut self) {
        self.expanded = false;
        self.focus = PanelFocus::LogTable;
        self.maximized = false;
    }

    /// Focus down into panel content (expand if collapsed).
    pub fn focus_panel(&mut self) {
        self.expanded = true;
        self.focus = PanelFocus::PanelContent;
    }

    /// Focus back to log table.
    pub fn focus_log_table(&mut self) {
        self.focus = PanelFocus::LogTable;
    }

    /// Switch to next panel tab.
    pub fn next_panel(&mut self) {
        self.active = self.active.next();
    }

    /// Switch to previous panel tab.
    pub fn prev_panel(&mut self) {
        self.active = self.active.prev();
    }

    /// Toggle maximize.
    pub fn toggle_maximize(&mut self) {
        if self.expanded {
            self.maximized = !self.maximized;
        }
    }

    /// Whether panel content should be rendered.
    pub fn is_content_visible(&self) -> bool {
        self.expanded
    }

    /// Whether the panel has keyboard focus.
    pub fn has_focus(&self) -> bool {
        self.focus == PanelFocus::PanelContent
    }
}
