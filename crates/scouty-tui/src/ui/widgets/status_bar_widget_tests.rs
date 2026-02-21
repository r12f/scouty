#[cfg(test)]
mod tests {
    use crate::ui::widgets::status_bar_widget::StatusBarWidget;
    use crate::ui::{dispatch_key, ComponentResult, UiComponent};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn test_enable_jk_navigation() {
        let widget = StatusBarWidget;
        assert!(!widget.enable_jk_navigation());
    }

    #[test]
    fn test_esc_closes() {
        let mut widget = StatusBarWidget;
        assert_eq!(
            dispatch_key(&mut widget, key(KeyCode::Esc)),
            ComponentResult::Close
        );
    }

    #[test]
    fn test_chars_ignored() {
        let mut widget = StatusBarWidget;
        assert_eq!(
            dispatch_key(&mut widget, key(KeyCode::Char('x'))),
            ComponentResult::Ignored
        );
    }
}
