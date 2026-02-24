#[cfg(test)]
mod tests {
    use super::super::SavePresetWindow;
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
    fn test_esc_closes() {
        let mut w = SavePresetWindow::new();
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Esc)),
            ComponentResult::Close
        );
        assert!(!w.confirmed);
    }

    #[test]
    fn test_enter_with_empty_stays() {
        let mut w = SavePresetWindow::new();
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Enter)),
            ComponentResult::Consumed
        );
        assert!(!w.confirmed);
    }

    #[test]
    fn test_type_and_confirm() {
        let mut w = SavePresetWindow::new();
        dispatch_key(&mut w, key(KeyCode::Char('t')));
        dispatch_key(&mut w, key(KeyCode::Char('e')));
        dispatch_key(&mut w, key(KeyCode::Char('s')));
        dispatch_key(&mut w, key(KeyCode::Char('t')));
        assert_eq!(w.input.value(), "test");
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Enter)),
            ComponentResult::Close
        );
        assert!(w.confirmed);
    }

    #[test]
    fn test_backspace() {
        let mut w = SavePresetWindow::new();
        dispatch_key(&mut w, key(KeyCode::Char('a')));
        dispatch_key(&mut w, key(KeyCode::Char('b')));
        dispatch_key(&mut w, key(KeyCode::Backspace));
        assert_eq!(w.input.value(), "a");
    }
}
