//! MainWindow — the root window at the bottom of the WindowStack.
//!
//! Wraps `App` and dispatches keys through the new architecture.
//! Phase 2: LogTable keys go through LogTableWidget, everything else
//! stays in MainWindow (will be migrated in later phases).

#[cfg(test)]
#[path = "main_window_tests.rs"]
mod main_window_tests;

use crate::app::{App, InputMode};
use crate::keybinding::{Action, Keymap};
use crate::panel::PanelId;
use crate::ui::framework::{KeyAction, OverlayStack, Window, WindowAction};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::Frame;

/// Panel key handler function signature.
type PanelKeyHandler = fn(&mut App, KeyEvent) -> KeyAction;

/// Registry of panel key handlers.
///
/// Adding a new panel requires only adding an entry here — no changes
/// to the dispatch logic in `handle_normal_key`.
fn panel_key_handler(panel: PanelId) -> PanelKeyHandler {
    match panel {
        PanelId::Detail => crate::ui::widgets::detail_panel_keys::handle_key,
        PanelId::Region => crate::ui::widgets::region_panel_keys::handle_key,
        PanelId::Category => crate::ui::widgets::category_panel_keys::handle_key,
        PanelId::Stats => crate::ui::widgets::stats_panel_keys::handle_key,
    }
}

/// Dispatch a key event to the currently focused panel's handler.
fn dispatch_panel_key(app: &mut App, key: KeyEvent) -> KeyAction {
    let handler = panel_key_handler(app.panel_state.active);
    handler(app, key)
}

/// Registry of panel shortcut hint providers.
pub fn panel_shortcut_hints(panel: PanelId) -> Vec<(&'static str, &'static str)> {
    match panel {
        PanelId::Detail => crate::ui::widgets::detail_panel_keys::shortcut_hints(),
        PanelId::Region => crate::ui::widgets::region_panel_keys::shortcut_hints(),
        PanelId::Category => crate::ui::widgets::category_panel_keys::shortcut_hints(),
        PanelId::Stats => crate::ui::widgets::stats_panel_keys::shortcut_hints(),
    }
}

/// The root window managing the main TUI view.
pub struct MainWindow {
    pub app: App,
    pub keymap: Keymap,
    pub overlay_stack: OverlayStack,
}

impl MainWindow {
    pub fn new(app: App, keymap: Keymap) -> Self {
        Self {
            app,
            keymap,
            overlay_stack: OverlayStack::new(),
        }
    }

