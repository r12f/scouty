#[cfg(test)]
mod tests {
    use crate::ui::windows::field_filter_window::FieldFilterWindow;
    use crate::ui::{dispatch_key, ComponentResult, UiComponent};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    fn sample_window() -> FieldFilterWindow {
        FieldFilterWindow {
            fields: vec![
                ("host".into(), "foo".into(), false),
                ("level".into(), "error".into(), false),
                ("pid".into(), "123".into(), false),
            ],
            cursor: 0,
            exclude: true,
            logic_or: false,
            confirmed: false,
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
    fn test_toggle() {
        let mut w = sample_window();
        assert!(!w.fields[0].2);
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char(' '))),
            ComponentResult::Consumed
        );
        assert!(w.fields[0].2);
    }

    #[test]
    fn test_tab_toggles_exclude() {
        let mut w = sample_window();
        assert!(w.exclude);
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Tab)),
            ComponentResult::Consumed
        );
        assert!(!w.exclude);
    }

    #[test]
    fn test_o_toggles_logic() {
        let mut w = sample_window();
        assert!(!w.logic_or);
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char('o'))),
            ComponentResult::Consumed
        );
        assert!(w.logic_or);
    }

    #[test]
    fn test_enter_confirms() {
        let mut w = sample_window();
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Enter)),
            ComponentResult::Close
        );
        assert!(w.confirmed);
    }

    #[test]
    fn test_esc_cancels() {
        let mut w = sample_window();
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Esc)),
            ComponentResult::Close
        );
        assert!(!w.confirmed);
    }

    #[test]
    fn test_jk_navigation_enabled() {
        let w = sample_window();
        assert!(w.enable_jk_navigation());
    }
}
