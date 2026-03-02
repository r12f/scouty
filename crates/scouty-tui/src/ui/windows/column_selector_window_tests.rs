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
            width_overrides: vec![None; 3],
            col_widths: [19, 5, 11, 3, 3, 9, 7, 8],
            theme: crate::config::Theme::default(),
            auto_widths: std::collections::HashMap::new(),
            width_overrides: std::collections::HashMap::new(),
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

    // ── Width adjustment tests ──────────────────────────────────────

    #[test]
    fn test_width_increase_with_l() {
        let mut w = sample_window();
        // cursor on Time (index 0), auto width = 19
        assert_eq!(w.width_display(0), "19");
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char('l'))),
            ComponentResult::Consumed
        );
        assert_eq!(w.width_overrides[0], Some(20));
        assert_eq!(w.width_display(0), "20");
    }

    #[test]
    fn test_width_decrease_with_h() {
        let mut w = sample_window();
        // Set override first so we can decrease
        w.width_overrides[0] = Some(21);
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char('h'))),
            ComponentResult::Consumed
        );
        assert_eq!(w.width_overrides[0], Some(20));
    }

    #[test]
    fn test_width_respects_min() {
        let mut w = sample_window();
        // Time min_width is 19, auto is 19, try to decrease
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char('h'))),
            ComponentResult::Consumed
        );
        // Should not go below 19
        assert_eq!(w.width_overrides[0], None); // unchanged since 19 - 1 = 18 < 19 min
    }

    #[test]
    fn test_width_left_right_arrows() {
        let mut w = sample_window();
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Right)),
            ComponentResult::Consumed
        );
        assert_eq!(w.width_overrides[0], Some(20));
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Left)),
            ComponentResult::Consumed
        );
        assert_eq!(w.width_overrides[0], Some(19));
    }

    #[test]
    fn test_width_reset_with_r() {
        let mut w = sample_window();
        w.width_overrides[0] = Some(25);
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char('r'))),
            ComponentResult::Consumed
        );
        assert_eq!(w.width_overrides[0], None);
    }

    #[test]
    fn test_log_column_not_adjustable() {
        let mut w = sample_window();
        w.cursor = 2; // Log column
        assert_eq!(w.width_display(2), "fill");
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char('l'))),
            ComponentResult::Consumed
        );
        assert_eq!(w.width_overrides[2], None); // unchanged
    }

    #[test]
    fn test_hidden_column_shows_dash() {
        let mut w = sample_window();
        w.columns[1].1 = false; // hide Level
        assert_eq!(w.width_display(1), "-");
    }

    #[test]
    fn test_hidden_column_not_adjustable() {
        let mut w = sample_window();
        w.columns[1].1 = false; // hide Level
        w.cursor = 1;
        assert_eq!(
            dispatch_key(&mut w, key(KeyCode::Char('l'))),
            ComponentResult::Consumed
        );
        assert_eq!(w.width_overrides[1], None); // unchanged
    }
}
