#[cfg(test)]
mod tests {
    use super::super::DensitySelectorWindow;
    use crate::app::DensitySource;
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

    fn sample_options() -> Vec<DensitySource> {
        vec![
            DensitySource::All,
            DensitySource::Level("ERROR".to_string()),
            DensitySource::Level("WARN".to_string()),
            DensitySource::Highlight("timeout".to_string()),
        ]
    }

    #[test]
    fn test_esc_closes_without_selection() {
        let mut w = DensitySelectorWindow::new(sample_options(), 0);
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Esc)),
            ComponentResult::Close
        );
        assert!(!w.confirmed);
        assert!(w.selected.is_none());
    }

    #[test]
    fn test_navigate_and_select() {
        let mut w = DensitySelectorWindow::new(sample_options(), 0);
        dispatch_key(&mut w, key(KeyCode::Char('j')));
        assert_eq!(w.cursor, 1);
        dispatch_key(&mut w, key(KeyCode::Down));
        assert_eq!(w.cursor, 2);
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Enter)),
            ComponentResult::Close
        );
        assert!(w.confirmed);
        assert_eq!(w.selected, Some(DensitySource::Level("WARN".to_string())));
    }

    #[test]
    fn test_select_first() {
        let mut w = DensitySelectorWindow::new(sample_options(), 0);
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Enter)),
            ComponentResult::Close
        );
        assert!(w.confirmed);
        assert_eq!(w.selected, Some(DensitySource::All));
    }

    #[test]
    fn test_select_highlight() {
        let mut w = DensitySelectorWindow::new(sample_options(), 0);
        dispatch_key(&mut w, key(KeyCode::Down));
        dispatch_key(&mut w, key(KeyCode::Down));
        dispatch_key(&mut w, key(KeyCode::Down));
        assert_eq!(w.cursor, 3);
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Enter)),
            ComponentResult::Close
        );
        assert_eq!(
            w.selected,
            Some(DensitySource::Highlight("timeout".to_string()))
        );
    }

    #[test]
    fn test_cursor_bounds() {
        let mut w = DensitySelectorWindow::new(sample_options(), 0);
        dispatch_key(&mut w, key(KeyCode::Up));
        assert_eq!(w.cursor, 0);
        for _ in 0..10 {
            dispatch_key(&mut w, key(KeyCode::Down));
        }
        assert_eq!(w.cursor, 3);
    }

    #[test]
    fn test_initial_cursor_clamped() {
        let w = DensitySelectorWindow::new(sample_options(), 100);
        assert_eq!(w.cursor, 3);
    }
}
