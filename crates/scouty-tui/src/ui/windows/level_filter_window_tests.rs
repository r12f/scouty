#[cfg(test)]
mod tests {
    use super::super::LevelFilterWindow;
    use crate::app::LevelFilterPreset;
    use crate::config::Theme;
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

    #[test]
    fn test_esc_closes() {
        let theme = Theme::default();
        let mut w = LevelFilterWindow::new(None, &theme);
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Esc)),
            ComponentResult::Close
        );
        assert!(!w.confirmed);
    }

    #[test]
    fn test_number_keys_instant_select() {
        let theme = Theme::default();
        for (n, expected) in [
            ('1', LevelFilterPreset::All),
            ('2', LevelFilterPreset::DebugPlus),
            ('3', LevelFilterPreset::InfoPlus),
            ('4', LevelFilterPreset::WarnPlus),
            ('5', LevelFilterPreset::ErrorPlus),
        ] {
            let mut w = LevelFilterWindow::new(None, &theme);
            let result = dispatch_key(&mut w, key(KeyCode::Char(n)));
            assert_eq!(result, ComponentResult::Close);
            assert!(w.confirmed);
            assert_eq!(w.selected, Some(expected));
        }
    }

    #[test]
    fn test_navigation_and_enter() {
        let theme = Theme::default();
        let mut w = LevelFilterWindow::new(None, &theme);
        assert_eq!(w.cursor, 0);

        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char('j'))),
            ComponentResult::Consumed
        );
        assert_eq!(w.cursor, 1);

        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Down)),
            ComponentResult::Consumed
        );
        assert_eq!(w.cursor, 2);

        let result = dispatch_key(&mut w, key(KeyCode::Enter));
        assert_eq!(result, ComponentResult::Close);
        assert!(w.confirmed);
        assert_eq!(w.selected, Some(LevelFilterPreset::InfoPlus));
    }

    #[test]
    fn test_navigation_bounds() {
        let theme = Theme::default();
        let mut w = LevelFilterWindow::new(None, &theme);

        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Up)),
            ComponentResult::Consumed
        );
        assert_eq!(w.cursor, 0);

        for _ in 0..10 {
            dispatch_key(&mut w, key(KeyCode::Char('j')));
        }
        assert_eq!(w.cursor, 4);

        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char('j'))),
            ComponentResult::Consumed
        );
        assert_eq!(w.cursor, 4);
    }

    #[test]
    fn test_current_level_cursor_position() {
        let theme = Theme::default();
        let w = LevelFilterWindow::new(Some(LevelFilterPreset::WarnPlus), &theme);
        assert_eq!(w.cursor, 3);
    }

    #[test]
    fn test_level_filter_preset_matches() {
        use scouty::record::LogLevel;

        assert!(LevelFilterPreset::All.matches_level(Some(&LogLevel::Trace)));
        assert!(LevelFilterPreset::All.matches_level(None));

        assert!(!LevelFilterPreset::DebugPlus.matches_level(Some(&LogLevel::Trace)));
        assert!(LevelFilterPreset::DebugPlus.matches_level(Some(&LogLevel::Debug)));
        assert!(LevelFilterPreset::DebugPlus.matches_level(Some(&LogLevel::Fatal)));

        assert!(!LevelFilterPreset::InfoPlus.matches_level(Some(&LogLevel::Debug)));
        assert!(LevelFilterPreset::InfoPlus.matches_level(Some(&LogLevel::Info)));

        assert!(!LevelFilterPreset::WarnPlus.matches_level(Some(&LogLevel::Info)));
        assert!(LevelFilterPreset::WarnPlus.matches_level(Some(&LogLevel::Warn)));

        assert!(!LevelFilterPreset::ErrorPlus.matches_level(Some(&LogLevel::Warn)));
        assert!(LevelFilterPreset::ErrorPlus.matches_level(Some(&LogLevel::Error)));
        assert!(LevelFilterPreset::ErrorPlus.matches_level(Some(&LogLevel::Fatal)));
    }
}
