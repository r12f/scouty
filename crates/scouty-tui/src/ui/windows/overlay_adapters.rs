//! Overlay adapters — wrap existing UiComponent windows into OverlayWindow impls.
//!
//! Each adapter owns minimal state (cursor, scroll) and delegates to the
//! underlying UiComponent for key handling and rendering.

#[cfg(test)]
#[path = "overlay_adapters_tests.rs"]
mod overlay_adapters_tests;

use crate::app::{App, InputMode};
use crate::ui::framework::{OverlayWindow, WindowAction};
use crate::ui::{dispatch_key, ComponentResult, UiComponent};
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::Frame;

// ── ColumnSelectorOverlay ───────────────────────────────────────────

pub struct ColumnSelectorOverlay;

impl ColumnSelectorOverlay {
    pub fn new() -> Self {
        Self
    }
}

impl OverlayWindow for ColumnSelectorOverlay {
    fn name(&self) -> &str {
        "ColumnSelector"
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        use crate::ui::windows::column_selector_window::ColumnSelectorWindow;
        let window = ColumnSelectorWindow::from_app(app);
        <ColumnSelectorWindow as UiComponent>::render(&window, frame, area);
    }

    fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> WindowAction {
        use crate::ui::windows::column_selector_window::ColumnSelectorWindow;
        let mut window = ColumnSelectorWindow::from_app(app);
        let result = dispatch_key(&mut window, key);
        window.sync_to_app(app);
        if result == ComponentResult::Close {
            app.input_mode = InputMode::Normal;
            WindowAction::Close
        } else {
            WindowAction::Handled
        }
    }

    fn shortcut_hints(&self) -> Vec<(&str, &str)> {
        vec![("Space", "Toggle"), ("Esc", "Close")]
    }
}

// ── FilterManagerOverlay ────────────────────────────────────────────

pub struct FilterManagerOverlay;

impl FilterManagerOverlay {
    pub fn new() -> Self {
        Self
    }
}

impl OverlayWindow for FilterManagerOverlay {
    fn name(&self) -> &str {
        "FilterManager"
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        use crate::ui::windows::filter_manager_window::FilterManagerWindow;
        let window = FilterManagerWindow::from_app(app);
        window.render_with_app(frame, app, area);
    }

    fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> WindowAction {
        use crate::ui::windows::filter_manager_window::FilterManagerWindow;
        let mut window = FilterManagerWindow::from_app(app);
        let result = dispatch_key(&mut window, key);
        window.apply_to_app(app);
        if result == ComponentResult::Close {
            match window.action {
                Some("save_preset") => {
                    app.preset_name_input.clear();
                    app.input_mode = InputMode::SavePreset;
                }
                Some("load_preset") => {
                    app.preset_list = crate::config::filter_preset::list_presets();
                    app.preset_list_cursor = 0;
                    app.input_mode = InputMode::LoadPreset;
                }
                _ => {
                    app.input_mode = InputMode::Normal;
                }
            }
            WindowAction::Close
        } else {
            WindowAction::Handled
        }
    }

    fn shortcut_hints(&self) -> Vec<(&str, &str)> {
        vec![("d", "Delete"), ("Esc", "Close")]
    }
}

// ── BookmarkManagerOverlay ──────────────────────────────────────────

pub struct BookmarkManagerOverlay;

impl BookmarkManagerOverlay {
    pub fn new() -> Self {
        Self
    }
}

impl OverlayWindow for BookmarkManagerOverlay {
    fn name(&self) -> &str {
        "BookmarkManager"
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        use crate::ui::windows::bookmark_manager_window::BookmarkManagerWindow;
        let window = BookmarkManagerWindow::from_app(app);
        window.render_with_app(frame, app, area);
    }

    fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> WindowAction {
        use crate::ui::windows::bookmark_manager_window::BookmarkManagerWindow;
        let mut window = BookmarkManagerWindow::from_app(app);
        let result = dispatch_key(&mut window, key);
        window.apply_to_app(app);
        if result == ComponentResult::Close {
            app.input_mode = InputMode::Normal;
            WindowAction::Close
        } else {
            WindowAction::Handled
        }
    }

    fn shortcut_hints(&self) -> Vec<(&str, &str)> {
        vec![("Enter", "Jump"), ("d", "Delete"), ("Esc", "Close")]
    }
}

// ── HighlightManagerOverlay ─────────────────────────────────────────

pub struct HighlightManagerOverlay;

impl HighlightManagerOverlay {
    pub fn new() -> Self {
        Self
    }
}

impl OverlayWindow for HighlightManagerOverlay {
    fn name(&self) -> &str {
        "HighlightManager"
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        use crate::ui::windows::highlight_manager_window::HighlightManagerWindow;
        let window = HighlightManagerWindow::from_app(app);
        window.render_with_app(frame, app, area);
    }

    fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> WindowAction {
        use crate::ui::windows::highlight_manager_window::HighlightManagerWindow;
        let mut window = HighlightManagerWindow::from_app(app);
        let result = dispatch_key(&mut window, key);
        window.apply_to_app(app);
        if result == ComponentResult::Close {
            app.input_mode = InputMode::Normal;
            WindowAction::Close
        } else {
            WindowAction::Handled
        }
    }

    fn shortcut_hints(&self) -> Vec<(&str, &str)> {
        vec![("d", "Delete"), ("Esc", "Close")]
    }
}

// ── SaveDialogOverlay ───────────────────────────────────────────────

pub struct SaveDialogOverlay;

impl SaveDialogOverlay {
    pub fn new() -> Self {
        Self
    }
}

