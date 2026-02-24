#[cfg(test)]
mod tests {
    use super::super::LoadPresetWindow;
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

    fn sample_presets() -> Vec<String> {
        vec!["alpha".into(), "beta".into(), "gamma".into()]
    }

    #[test]
    fn test_esc_closes() {
        let mut w = LoadPresetWindow::new(sample_presets());
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Esc)),
            ComponentResult::Close
        );
        assert!(!w.confirmed);
    }

    #[test]
    fn test_navigate_and_select() {
        let mut w = LoadPresetWindow::new(sample_presets());
        dispatch_key(&mut w, key(KeyCode::Char('j')));
        assert_eq!(w.cursor, 1);
        dispatch_key(&mut w, key(KeyCode::Down));
        assert_eq!(w.cursor, 2);
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Enter)),
            ComponentResult::Close
        );
        assert!(w.confirmed);
        assert_eq!(w.selected, Some("gamma".to_string()));
    }

    #[test]
    fn test_delete_preset() {
        let mut w = LoadPresetWindow::new(sample_presets());
        dispatch_key(&mut w, key(KeyCode::Char('j'))); // cursor at "beta"
        dispatch_key(&mut w, key(KeyCode::Char('d')));
        assert_eq!(w.delete_name, Some("beta".to_string()));
        assert_eq!(w.presets, vec!["alpha", "gamma"]);
        assert_eq!(w.cursor, 1); // stays at same index
    }

    #[test]
    fn test_delete_last_adjusts_cursor() {
        let mut w = LoadPresetWindow::new(sample_presets());
        // Go to last
        dispatch_key(&mut w, key(KeyCode::Down));
        dispatch_key(&mut w, key(KeyCode::Down));
        assert_eq!(w.cursor, 2);
        dispatch_key(&mut w, key(KeyCode::Char('d'))); // delete "gamma"
        assert_eq!(w.cursor, 1); // adjusted to last item
    }

    #[test]
    fn test_empty_presets_enter_does_nothing() {
        let mut w = LoadPresetWindow::new(vec![]);
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Enter)),
            ComponentResult::Consumed
        );
        assert!(!w.confirmed);
    }
}
