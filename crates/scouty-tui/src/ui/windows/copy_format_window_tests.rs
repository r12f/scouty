#[cfg(test)]
mod tests {
    use crate::app::CopyFormat;
    use crate::ui::windows::copy_format_window::CopyFormatWindow;
    use crate::ui::{dispatch_key, ComponentResult, UiComponent};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    fn sample_window() -> CopyFormatWindow {
        CopyFormatWindow {
            cursor: 0,
            confirmed: false,
            theme: crate::config::Theme::default(),
        }
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
    fn test_enter_confirms() {
        let mut w = sample_window();
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Enter)),
            ComponentResult::Close
        );
        assert!(w.confirmed);
        assert_eq!(w.selected_format(), CopyFormat::Raw);
    }

    #[test]
    fn test_jk_navigation() {
        let mut w = sample_window();
        assert_eq!(w.cursor, 0);

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

        // Can't go past end
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

        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char('k'))),
            ComponentResult::Consumed
        );
        assert_eq!(w.cursor, 0);

        // Can't go past start
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Up)),
            ComponentResult::Consumed
        );
        assert_eq!(w.cursor, 0);
    }

    #[test]
    fn test_select_json() {
        let mut w = sample_window();
        dispatch_key(&mut w, key(KeyCode::Down)); // cursor=1 (JSON)
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Enter)),
            ComponentResult::Close
        );
        assert!(w.confirmed);
        assert_eq!(w.selected_format(), CopyFormat::Json);
    }

    #[test]
    fn test_select_yaml() {
        let mut w = sample_window();
        dispatch_key(&mut w, key(KeyCode::Down));
        dispatch_key(&mut w, key(KeyCode::Down)); // cursor=2 (YAML)
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Enter)),
            ComponentResult::Close
        );
        assert!(w.confirmed);
        assert_eq!(w.selected_format(), CopyFormat::Yaml);
    }

    #[test]
    fn test_enable_jk_navigation() {
        let w = sample_window();
        assert!(w.enable_jk_navigation());
    }
}
