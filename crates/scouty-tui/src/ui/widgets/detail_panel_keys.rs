//! Detail panel key handling and shortcut hints.
//!
//! Extracted from MainWindow — handles tree navigation when the detail
//! panel has focus.

#[cfg(test)]
#[path = "detail_panel_keys_tests.rs"]
mod detail_panel_keys_tests;

use crate::app::App;
use crate::ui::framework::KeyAction;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle a key event when the detail panel tree has focus.
pub fn handle_key(app: &mut App, key: KeyEvent) -> KeyAction {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let handled = match key.code {
        KeyCode::Char('j') | KeyCode::Down if !ctrl => {
            app.detail_tree_move_down();
            true
        }
        KeyCode::Char('k') | KeyCode::Up if !ctrl => {
            app.detail_tree_move_up();
            true
        }
        KeyCode::Right | KeyCode::Enter => {
            app.detail_tree_toggle();
            true
        }
        KeyCode::Left => {
            app.detail_tree_collapse_or_parent();
            true
        }
        KeyCode::Char('h') => {
            app.detail_scroll_left();
            true
        }
        KeyCode::Char('l') => {
            app.detail_scroll_right();
            true
        }
        KeyCode::Char('H') => {
            app.detail_tree_collapse_all();
            true
        }
        KeyCode::Char('L') => {
            app.detail_tree_expand_all();
            true
        }
        KeyCode::Char('f') => {
            app.detail_tree_quick_filter();
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

/// Shortcut hints for the detail panel (tree focused).
pub fn shortcut_hints() -> Vec<(&'static str, &'static str)> {
    vec![("←/→", "Fold"), ("H/L", "All")]
}
