#[cfg(test)]
mod tests {
    use crate::ui::windows::help_window::HelpWindow;
    use crate::ui::{dispatch_key, ComponentResult};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn test_esc_closes() {
        let mut w = HelpWindow;
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Esc)),
            ComponentResult::Close
        );
    }

    #[test]
    fn test_any_key_closes() {
        let mut w = HelpWindow;
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char('q'))),
            ComponentResult::Close
        );
    }

    #[test]
    fn test_enter_closes() {
        let mut w = HelpWindow;
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Enter)),
            ComponentResult::Close
        );
    }
}
