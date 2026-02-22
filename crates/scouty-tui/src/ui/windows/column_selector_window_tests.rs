#[cfg(test)]
mod tests {
    use crate::app::Column;
    use crate::ui::windows::column_selector_window::ColumnSelectorWindow;
    use crate::ui::{dispatch_key, ComponentResult, UiComponent};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    fn sample_window() -> ColumnSelectorWindow {
        ColumnSelectorWindow {
            cursor: 0,
            columns: vec![
                (Column::Time, true),
                (Column::Level, true),
                (Column::Log, true),
            ],
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
            dispatch_key(&mut w, key(KeyCode::Up)),
            ComponentResult::Consumed
        );
        assert_eq!(w.cursor, 0);
    }

    #[test]
    fn test_toggle_space() {
        let mut w = sample_window();
        assert!(w.columns[0].1);
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char(' '))),
            ComponentResult::Consumed
        );
        assert!(!w.columns[0].1);
    }

    #[test]
    fn test_toggle_enter() {
        let mut w = sample_window();
        assert!(w.columns[0].1);
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Enter)),
            ComponentResult::Consumed
        );
        assert!(!w.columns[0].1);
    }

    #[test]
    fn test_log_column_not_togglable() {
        let mut w = sample_window();
        w.cursor = 2;
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char(' '))),
            ComponentResult::Consumed
        );
        assert!(w.columns[2].1);
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
    fn test_jk_navigation() {
        let mut w = sample_window();
        assert!(w.enable_jk_navigation());
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char('j'))),
            ComponentResult::Consumed
        );
        assert_eq!(w.cursor, 1);
    }
}
