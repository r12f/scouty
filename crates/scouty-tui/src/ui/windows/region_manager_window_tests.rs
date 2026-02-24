//! Tests for RegionManagerWindow.

mod tests {
    use crate::ui::windows::region_manager_window::{
        RegionAction, RegionEntry, RegionManagerWindow,
    };
    use crate::ui::{dispatch_key, ComponentResult, UiComponent};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn make_window(n: usize) -> RegionManagerWindow {
        let entries: Vec<RegionEntry> = (0..n)
            .map(|i| RegionEntry {
                name: format!("Region {}", i),
                definition_name: "test_region".to_string(),
                time_range: format!("10:00:0{} → 10:00:1{}", i, i),
                start_index: i * 10,
                end_index: i * 10 + 5,
            })
            .collect();
        RegionManagerWindow {
            cursor: 0,
            entries,
            action: None,
        }
    }

    #[test]
    fn test_navigation() {
        let mut w = make_window(5);
        assert_eq!(w.cursor, 0);

        dispatch_key(&mut w, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(w.cursor, 1);

        dispatch_key(&mut w, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(w.cursor, 2);

        dispatch_key(&mut w, KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(w.cursor, 1);
    }

    #[test]
    fn test_up_at_top_stays() {
        let mut w = make_window(3);
        dispatch_key(&mut w, KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(w.cursor, 0);
    }

    #[test]
    fn test_down_at_bottom_stays() {
        let mut w = make_window(3);
        w.cursor = 2;
        dispatch_key(&mut w, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(w.cursor, 2);
    }

    #[test]
    fn test_enter_jumps() {
        let mut w = make_window(3);
        w.cursor = 1;
        let result = dispatch_key(&mut w, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(result, ComponentResult::Close);
        assert!(matches!(w.action, Some(RegionAction::Jump(10))));
    }

    #[test]
    fn test_f_filters() {
        let mut w = make_window(3);
        w.cursor = 2;
        let result = dispatch_key(
            &mut w,
            KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE),
        );
        assert_eq!(result, ComponentResult::Close);
        assert!(matches!(w.action, Some(RegionAction::Filter(20, 25))));
    }

    #[test]
    fn test_esc_closes() {
        let mut w = make_window(3);
        let result = dispatch_key(&mut w, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(result, ComponentResult::Close);
        assert!(w.action.is_none());
    }

    #[test]
    fn test_empty_entries() {
        let mut w = make_window(0);
        let result = dispatch_key(&mut w, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(result, ComponentResult::Consumed);
    }
}
