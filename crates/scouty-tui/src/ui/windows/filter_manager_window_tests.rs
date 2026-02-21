#[cfg(test)]
mod tests {
    use crate::ui::windows::filter_manager_window::FilterManagerWindow;
    use crate::ui::{dispatch_key, ComponentResult, UiComponent};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    fn sample_window() -> FilterManagerWindow {
        FilterManagerWindow {
            cursor: 0,
            filter_count: 3,
            action: None,
        }
    }

    #[test]
    fn test_navigation() {
        let mut w = sample_window();
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Down)),
            ComponentResult::Consumed
        );
        assert_eq!(w.cursor, 1);
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char('j'))),
            ComponentResult::Consumed
        );
        assert_eq!(w.cursor, 2);
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Down)),
            ComponentResult::Consumed
        );
        assert_eq!(w.cursor, 2);
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Up)),
            ComponentResult::Consumed
        );
        assert_eq!(w.cursor, 1);
    }

    #[test]
    fn test_delete() {
        let mut w = sample_window();
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char('d'))),
            ComponentResult::Consumed
        );
        assert_eq!(w.action, Some("delete"));
    }

    #[test]
    fn test_clear() {
        let mut w = sample_window();
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char('c'))),
            ComponentResult::Consumed
        );
        assert_eq!(w.action, Some("clear"));
    }

    #[test]
    fn test_esc_closes() {
        let mut w = sample_window();
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Esc)),
            ComponentResult::Close
        );
    }

    #[test]
    fn test_enter_closes() {
        let mut w = sample_window();
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Enter)),
            ComponentResult::Close
        );
    }

    #[test]
    fn test_jk_navigation_enabled() {
        let w = sample_window();
        assert!(w.enable_jk_navigation());
    }
}
