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
    Stats,
    Category,
}

impl PanelId {
    /// Display name for the tab bar.
    pub fn name(self) -> &'static str {
        match self {
            PanelId::Detail => "Detail",
            PanelId::Region => "Region",
            PanelId::Stats => "Stats",
            PanelId::Category => "Category",
        }
    }

    /// All panels in tab order.
    pub fn all() -> &'static [PanelId] {
        &[
            PanelId::Detail,
            PanelId::Region,
            PanelId::Stats,
            PanelId::Category,
        ]
    }

    /// Next panel in tab order.
    pub fn next(self) -> PanelId {
        let all = Self::all();
        let idx = all.iter().position(|&p| p == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }

    /// Previous panel in tab order.
    pub fn prev(self) -> PanelId {
        let all = Self::all();
        let idx = all.iter().position(|&p| p == self).unwrap_or(0);
        all[(idx + all.len() - 1) % all.len()]
    }

    /// Default height strategy.
    pub fn default_height(self) -> PanelHeight {
        match self {
            PanelId::Detail => PanelHeight::FitContent,
            PanelId::Region => PanelHeight::Percentage(40),
            PanelId::Stats => PanelHeight::Percentage(40),
            PanelId::Category => PanelHeight::Percentage(40),
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

    /// Toggle panel expand/collapse without changing focus.
    /// If expanding a different panel, switch active but keep focus on log table.
    pub fn toggle_expand(&mut self, panel: PanelId) {
        if self.expanded && self.active == panel {
            // Collapse
            self.expanded = false;
            self.maximized = false;
            self.focus = PanelFocus::LogTable;
            tracing::debug!(?panel, "panel: collapsed → log table focus");
        } else {
            // Expand (or switch to different panel)
            self.active = panel;
            self.expanded = true;
            tracing::debug!(?panel, "panel: expanded (focus unchanged)");
        }
    }

    /// Focus down into panel content (expand if collapsed).
    pub fn focus_panel(&mut self) {
        self.expanded = true;
        self.focus = PanelFocus::PanelContent;
        tracing::debug!(active = ?self.active, "panel focus: entered panel content");
    }

    /// Focus back to log table.
    pub fn focus_log_table(&mut self) {
        self.focus = PanelFocus::LogTable;
        tracing::debug!("panel focus: returned to log table");
    }

    /// Switch to next panel tab.
    pub fn next_panel(&mut self) {
        self.active = self.active.next();
        tracing::debug!(active = ?self.active, "panel: switched to next tab");
    }

    /// Switch to previous panel tab.
    pub fn prev_panel(&mut self) {
        self.active = self.active.prev();
        tracing::debug!(active = ?self.active, "panel: switched to prev tab");
    }

    /// Toggle maximize.
    pub fn toggle_maximize(&mut self) {
        if self.expanded {
            self.maximized = !self.maximized;
            tracing::debug!(maximized = self.maximized, "panel: toggled maximize");
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

    /// Whether a specific panel is currently open (expanded and active).
    pub fn is_panel_open(&self, panel: PanelId) -> bool {
        self.expanded && self.active == panel
    }

    /// Whether a specific panel has keyboard focus.
    pub fn is_panel_focused(&self, panel: PanelId) -> bool {
        self.has_focus() && self.active == panel
    }
}
