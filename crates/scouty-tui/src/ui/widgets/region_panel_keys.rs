//! Region panel key handling and shortcut hints.
//!
//! Extracted from MainWindow — handles navigation when the region
//! panel has focus.

#[cfg(test)]
#[path = "region_panel_keys_tests.rs"]
mod region_panel_keys_tests;

use crate::app::App;
use crate::ui::framework::KeyAction;
use crate::ui::widgets::region_panel_widget::RegionPanelWidget;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle a key event when the region panel has focus.
pub fn handle_key(app: &mut App, key: KeyEvent) -> KeyAction {
    let entries = RegionPanelWidget::build_entries(app);
    let max_cursor = entries.len().saturating_sub(1);
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    let handled = match key.code {
        KeyCode::Char('j') | KeyCode::Down if !ctrl => {
            if app.region_manager_cursor < max_cursor {
                app.region_manager_cursor += 1;
            }
            true
        }
        KeyCode::Char('k') | KeyCode::Up if !ctrl => {
            if app.region_manager_cursor > 0 {
                app.region_manager_cursor -= 1;
            }
            true
        }
        KeyCode::Enter => {
            if let Some(entry) = entries.get(app.region_manager_cursor) {
                app.jump_to_record_index(entry.start_index);
                app.panel_state.focus_log_table();
            }
            true
        }
        KeyCode::Char('f') => {
            if let Some(entry) = entries.get(app.region_manager_cursor) {
                let expr = format!("_region_type == \"{}\"", entry.definition_name);
                app.add_filter_expr(&expr);
            }
            true
        }
        KeyCode::Char('t') => {
            if let Some(entry) = entries.get(app.region_manager_cursor) {
                if app.region_panel_type_filter.as_deref() == Some(&entry.definition_name) {
                    app.region_panel_type_filter = None;
                } else {
                    app.region_panel_type_filter = Some(entry.definition_name.clone());
                }
                app.region_manager_cursor = 0;
            }
            true
        }
        KeyCode::Char('s') => {
            app.region_panel_sort = app.region_panel_sort.toggle();
            true
        }
        KeyCode::Esc => {
            app.panel_state.focus_log_table();
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

/// Shortcut hints for the region panel.
pub fn shortcut_hints() -> Vec<(&'static str, &'static str)> {
    vec![("j/k", "↑↓")]
}