impl OverlayWindow for SaveDialogOverlay {
    fn name(&self) -> &str {
        "SaveDialog"
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        use crate::ui::windows::save_dialog_window::SaveDialogWindow;
        let window = SaveDialogWindow::from_app(app);
        <SaveDialogWindow as UiComponent>::render(&window, frame, area);
    }

    fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> WindowAction {
        use crate::ui::windows::save_dialog_window::SaveDialogWindow;
        let mut window = SaveDialogWindow::from_app(app);
        let result = dispatch_key(&mut window, key);
        app.save_path_input = window.path_input.clone();
        app.save_format_cursor = window.format_cursor;
        app.save_dialog_focus = window.focus;
        if result == ComponentResult::Close {
            if window.confirmed {
                let path = window.expanded_path();
                let format = window.selected_format();
                let msg = SaveDialogWindow::execute_save(app, &path, format);
                app.set_status(msg);
            }
            app.input_mode = InputMode::Normal;
            app.save_path_input = crate::text_input::TextInput::with_text("./scouty-export.log");
            app.save_format_cursor = 0;
            app.save_dialog_focus = crate::ui::windows::save_dialog_window::Focus::Path;
            WindowAction::Close
        } else {
            WindowAction::Handled
        }
    }

    fn shortcut_hints(&self) -> Vec<(&str, &str)> {
        vec![("Tab", "Switch"), ("Enter", "Save"), ("Esc", "Cancel")]
    }
}

// ── RegionManagerOverlay ────────────────────────────────────────────

/// Region manager overlay — available for future use when the region
/// manager transitions from panel to overlay mode.
#[allow(dead_code)]
pub struct RegionManagerOverlay;

#[allow(dead_code)]
impl RegionManagerOverlay {
    pub fn new() -> Self {
        Self
    }
}

impl OverlayWindow for RegionManagerOverlay {
    fn name(&self) -> &str {
        "RegionManager"
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        use crate::ui::windows::region_manager_window::RegionManagerWindow;
        let window = RegionManagerWindow::from_app(app);
        window.render_with_app(frame, area, app);
    }

    fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> WindowAction {
        use crate::ui::windows::region_manager_window::RegionManagerWindow;
        let mut window = RegionManagerWindow::from_app(app);
        let result = dispatch_key(&mut window, key);
        app.region_manager_cursor = window.cursor;
        if result == ComponentResult::Close {
            if let Some(action) = window.action {
                match action {
                    crate::ui::windows::region_manager_window::RegionAction::Jump(idx) => {
                        app.jump_to_record_index(idx);
                    }
                    crate::ui::windows::region_manager_window::RegionAction::Filter(
                        _start,
                        _end,
                    ) => {
                        let def_name = &app.regions.regions()[window.cursor].definition_name;
                        let expr = format!("_region_type == \"{}\"", def_name);
                        app.add_filter_expr(&expr);
                    }
                }
            }
            app.input_mode = InputMode::Normal;
            WindowAction::Close
        } else {
            WindowAction::Handled
        }
    }

    fn shortcut_hints(&self) -> Vec<(&str, &str)> {
        vec![("Enter", "Jump"), ("f", "Filter"), ("Esc", "Close")]
    }
}

// ── LevelFilterOverlay ──────────────────────────────────────────────

pub struct LevelFilterOverlay;

impl LevelFilterOverlay {
    pub fn new() -> Self {
        Self
    }
}

impl OverlayWindow for LevelFilterOverlay {
    fn name(&self) -> &str {
        "LevelFilter"
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        use crate::ui::windows::level_filter_window::LevelFilterWindow;
        let window = LevelFilterWindow::from_app(app);
        <LevelFilterWindow as UiComponent>::render(&window, frame, area);
    }

    fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> WindowAction {
        use crate::ui::windows::level_filter_window::LevelFilterWindow;
        let mut window = LevelFilterWindow::from_app(app);
        let result = dispatch_key(&mut window, key);
        if result == ComponentResult::Close {
            if window.confirmed {
                if let Some(preset) = window.selected {
                    app.apply_level_filter(preset);
                }
            }
            app.input_mode = InputMode::Normal;
            WindowAction::Close
        } else {
            WindowAction::Handled
        }
    }

    fn shortcut_hints(&self) -> Vec<(&str, &str)> {
        vec![("1-8", "Level"), ("Esc", "Close")]
    }
}

// ── DensitySelectorOverlay ──────────────────────────────────────────

pub struct DensitySelectorOverlay;

impl DensitySelectorOverlay {
    pub fn new() -> Self {
        Self
    }
}

impl OverlayWindow for DensitySelectorOverlay {
    fn name(&self) -> &str {
        "DensitySelector"
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        use crate::ui::windows::density_selector_window::DensitySelectorWindow;
        let options = app.density_source_options();
        let window = DensitySelectorWindow::new(options, app.density_selector_cursor);
        <DensitySelectorWindow as UiComponent>::render(&window, frame, area);
    }

    fn handle_key(&mut self, app: &mut App, key: KeyEvent) -> WindowAction {
        use crate::ui::windows::density_selector_window::DensitySelectorWindow;
        let options = app.density_source_options();
        let mut window = DensitySelectorWindow::new(options, app.density_selector_cursor);
        let result = dispatch_key(&mut window, key);
        app.density_selector_cursor = window.cursor;
        if result == ComponentResult::Close {
            if window.confirmed {
                if let Some(source) = window.selected {
                    app.density_source = source;
                    app.density_cache = None;
                    app.set_status(format!("Density: {}", app.density_source_label()));
                }
            }
            app.input_mode = InputMode::Normal;
            WindowAction::Close
        } else {
            WindowAction::Handled
        }
    }

    fn shortcut_hints(&self) -> Vec<(&str, &str)> {
        vec![("j/k", "Select"), ("Enter", "Apply"), ("Esc", "Close")]
    }
}