    /// Handle panel system keys (Tab/BackTab, z).
    fn handle_panel_keys(&mut self, key: KeyEvent) -> KeyAction {
        let handled = match key.code {
            KeyCode::Tab if key.modifiers.is_empty() => {
                if self.app.panel_state.focus == crate::panel::PanelFocus::LogTable {
                    self.app.panel_state.active = crate::panel::PanelId::all()[0];
                    self.app.panel_state.focus_panel();
                    tracing::debug!(active = ?self.app.panel_state.active, "Tab: log table → panel");
                } else {
                    let all = crate::panel::PanelId::all();
                    if self.app.panel_state.active == *all.last().unwrap() {
                        self.app.panel_state.focus_log_table();
                        tracing::debug!("Tab: last panel → log table");
                    } else {
                        self.app.panel_state.next_panel();
                        tracing::debug!(active = ?self.app.panel_state.active, "Tab: → next panel");
                    }
                }
                true
            }
            KeyCode::BackTab | KeyCode::Tab
                if key.code == KeyCode::BackTab
                    || key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::SHIFT) =>
            {
                if self.app.panel_state.focus == crate::panel::PanelFocus::LogTable {
                    let all = crate::panel::PanelId::all();
                    let target = *all.last().unwrap();
                    tracing::info!(target_panel = ?target, "BackTab: entering panels from log table (reverse)");
                    self.app.panel_state.active = target;
                    self.app.panel_state.focus_panel();
                    tracing::debug!(active = ?self.app.panel_state.active, "Shift+Tab: log table → panel");
                } else {
                    let all = crate::panel::PanelId::all();
                    if self.app.panel_state.active == all[0] {
                        self.app.panel_state.focus_log_table();
                        tracing::debug!("Shift+Tab: first panel → log table");
                    } else {
                        self.app.panel_state.prev_panel();
                        tracing::debug!(active = ?self.app.panel_state.active, "Shift+Tab: → prev panel");
                    }
                }
                true
            }
            KeyCode::Char('z') if key.modifiers.is_empty() && self.app.panel_state.expanded => {
                self.app.panel_state.toggle_maximize();
                true
            }
            KeyCode::Esc if self.app.panel_state.has_focus() => {
                self.app.panel_state.focus_log_table();
                tracing::debug!("Esc: panel → log table");
                true
            }
            _ => false,
        };
        if handled {
            KeyAction::Handled
        } else {
            KeyAction::Unhandled
        }
    }

    /// Handle Normal-mode log table keys via the keymap.
    /// Returns `Some(true)` if should quit, `Some(false)` if handled, `None` if unhandled.
    fn handle_log_table_key(&mut self, key: KeyEvent) -> Option<bool> {
        let action = self.keymap.action(&key)?;
        tracing::debug!(?action, "MainWindow: action dispatched");
        match action {
            Action::Quit => return Some(true),
            Action::CloseDetail => {
                if self
                    .app
                    .panel_state
                    .is_panel_open(crate::panel::PanelId::Detail)
                {
                    self.app.panel_state.close();
                }
            }
            Action::MoveDown => self.app.select_down(1),
            Action::MoveUp => self.app.select_up(1),
            Action::PageDown => self.app.page_down(),
            Action::PageUp => self.app.page_up(),
            Action::ScrollToTop => self.app.scroll_to_top(),
            Action::ScrollToBottom => self.app.scroll_to_bottom(),
            Action::ToggleDetail => self.app.toggle_detail(),
            Action::Filter => {
                self.app.input_mode = InputMode::Filter;
                self.overlay_stack.push(Box::new(
                    crate::ui::windows::overlay_adapters::FilterOverlay::new(),
                ));
            }
            Action::Search => {
                self.app.input_mode = InputMode::Search;
                self.overlay_stack.push(Box::new(
                    crate::ui::windows::overlay_adapters::SearchOverlay::new(),
                ));
            }
            Action::JumpForward => {
                self.app.input_mode = InputMode::JumpForward;
                self.app.time_input.clear();
                self.overlay_stack.push(Box::new(
                    crate::ui::windows::overlay_adapters::JumpOverlay::new(true),
                ));
            }
            Action::JumpBackward => {
                self.app.input_mode = InputMode::JumpBackward;
                self.app.time_input.clear();
                self.overlay_stack.push(Box::new(
                    crate::ui::windows::overlay_adapters::JumpOverlay::new(false),
                ));
            }
            Action::QuickExclude => {
                self.app.input_mode = InputMode::QuickExclude;
                self.app.quick_filter_input.clear();
                self.overlay_stack.push(Box::new(
                    crate::ui::windows::overlay_adapters::QuickFilterOverlay::new(true),
                ));
            }
            Action::QuickInclude => {
                self.app.input_mode = InputMode::QuickInclude;
                self.app.quick_filter_input.clear();
                self.overlay_stack.push(Box::new(
                    crate::ui::windows::overlay_adapters::QuickFilterOverlay::new(false),
                ));
            }
            Action::FieldExclude => {
                self.app.open_field_filter(true);
                self.overlay_stack.push(Box::new(
                    crate::ui::windows::overlay_adapters::FieldFilterOverlay::new(),
                ));
            }
            Action::FieldInclude => {
                self.app.open_field_filter(false);
                self.overlay_stack.push(Box::new(
                    crate::ui::windows::overlay_adapters::FieldFilterOverlay::new(),
                ));
            }
            Action::FilterManager => {
                self.app.input_mode = InputMode::FilterManager;
                self.app.filter_manager_cursor = 0;
                self.overlay_stack.push(Box::new(
                    crate::ui::windows::overlay_adapters::FilterManagerOverlay::new(),
                ));
            }
            Action::LevelFilter => {
                self.app.input_mode = InputMode::LevelFilter;
                self.app.level_filter_cursor = self
                    .app
                    .level_filter
                    .map(|l| (l.as_number() - 1) as usize)
                    .unwrap_or(0);
                self.overlay_stack.push(Box::new(
                    crate::ui::windows::overlay_adapters::LevelFilterOverlay::new(),
                ));
            }
            Action::DensityCycle => self.app.cycle_density_source(),
            Action::DensitySelector => {
                self.app.density_selector_cursor = self
                    .app
                    .density_source_options()
                    .iter()
                    .position(|s| *s == self.app.density_source)
                    .unwrap_or(0);
                self.app.input_mode = InputMode::DensitySelector;
                self.overlay_stack.push(Box::new(
                    crate::ui::windows::overlay_adapters::DensitySelectorOverlay::new(),
                ));
            }
            Action::GotoLine => {
                self.app.input_mode = InputMode::GotoLine;
                self.app.goto_input.clear();
                self.overlay_stack.push(Box::new(
                    crate::ui::windows::overlay_adapters::GotoLineOverlay::new(),
                ));
            }
            Action::ToggleFollow => self.app.toggle_follow(),
            Action::NextMatch => self.app.next_search_match(),
            Action::PrevMatch => self.app.prev_search_match(),
            Action::CopyRaw => {
                if let Some(text) = self.app.copy_raw() {
                    crate::app::osc52_copy(&text);
                }
            }
            Action::CopyFormat => {
                self.app.input_mode = InputMode::CopyFormat;
                self.app.copy_format_cursor = 0;
                self.overlay_stack.push(Box::new(
                    crate::ui::windows::overlay_adapters::CopyFormatOverlay::new(),
                ));
            }
            Action::Save => {
                self.app.input_mode = InputMode::SaveDialog;
                self.overlay_stack.push(Box::new(
                    crate::ui::windows::overlay_adapters::SaveDialogOverlay::new(),
                ));
            }
            Action::ColumnSelector => {
                self.app.input_mode = InputMode::ColumnSelector;
                self.app.column_config.cursor = 0;
                self.overlay_stack.push(Box::new(
                    crate::ui::windows::overlay_adapters::ColumnSelectorOverlay::new(),
                ));
            }
            Action::Help => {
                self.app.input_mode = InputMode::Help;
                self.app.help_scroll = 0;
                self.overlay_stack.push(Box::new(
                    crate::ui::windows::help_window::HelpOverlay::new(&self.app),
                ));
            }
            Action::Command => {
                self.app.command_input.clear();
                self.app.input_mode = InputMode::Command;
                self.overlay_stack.push(Box::new(
                    crate::ui::windows::overlay_adapters::CommandOverlay::new(),
                ));
            }
            Action::AddHighlight => {
                self.app.input_mode = InputMode::Highlight;
                self.app.highlight_input.clear();
                self.overlay_stack.push(Box::new(
                    crate::ui::windows::overlay_adapters::HighlightOverlay::new(),
                ));
            }
            Action::HighlightManager => {
                self.app.input_mode = InputMode::HighlightManager;
                self.app.highlight_manager_cursor = 0;
                self.overlay_stack.push(Box::new(
                    crate::ui::windows::overlay_adapters::HighlightManagerOverlay::new(),
                ));
            }
            Action::ToggleBookmark => self.app.toggle_bookmark(),
            Action::NextBookmark => self.app.jump_next_bookmark(),
            Action::PrevBookmark => self.app.jump_prev_bookmark(),
            Action::BookmarkManager => {
                self.app.input_mode = InputMode::BookmarkManager;
                self.app.bookmark_manager_cursor = 0;
                self.overlay_stack.push(Box::new(
                    crate::ui::windows::overlay_adapters::BookmarkManagerOverlay::new(),
                ));
            }
            Action::RegionManager => {
                self.app
                    .panel_state
                    .toggle_expand(crate::panel::PanelId::Region);
            }
            Action::NextRegion => {
                if let Some(record_idx) = self.app.filtered_indices.get(self.app.selected) {
                    if let Some(region) = self
                        .app
                        .regions
                        .regions()
                        .iter()
                        .find(|r| r.start_index > *record_idx)
                    {
                        self.app.jump_to_record_index(region.start_index);
                    }
                }
            }
            Action::Stats => {
                self.app
                    .panel_state
                    .toggle_expand(crate::panel::PanelId::Stats);
            }
            Action::Category => {
                self.app
                    .panel_state
                    .toggle_expand(crate::panel::PanelId::Category);
            }
        }
        Some(false)
    }

    /// Handle a key event in Normal mode.
    /// Returns `WindowAction::Close` if the app should quit.
    pub fn handle_normal_key(&mut self, key: KeyEvent) -> WindowAction {
        // 1. Focused panel gets priority (dispatched via registry, not inline match)
        if self.app.panel_state.has_focus()
            && dispatch_panel_key(&mut self.app, key) == KeyAction::Handled
        {
            return WindowAction::Handled;
        }

        // 2. Panel system keys (Tab/Shift+Tab/z/Esc)
        if self.handle_panel_keys(key) == KeyAction::Handled {
            return WindowAction::Handled;
        }

        // 3. Global keys (quit) — must work regardless of panel focus
        if self.keymap.action(&key) == Some(Action::Quit) {
            return WindowAction::Close;
        }

        // 4. If a panel has focus, do NOT fall through to log table keys
        if self.app.panel_state.has_focus() {
            tracing::debug!(
                panel = ?self.app.panel_state.active,
                key_code = ?key.code,
                "key not handled by focused panel, ignoring"
            );
            return WindowAction::Handled;
        }

        // 5. Log table / global keys via keymap (only when log table has focus)
        match self.handle_log_table_key(key) {
            Some(true) => WindowAction::Close,
            Some(false) => WindowAction::Handled,
            None => WindowAction::Unhandled,
        }
    }

    /// Handle a key event for overlay input modes (Filter, Search, etc.).
    /// Returns `true` if should quit.
    /// Legacy overlay key handler — now all modes dispatched via overlay_stack.
    /// This fallback handles the case where overlay_stack is empty but
    /// input_mode is non-Normal (shouldn't happen, but safety).
    pub fn handle_overlay_key(&mut self, _key: KeyEvent) -> bool {
        // All modes are now dispatched via overlay_stack.
        // If we reach here, something went wrong — reset to Normal.
        tracing::warn!(
            mode = ?self.app.input_mode,
            "handle_overlay_key reached unexpectedly, resetting to Normal"
        );
        self.app.input_mode = InputMode::Normal;
        false
    }
}

