#[cfg(test)]
mod tests {
    use crate::app::{FieldEntry, FieldEntryKind};
    use crate::ui::windows::field_filter_window::FieldFilterWindow;
    use crate::ui::{dispatch_key, ComponentResult, UiComponent};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    fn make_field(name: &str, val: &str) -> FieldEntry {
        FieldEntry {
            name: name.to_string(),
            value: val.to_string(),
            checked: false,
            kind: FieldEntryKind::Field,
        }
    }

    fn sample_window() -> FieldFilterWindow {
        FieldFilterWindow {
            fields: vec![
                make_field("host", "foo"),
                make_field("level", "error"),
                make_field("pid", "123"),
            ],
            cursor: 0,
            exclude: true,
            logic_or: false,
            confirmed: false,
            theme: crate::config::Theme::default(),
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
        assert!(!w.fields[0].checked);
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char(' '))),
            ComponentResult::Consumed
        );
        assert!(w.fields[0].checked);
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
