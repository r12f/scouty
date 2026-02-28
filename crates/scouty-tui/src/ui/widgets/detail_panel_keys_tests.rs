#[cfg(test)]
mod tests {
    use crate::app::App;
    use crate::ui::framework::KeyAction;
    use crate::ui::widgets::detail_panel_keys::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn test_app() -> App {
        let mut app = App::load_stdin(Vec::new()).unwrap();
        app.detail_open = true;
        app.detail_tree_focus = true;
        app
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_j_handled() {
        let mut app = test_app();
        assert_eq!(
            handle_key(&mut app, key(KeyCode::Char('j'))),
            KeyAction::Handled
        );
    }

    #[test]
    fn test_esc_exits_focus() {
        let mut app = test_app();
        assert_eq!(handle_key(&mut app, key(KeyCode::Esc)), KeyAction::Handled);
        assert!(!app.detail_tree_focus);
    }

    #[test]
    fn test_unhandled_key() {
        let mut app = test_app();
        assert_eq!(
            handle_key(&mut app, key(KeyCode::Char('x'))),
            KeyAction::Unhandled
        );
    }

    #[test]
    fn test_shortcut_hints() {
        let hints = shortcut_hints();
        assert_eq!(hints[0], ("←/→", "Fold"));
        assert_eq!(hints[1], ("H/L", "All"));
    }
}