impl Window for MainWindow {
    fn name(&self) -> &str {
        "MainWindow"
    }

    fn render(&self, _frame: &mut Frame, _area: Rect) {
        // Rendering goes through the legacy path via main_window.app directly
        // (MainWindow.render() is not used — the main loop calls ui::render directly)
    }

    fn handle_key(&mut self, event: KeyEvent) -> WindowAction {
        self.app.clear_status();

        tracing::debug!(
            key_code = ?event.code,
            modifiers = ?event.modifiers,
            input_mode = ?self.app.input_mode,
            overlay_active = !self.overlay_stack.is_empty(),
            "MainWindow: key event"
        );

        // Overlay stack gets priority — input isolation
        if !self.overlay_stack.is_empty() {
            self.overlay_stack.handle_key(&mut self.app, event);

            // After overlay close, check if a transition was requested
            // (e.g., FilterManager → SavePreset/LoadPreset)
            if self.overlay_stack.is_empty() && self.app.input_mode != InputMode::Normal {
                match self.app.input_mode {
                    InputMode::SavePreset => {
                        self.overlay_stack.push(Box::new(
                            crate::ui::windows::overlay_adapters::SavePresetOverlay::new(),
                        ));
                    }
                    InputMode::LoadPreset => {
                        self.overlay_stack.push(Box::new(
                            crate::ui::windows::overlay_adapters::LoadPresetOverlay::new(),
                        ));
                    }
                    _ => {}
                }
            }

            // Check if command mode triggered quit
            if self.app.should_quit {
                return WindowAction::Close;
            }

            return WindowAction::Handled;
        }

        match self.app.input_mode {
            InputMode::Normal => self.handle_normal_key(event),
            _ => {
                if self.handle_overlay_key(event) {
                    WindowAction::Close // quit
                } else {
                    WindowAction::Handled
                }
            }
        }
    }

    fn shortcut_hints(&self) -> Vec<(&str, &str)> {
        // Overlay gets priority for hints
        if let Some(overlay) = self.overlay_stack.top() {
            return overlay.shortcut_hints();
        }

        let panel_focused = self.app.panel_state.expanded
            && self.app.panel_state.focus == crate::panel::PanelFocus::PanelContent;

        if panel_focused {
            // Collect panel-specific hints via registry, then common panel hints
            let mut hints: Vec<(&str, &str)> = panel_shortcut_hints(self.app.panel_state.active);
            // Common panel hints from MainWindow
            hints.push(("Tab/S-Tab", "Switch"));
            hints.push(("z", "Max"));
            hints.push(("Esc", "Close"));
            hints
        } else if self.app.follow_mode {
            vec![("Ctrl+]", "Stop Follow"), ("?", "Help")]
        } else {
            vec![
                ("j/k", "↑↓"),
                ("/", "Search"),
                ("f", "Filter"),
                ("-/=", "Exclude/Include"),
                ("Enter", "Detail"),
                ("?", "Help"),
            ]
        }
    }
}
