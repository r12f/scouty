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
use crate::ui::framework::{KeyAction, Window, WindowAction};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::Frame;

/// The root window managing the main TUI view.
pub struct MainWindow {
    pub app: App,
    pub keymap: Keymap,
}

impl MainWindow {
    pub fn new(app: App, keymap: Keymap) -> Self {
        Self { app, keymap }
    }

    /// Handle keys when detail tree has focus.
    fn handle_detail_tree_key(&mut self, key: KeyEvent) -> KeyAction {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let handled = match key.code {
            KeyCode::Char('j') | KeyCode::Down if !ctrl => {
                self.app.detail_tree_move_down();
                true
            }
            KeyCode::Char('k') | KeyCode::Up if !ctrl => {
                self.app.detail_tree_move_up();
                true
            }
            KeyCode::Char('l') | KeyCode::Enter => {
                self.app.detail_tree_toggle();
                true
            }
            KeyCode::Char('h') => {
                self.app.detail_tree_collapse_or_parent();
                true
            }
            KeyCode::Char('H') => {
                self.app.detail_tree_collapse_all();
                true
            }
            KeyCode::Char('L') => {
                self.app.detail_tree_expand_all();
                true
            }
            KeyCode::Char('f') => {
                self.app.detail_tree_quick_filter();
                true
            }
            KeyCode::Esc => {
                self.app.detail_tree_focus = false;
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

    /// Handle keys when region panel has focus.
    fn handle_region_panel_key(&mut self, key: KeyEvent) -> KeyAction {
        use crate::ui::widgets::region_panel_widget::RegionPanelWidget;

        let entries = RegionPanelWidget::build_entries(&self.app);
        let max_cursor = entries.len().saturating_sub(1);
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        let handled = match key.code {
            KeyCode::Char('j') | KeyCode::Down if !ctrl => {
                if self.app.region_manager_cursor < max_cursor {
                    self.app.region_manager_cursor += 1;
                }
                true
            }
            KeyCode::Char('k') | KeyCode::Up if !ctrl => {
                if self.app.region_manager_cursor > 0 {
                    self.app.region_manager_cursor -= 1;
                }
                true
            }
            KeyCode::Enter => {
                if let Some(entry) = entries.get(self.app.region_manager_cursor) {
                    self.app.jump_to_record_index(entry.start_index);
                    self.app.panel_state.focus_log_table();
                }
                true
            }
            KeyCode::Char('f') => {
                if let Some(entry) = entries.get(self.app.region_manager_cursor) {
                    let expr = format!("_region_type == \"{}\"", entry.definition_name);
                    self.app.add_filter_expr(&expr);
                }
                true
            }
            KeyCode::Char('t') => {
                if let Some(entry) = entries.get(self.app.region_manager_cursor) {
                    if self.app.region_panel_type_filter.as_deref() == Some(&entry.definition_name)
                    {
                        self.app.region_panel_type_filter = None;
                    } else {
                        self.app.region_panel_type_filter = Some(entry.definition_name.clone());
                    }
                    self.app.region_manager_cursor = 0;
                }
                true
            }
            KeyCode::Char('s') => {
                self.app.region_panel_sort = self.app.region_panel_sort.toggle();
                true
            }
            KeyCode::Esc => {
                self.app.panel_state.focus_log_table();
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

    /// Handle panel system keys (Ctrl+arrows, Tab/BackTab, z).
    fn handle_panel_keys(&mut self, key: KeyEvent) -> KeyAction {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let handled = match key.code {
            KeyCode::Down if ctrl => {
                self.app.panel_state.focus_panel();
                self.app.detail_open = self.app.panel_state.expanded
                    && self.app.panel_state.active == crate::panel::PanelId::Detail;
                true
            }
            KeyCode::Up if ctrl => {
                self.app.panel_state.focus_log_table();
                true
            }
            KeyCode::Right if ctrl => {
                self.app.panel_state.next_panel();
                true
            }
            KeyCode::Left if ctrl => {
                self.app.panel_state.prev_panel();
                true
            }
            KeyCode::Tab if key.modifiers.is_empty() => {
                if self.app.panel_state.focus == crate::panel::PanelFocus::LogTable {
                    self.app.panel_state.active = crate::panel::PanelId::all()[0];
                    self.app.panel_state.focus_panel();
                    self.app.detail_open = self.app.panel_state.expanded
                        && self.app.panel_state.active == crate::panel::PanelId::Detail;
                    tracing::debug!(active = ?self.app.panel_state.active, "Tab: log table → panel");
                } else {
                    let all = crate::panel::PanelId::all();
                    if self.app.panel_state.active == *all.last().unwrap() {
                        self.app.detail_tree_focus = false;
                        self.app.panel_state.focus_log_table();
                        tracing::debug!("Tab: last panel → log table");
                    } else {
                        self.app.detail_tree_focus = false;
                        self.app.panel_state.next_panel();
                        tracing::debug!(active = ?self.app.panel_state.active, "Tab: → next panel");
                    }
                }
                true
            }
            KeyCode::BackTab => {
                if self.app.panel_state.focus == crate::panel::PanelFocus::LogTable {
                    let all = crate::panel::PanelId::all();
                    let target = *all.last().unwrap();
                    tracing::info!(target_panel = ?target, "BackTab: entering panels from log table (reverse)");
                    self.app.panel_state.active = target;
                    self.app.panel_state.focus_panel();
                    self.app.detail_open = self.app.panel_state.expanded
                        && self.app.panel_state.active == crate::panel::PanelId::Detail;
                    tracing::debug!(active = ?self.app.panel_state.active, "Shift+Tab: log table → panel");
                } else {
                    let all = crate::panel::PanelId::all();
                    if self.app.panel_state.active == all[0] {
                        self.app.detail_tree_focus = false;
                        self.app.panel_state.focus_log_table();
                        tracing::debug!("Shift+Tab: first panel → log table");
                    } else {
                        self.app.detail_tree_focus = false;
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
                if self.app.detail_open {
                    self.app.detail_open = false;
                }
            }
            Action::MoveDown => self.app.select_down(1),
            Action::MoveUp => self.app.select_up(1),
            Action::PageDown => self.app.page_down(),
            Action::PageUp => self.app.page_up(),
            Action::ScrollToTop => self.app.scroll_to_top(),
            Action::ScrollToBottom => self.app.scroll_to_bottom(),
            Action::ToggleDetail => self.app.toggle_detail(),
            Action::Filter => self.app.input_mode = InputMode::Filter,
            Action::Search => self.app.input_mode = InputMode::Search,
            Action::JumpForward => {
                self.app.input_mode = InputMode::JumpForward;
                self.app.time_input.clear();
            }
            Action::JumpBackward => {
                self.app.input_mode = InputMode::JumpBackward;
                self.app.time_input.clear();
            }
            Action::QuickExclude => {
                self.app.input_mode = InputMode::QuickExclude;
                self.app.quick_filter_input.clear();
            }
            Action::QuickInclude => {
                self.app.input_mode = InputMode::QuickInclude;
                self.app.quick_filter_input.clear();
            }
            Action::FieldExclude => self.app.open_field_filter(true),
            Action::FieldInclude => self.app.open_field_filter(false),
            Action::FilterManager => {
                self.app.input_mode = InputMode::FilterManager;
                self.app.filter_manager_cursor = 0;
            }
            Action::LevelFilter => {
                self.app.input_mode = InputMode::LevelFilter;
                self.app.level_filter_cursor = self
                    .app
                    .level_filter
                    .map(|l| (l.as_number() - 1) as usize)
                    .unwrap_or(0);
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
            }
            Action::GotoLine => {
                self.app.input_mode = InputMode::GotoLine;
                self.app.goto_input.clear();
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
            }
            Action::Save => self.app.input_mode = InputMode::SaveDialog,
            Action::ColumnSelector => {
                self.app.input_mode = InputMode::ColumnSelector;
                self.app.column_config.cursor = 0;
            }
            Action::Help => {
                self.app.input_mode = InputMode::Help;
                self.app.help_scroll = 0;
            }
            Action::Command => {
                self.app.command_input.clear();
                self.app.input_mode = InputMode::Command;
            }
            Action::AddHighlight => {
                self.app.input_mode = InputMode::Highlight;
                self.app.highlight_input.clear();
            }
            Action::HighlightManager => {
                self.app.input_mode = InputMode::HighlightManager;
                self.app.highlight_manager_cursor = 0;
            }
            Action::ToggleBookmark => self.app.toggle_bookmark(),
            Action::NextBookmark => self.app.jump_next_bookmark(),
            Action::PrevBookmark => self.app.jump_prev_bookmark(),
            Action::BookmarkManager => {
                self.app.input_mode = InputMode::BookmarkManager;
                self.app.bookmark_manager_cursor = 0;
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
        }
        Some(false)
    }

    /// Handle a key event in Normal mode.
    /// Returns `WindowAction::Close` if the app should quit.
    pub fn handle_normal_key(&mut self, key: KeyEvent) -> WindowAction {
        // 1. Detail tree focus
        if self.app.detail_open
            && self.app.detail_tree_focus
            && self.handle_detail_tree_key(key) == KeyAction::Handled
        {
            return WindowAction::Handled;
        }

        // 2. Region panel focus
        if self.app.panel_state.has_focus()
            && self.app.panel_state.active == crate::panel::PanelId::Region
            && self.handle_region_panel_key(key) == KeyAction::Handled
        {
            return WindowAction::Handled;
        }

        // 3. Panel system keys
        if self.handle_panel_keys(key) == KeyAction::Handled {
            return WindowAction::Handled;
        }

        // 4. Log table / global keys via keymap
        match self.handle_log_table_key(key) {
            Some(true) => WindowAction::Close, // quit
            Some(false) => WindowAction::Handled,
            None => WindowAction::Unhandled,
        }
    }

    /// Handle a key event for overlay input modes (Filter, Search, etc.).
    /// Returns `true` if should quit.
    pub fn handle_overlay_key(&mut self, key: KeyEvent) -> bool {
        match self.app.input_mode {
            InputMode::Normal => unreachable!(),
            InputMode::Filter => match key.code {
                KeyCode::Enter => {
                    self.app.apply_filter();
                    if self.app.filter_error.is_none() {
                        self.app.input_mode = InputMode::Normal;
                    }
                }
                KeyCode::Esc => self.app.input_mode = InputMode::Normal,
                _ => {
                    if self.app.filter_input.handle_key(key) {
                        self.app.filter_error = None;
                    }
                }
            },
            InputMode::Search => match key.code {
                KeyCode::Enter => {
                    self.app.execute_search();
                    self.app.input_mode = InputMode::Normal;
                }
                KeyCode::Esc => self.app.input_mode = InputMode::Normal,
                _ => {
                    self.app.search_input.handle_key(key);
                }
            },
            InputMode::JumpForward | InputMode::JumpBackward => match key.code {
                KeyCode::Enter => {
                    let forward = self.app.input_mode == InputMode::JumpForward;
                    if self.app.jump_relative(forward) {
                        self.app.input_mode = InputMode::Normal;
                    }
                }
                KeyCode::Esc => self.app.input_mode = InputMode::Normal,
                _ => {
                    self.app.time_input.handle_key(key);
                }
            },
            InputMode::GotoLine => {
                use crate::ui::windows::goto_line_window::GotoLineWindow;
                let mut window = GotoLineWindow::new();
                window.input = self.app.goto_input.value().to_string();
                let result = crate::ui::dispatch_key(&mut window, key);
                self.app.goto_input.set(&window.input);
                if result == crate::ui::ComponentResult::Close {
                    if window.confirmed {
                        self.app.goto_line();
                    }
                    self.app.input_mode = InputMode::Normal;
                }
            }
            InputMode::QuickExclude => match key.code {
                KeyCode::Enter => {
                    self.app.apply_quick_exclude();
                    self.app.input_mode = InputMode::Normal;
                }
                KeyCode::Esc => self.app.input_mode = InputMode::Normal,
                _ => {
                    self.app.quick_filter_input.handle_key(key);
                }
            },
            InputMode::QuickInclude => match key.code {
                KeyCode::Enter => {
                    self.app.apply_quick_include();
                    self.app.input_mode = InputMode::Normal;
                }
                KeyCode::Esc => self.app.input_mode = InputMode::Normal,
                _ => {
                    self.app.quick_filter_input.handle_key(key);
                }
            },
            InputMode::FieldFilter => {
                use crate::ui::windows::field_filter_window::FieldFilterWindow;
                if let Some(mut window) = FieldFilterWindow::from_app(&self.app) {
                    let result = crate::ui::dispatch_key(&mut window, key);
                    match result {
                        crate::ui::ComponentResult::Close => {
                            if window.confirmed {
                                window.sync_to_app(&mut self.app);
                                self.app.apply_field_filter();
                            } else {
                                self.app.field_filter = None;
                            }
                            self.app.input_mode = InputMode::Normal;
                        }
                        _ => {
                            window.sync_to_app(&mut self.app);
                        }
                    }
                } else {
                    self.app.input_mode = InputMode::Normal;
                }
            }
            InputMode::FilterManager => {
                use crate::ui::windows::filter_manager_window::FilterManagerWindow;
                let mut window = FilterManagerWindow::from_app(&self.app);
                let result = crate::ui::dispatch_key(&mut window, key);
                window.apply_to_app(&mut self.app);
                if result == crate::ui::ComponentResult::Close {
                    match window.action {
                        Some("save_preset") => {
                            self.app.preset_name_input.clear();
                            self.app.input_mode = InputMode::SavePreset;
                        }
                        Some("load_preset") => {
                            self.app.preset_list = crate::config::filter_preset::list_presets();
                            self.app.preset_list_cursor = 0;
                            self.app.input_mode = InputMode::LoadPreset;
                        }
                        _ => {
                            self.app.input_mode = InputMode::Normal;
                        }
                    }
                }
            }
            InputMode::ColumnSelector => {
                use crate::ui::windows::column_selector_window::ColumnSelectorWindow;
                let mut window = ColumnSelectorWindow::from_app(&self.app);
                let result = crate::ui::dispatch_key(&mut window, key);
                window.sync_to_app(&mut self.app);
                if result == crate::ui::ComponentResult::Close {
                    self.app.input_mode = InputMode::Normal;
                }
            }
            InputMode::CopyFormat => {
                use crate::ui::windows::copy_format_window::CopyFormatWindow;
                let mut window = CopyFormatWindow::from_app(&self.app);
                let result = crate::ui::dispatch_key(&mut window, key);
                self.app.copy_format_cursor = window.cursor;
                if result == crate::ui::ComponentResult::Close {
                    if window.confirmed {
                        CopyFormatWindow::select_format(&mut self.app, window.selected_format());
                    }
                    self.app.input_mode = InputMode::Normal;
                    self.app.copy_format_cursor = 0;
                }
            }
            InputMode::Help => {
                use crate::ui::windows::help_window::HelpWindow;
                let mut window = HelpWindow::new(&self.app.theme);
                window.scroll = self.app.help_scroll;
                let result = crate::ui::dispatch_key(&mut window, key);
                self.app.help_scroll = window.scroll;
                if result == crate::ui::ComponentResult::Close {
                    self.app.input_mode = InputMode::Normal;
                }
            }
            InputMode::Command => match key.code {
                KeyCode::Enter => {
                    self.app.execute_command();
                    self.app.input_mode = InputMode::Normal;
                    if self.app.should_quit {
                        return true;
                    }
                }
                KeyCode::Esc => {
                    self.app.input_mode = InputMode::Normal;
                }
                _ => {
                    self.app.command_input.handle_key(key);
                }
            },
            InputMode::Highlight => match key.code {
                KeyCode::Enter => {
                    let pattern = self.app.highlight_input.value().to_string();
                    if let Err(e) = self.app.add_highlight_rule(&pattern) {
                        self.app.set_status(e);
                    }
                    self.app.input_mode = InputMode::Normal;
                }
                KeyCode::Esc => {
                    self.app.input_mode = InputMode::Normal;
                }
                _ => {
                    self.app.highlight_input.handle_key(key);
                }
            },
            InputMode::HighlightManager => {
                use crate::ui::windows::highlight_manager_window::HighlightManagerWindow;
                let mut window = HighlightManagerWindow::from_app(&self.app);
                let result = crate::ui::dispatch_key(&mut window, key);
                window.apply_to_app(&mut self.app);
                if result == crate::ui::ComponentResult::Close {
                    self.app.input_mode = InputMode::Normal;
                }
            }
            InputMode::BookmarkManager => {
                use crate::ui::windows::bookmark_manager_window::BookmarkManagerWindow;
                let mut window = BookmarkManagerWindow::from_app(&self.app);
                let result = crate::ui::dispatch_key(&mut window, key);
                window.apply_to_app(&mut self.app);
                if result == crate::ui::ComponentResult::Close {
                    self.app.input_mode = InputMode::Normal;
                }
            }
            InputMode::LevelFilter => {
                use crate::ui::windows::level_filter_window::LevelFilterWindow;
                let mut window = LevelFilterWindow::from_app(&self.app);
                let result = crate::ui::dispatch_key(&mut window, key);
                if result == crate::ui::ComponentResult::Close {
                    if window.confirmed {
                        if let Some(preset) = window.selected {
                            self.app.apply_level_filter(preset);
                        }
                    }
                    self.app.input_mode = InputMode::Normal;
                }
            }
            InputMode::SavePreset => {
                use crate::ui::windows::save_preset_window::SavePresetWindow;
                let mut window = SavePresetWindow::new();
                window.input = self.app.preset_name_input.clone();
                let result = crate::ui::dispatch_key(&mut window, key);
                self.app.preset_name_input = window.input;
                if result == crate::ui::ComponentResult::Close {
                    if window.confirmed {
                        let name = self.app.preset_name_input.value().to_string();
                        self.app.save_filter_preset(&name);
                    }
                    self.app.input_mode = InputMode::Normal;
                }
            }
            InputMode::LoadPreset => {
                use crate::ui::windows::load_preset_window::LoadPresetWindow;
                let mut window = LoadPresetWindow::new(self.app.preset_list.clone());
                window.cursor = self.app.preset_list_cursor;
                let result = crate::ui::dispatch_key(&mut window, key);
                if let Some(ref name) = window.delete_name {
                    let _ = crate::config::filter_preset::delete_preset(name);
                }
                self.app.preset_list = window.presets;
                self.app.preset_list_cursor = window.cursor;
                if result == crate::ui::ComponentResult::Close {
                    if window.confirmed {
                        if let Some(ref name) = window.selected {
                            self.app.load_filter_preset(name);
                        }
                    }
                    self.app.input_mode = InputMode::Normal;
                }
            }
            InputMode::DensitySelector => {
                use crate::ui::windows::density_selector_window::DensitySelectorWindow;
                let options = self.app.density_source_options();
                let mut window =
                    DensitySelectorWindow::new(options, self.app.density_selector_cursor);
                let result = crate::ui::dispatch_key(&mut window, key);
                self.app.density_selector_cursor = window.cursor;
                if result == crate::ui::ComponentResult::Close {
                    if window.confirmed {
                        if let Some(source) = window.selected {
                            self.app.density_source = source;
                            self.app.density_cache = None;
                            self.app.set_status(format!(
                                "Density: {}",
                                self.app.density_source_label()
                            ));
                        }
                    }
                    self.app.input_mode = InputMode::Normal;
                }
            }
            InputMode::SaveDialog => {
                use crate::ui::windows::save_dialog_window::SaveDialogWindow;
                let mut window = SaveDialogWindow::from_app(&self.app);
                let result = crate::ui::dispatch_key(&mut window, key);
                self.app.save_path_input = window.path_input.clone();
                self.app.save_format_cursor = window.format_cursor;
                self.app.save_dialog_focus = window.focus;
                if result == crate::ui::ComponentResult::Close {
                    if window.confirmed {
                        let path = window.expanded_path();
                        let format = window.selected_format();
                        let msg = SaveDialogWindow::execute_save(&self.app, &path, format);
                        self.app.set_status(msg);
                    }
                    self.app.input_mode = InputMode::Normal;
                    self.app.save_path_input =
                        crate::text_input::TextInput::with_text("./scouty-export.log");
                    self.app.save_format_cursor = 0;
                    self.app.save_dialog_focus =
                        crate::ui::windows::save_dialog_window::Focus::Path;
                }
            }
            InputMode::RegionManager => {
                use crate::ui::windows::region_manager_window::RegionManagerWindow;
                let mut window = RegionManagerWindow::from_app(&self.app);
                let result = crate::ui::dispatch_key(&mut window, key);
                self.app.region_manager_cursor = window.cursor;
                if result == crate::ui::ComponentResult::Close {
                    if let Some(action) = window.action {
                        match action {
                            crate::ui::windows::region_manager_window::RegionAction::Jump(idx) => {
                                self.app.jump_to_record_index(idx);
                            }
                            crate::ui::windows::region_manager_window::RegionAction::Filter(
                                _start,
                                _end,
                            ) => {
                                let def_name =
                                    &self.app.regions.regions()[window.cursor].definition_name;
                                let expr = format!("_region_type == \"{}\"", def_name);
                                self.app.add_filter_expr(&expr);
                            }
                        }
                    }
                    self.app.input_mode = InputMode::Normal;
                }
            }
        }
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
            "MainWindow: key event"
        );

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
}
