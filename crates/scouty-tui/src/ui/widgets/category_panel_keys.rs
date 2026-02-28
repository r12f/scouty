//! Category panel key handling and shortcut hints.

#[cfg(test)]
#[path = "category_panel_keys_tests.rs"]
mod category_panel_keys_tests;

use crate::app::App;
use crate::ui::framework::KeyAction;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handle a key event when the category panel has focus.
pub fn handle_key(app: &mut App, key: KeyEvent) -> KeyAction {
    let max_cursor = category_count(app).saturating_sub(1);
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    let handled = match key.code {
        KeyCode::Char('j') | KeyCode::Down if !ctrl => {
            if app.category_cursor < max_cursor {
                app.category_cursor += 1;
            }
            true
        }
        KeyCode::Char('k') | KeyCode::Up if !ctrl => {
            if app.category_cursor > 0 {
                app.category_cursor -= 1;
            }
            true
        }
        KeyCode::Home | KeyCode::Char('g') => {
            app.category_cursor = 0;
            true
        }
        KeyCode::End | KeyCode::Char('G') => {
            app.category_cursor = max_cursor;
            true
        }
        KeyCode::Enter => {
            if let Some(filter_expr) = selected_filter_expr(app) {
                app.add_filter_expr(&filter_expr);
                app.panel_state.focus_log_table();
            }
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

/// Number of categories currently loaded.
fn category_count(app: &App) -> usize {
    app.category_processor
        .as_ref()
        .map(|cp| cp.store.categories.len())
        .unwrap_or(0)
}

/// Build filter expression for the selected category.
fn selected_filter_expr(app: &App) -> Option<String> {
    let cp = app.category_processor.as_ref()?;
    let cat = cp.store.categories.get(app.category_cursor)?;
    Some(format!("{}", cat.definition.filter))
}

/// Shortcut hints for the category panel.
pub fn shortcut_hints() -> Vec<(&'static str, &'static str)> {
    vec![("j/k", "↑↓"), ("Enter", "Filter")]
}
