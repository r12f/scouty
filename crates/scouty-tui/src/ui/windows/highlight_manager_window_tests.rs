//! Tests for highlight manager window.

#[cfg(test)]
mod tests {
    use crate::ui::windows::highlight_manager_window::HighlightManagerWindow;
    use crate::ui::{dispatch_key, ComponentResult};
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_navigation() {
        let mut w = HighlightManagerWindow {
            cursor: 0,
            rule_count: 3,
            action: None,
        };
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Down)),
            ComponentResult::Consumed
        );
        assert_eq!(w.cursor, 1);
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Up)),
            ComponentResult::Consumed
        );
        assert_eq!(w.cursor, 0);
    }

    #[test]
    fn test_delete_action() {
        let mut w = HighlightManagerWindow {
            cursor: 0,
            rule_count: 2,
            action: None,
        };
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char('d'))),
            ComponentResult::Consumed
        );
        assert_eq!(w.action, Some("delete"));
    }

    #[test]
    fn test_close_on_esc() {
        let mut w = HighlightManagerWindow {
            cursor: 0,
            rule_count: 0,
            action: None,
        };
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Esc)),
            ComponentResult::Close
        );
    }
}
