#[cfg(test)]
mod tests {
    use crate::app::App;
    use crate::ui::framework::KeyAction;
    use crate::ui::widgets::region_panel_keys::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn test_app() -> App {
        App::load_stdin(Vec::new()).unwrap()
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
    fn test_s_toggles_sort() {
        let mut app = test_app();
        let original = app.region_panel_sort;
        handle_key(&mut app, key(KeyCode::Char('s')));
        assert_ne!(app.region_panel_sort, original);
    }

    #[test]
    fn test_esc_focuses_log_table() {
        let mut app = test_app();
        app.panel_state.focus_panel();
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.panel_state.focus, crate::panel::PanelFocus::LogTable);
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
        assert_eq!(hints[0], ("j/k", "↑↓"));
    }
}
