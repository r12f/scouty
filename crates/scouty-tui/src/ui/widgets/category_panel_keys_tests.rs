#[cfg(test)]
mod tests {
    use crate::app::App;
    use crate::ui::framework::KeyAction;
    use crate::ui::widgets::category_panel_keys::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn test_app() -> App {
        App::load_stdin(Vec::new()).unwrap()
    }

    #[test]
    fn test_cursor_down_no_categories() {
        let mut app = test_app();
        // No categories → handled but cursor stays at 0
        let result = handle_key(&mut app, key(KeyCode::Char('j')));
        assert_eq!(result, KeyAction::Handled);
        assert_eq!(app.category_cursor, 0);
    }

    #[test]
    fn test_cursor_up_at_zero() {
        let mut app = test_app();
        app.category_cursor = 0;
        let result = handle_key(&mut app, key(KeyCode::Char('k')));
        assert_eq!(result, KeyAction::Handled);
        assert_eq!(app.category_cursor, 0);
    }

    #[test]
    fn test_esc_returns_to_log_table() {
        let mut app = test_app();
        app.panel_state.focus_panel();
        let result = handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(result, KeyAction::Handled);
        assert_eq!(app.panel_state.focus, crate::panel::PanelFocus::LogTable);
    }

    #[test]
    fn test_unhandled_key() {
        let mut app = test_app();
        let result = handle_key(&mut app, key(KeyCode::Char('x')));
        assert_eq!(result, KeyAction::Unhandled);
    }

    #[test]
    fn test_home_key() {
        let mut app = test_app();
        app.category_cursor = 5;
        handle_key(&mut app, key(KeyCode::Home));
        assert_eq!(app.category_cursor, 0);
    }

    #[test]
    fn test_shortcut_hints() {
        let hints = shortcut_hints();
        assert!(!hints.is_empty());
        assert!(hints.iter().any(|(k, _)| *k == "Enter"));
    }

    #[test]
    fn test_out_of_range_cursor_clamped() {
        let mut app = test_app();
        // Set cursor beyond available categories (0 categories)
        app.category_cursor = 5;
        // handle_key clamps it to 0
        let result = handle_key(&mut app, key(KeyCode::Char('j')));
        assert_eq!(result, KeyAction::Handled);
        assert_eq!(app.category_cursor, 0);
    }
}
