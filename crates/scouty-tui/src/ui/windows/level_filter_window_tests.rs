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
            ('2', LevelFilterPreset::TracePlus),
            ('3', LevelFilterPreset::DebugPlus),
            ('4', LevelFilterPreset::InfoPlus),
            ('5', LevelFilterPreset::NoticePlus),
            ('6', LevelFilterPreset::WarnPlus),
            ('7', LevelFilterPreset::ErrorPlus),
            ('8', LevelFilterPreset::FatalOnly),
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

        // j moves down
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

        // cursor=2 is DebugPlus (0=All, 1=TracePlus, 2=DebugPlus)
        let result = dispatch_key(&mut w, key(KeyCode::Enter));
        assert_eq!(result, ComponentResult::Close);
        assert!(w.confirmed);
        assert_eq!(w.selected, Some(LevelFilterPreset::DebugPlus));
    }

    #[test]
    fn test_navigation_bounds() {
        let theme = Theme::default();
        let mut w = LevelFilterWindow::new(None, &theme);

        // Can't go above 0
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Up)),
            ComponentResult::Consumed
        );
        assert_eq!(w.cursor, 0);

        // Navigate past the end
        for _ in 0..20 {
            dispatch_key(&mut w, key(KeyCode::Char('j')));
        }
        assert_eq!(w.cursor, 7); // 8 options, max index = 7
    }

    #[test]
    fn test_current_level_cursor_position() {
        let theme = Theme::default();
        // WarnPlus is number 6, index 5
        let w = LevelFilterWindow::new(Some(LevelFilterPreset::WarnPlus), &theme);
        assert_eq!(w.cursor, 5);
    }

    #[test]
    fn test_level_filter_preset_matches() {
        use scouty::record::LogLevel;

        // All matches everything
        assert!(LevelFilterPreset::All.matches_level(Some(&LogLevel::Trace)));
        assert!(LevelFilterPreset::All.matches_level(None));

        // TracePlus
        assert!(LevelFilterPreset::TracePlus.matches_level(Some(&LogLevel::Trace)));
        assert!(LevelFilterPreset::TracePlus.matches_level(Some(&LogLevel::Fatal)));
        assert!(!LevelFilterPreset::TracePlus.matches_level(None));

        // DebugPlus
        assert!(!LevelFilterPreset::DebugPlus.matches_level(Some(&LogLevel::Trace)));
        assert!(LevelFilterPreset::DebugPlus.matches_level(Some(&LogLevel::Debug)));
        assert!(LevelFilterPreset::DebugPlus.matches_level(Some(&LogLevel::Fatal)));

        // InfoPlus
        assert!(!LevelFilterPreset::InfoPlus.matches_level(Some(&LogLevel::Debug)));
        assert!(LevelFilterPreset::InfoPlus.matches_level(Some(&LogLevel::Info)));

        // NoticePlus
        assert!(!LevelFilterPreset::NoticePlus.matches_level(Some(&LogLevel::Info)));
        assert!(LevelFilterPreset::NoticePlus.matches_level(Some(&LogLevel::Notice)));
        assert!(LevelFilterPreset::NoticePlus.matches_level(Some(&LogLevel::Fatal)));

        // WarnPlus
        assert!(!LevelFilterPreset::WarnPlus.matches_level(Some(&LogLevel::Notice)));
        assert!(LevelFilterPreset::WarnPlus.matches_level(Some(&LogLevel::Warn)));

        // ErrorPlus
        assert!(!LevelFilterPreset::ErrorPlus.matches_level(Some(&LogLevel::Warn)));
        assert!(LevelFilterPreset::ErrorPlus.matches_level(Some(&LogLevel::Error)));
        assert!(LevelFilterPreset::ErrorPlus.matches_level(Some(&LogLevel::Fatal)));

        // FatalOnly
        assert!(!LevelFilterPreset::FatalOnly.matches_level(Some(&LogLevel::Error)));
        assert!(LevelFilterPreset::FatalOnly.matches_level(Some(&LogLevel::Fatal)));
        assert!(!LevelFilterPreset::FatalOnly.matches_level(None));
    }
}
