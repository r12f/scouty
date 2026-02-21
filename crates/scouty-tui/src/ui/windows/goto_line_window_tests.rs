#[cfg(test)]
mod tests {
    use crate::ui::windows::goto_line_window::GotoLineWindow;
    use crate::ui::{dispatch_key, ComponentResult};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn test_digit_input() {
        let mut w = GotoLineWindow::new();
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char('1'))),
            ComponentResult::Consumed
        );
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char('5'))),
            ComponentResult::Consumed
        );
        assert_eq!(w.input, "15");
    }

    #[test]
    fn test_non_digit_ignored() {
        let mut w = GotoLineWindow::new();
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char('a'))),
            ComponentResult::Ignored
        );
        assert!(w.input.is_empty());
    }

    #[test]
    fn test_enter_closes_and_confirms() {
        let mut w = GotoLineWindow::new();
        w.input = "42".to_string();
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Enter)),
            ComponentResult::Close
        );
        assert!(w.confirmed);
    }

    #[test]
    fn test_esc_closes() {
        let mut w = GotoLineWindow::new();
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Esc)),
            ComponentResult::Close
        );
        assert!(!w.confirmed);
    }

    #[test]
    fn test_backspace() {
        let mut w = GotoLineWindow::new();
        w.input = "12".to_string();
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Backspace)),
            ComponentResult::Consumed
        );
        assert_eq!(w.input, "1");
    }
}
