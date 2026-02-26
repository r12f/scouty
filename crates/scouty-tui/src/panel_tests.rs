#[cfg(test)]
mod tests {
    use crate::panel::*;

    #[test]
    fn test_default_state() {
        let state = PanelState::default();
        assert_eq!(state.active, PanelId::Detail);
        assert!(!state.expanded);
        assert_eq!(state.focus, PanelFocus::LogTable);
        assert!(!state.maximized);
    }

    #[test]
    fn test_open_panel() {
        let mut state = PanelState::default();
        state.open(PanelId::Region);
        assert_eq!(state.active, PanelId::Region);
        assert!(state.expanded);
        assert!(state.has_focus());
    }

    #[test]
    fn test_close_panel() {
        let mut state = PanelState::default();
        state.open(PanelId::Detail);
        state.close();
        assert!(!state.expanded);
        assert!(!state.has_focus());
        assert!(!state.maximized);
    }

    #[test]
    fn test_focus_panel_expands_if_collapsed() {
        let mut state = PanelState::default();
        assert!(!state.expanded);
        state.focus_panel();
        assert!(state.expanded);
        assert!(state.has_focus());
    }

    #[test]
    fn test_focus_log_table() {
        let mut state = PanelState::default();
        state.open(PanelId::Detail);
        state.focus_log_table();
        assert!(!state.has_focus());
        assert!(state.expanded); // stays expanded
    }

    #[test]
    fn test_next_prev_panel() {
        let mut state = PanelState::default();
        assert_eq!(state.active, PanelId::Detail);
        state.next_panel();
        assert_eq!(state.active, PanelId::Region);
        state.next_panel();
        assert_eq!(state.active, PanelId::Detail);
        state.prev_panel();
        assert_eq!(state.active, PanelId::Region);
    }

    #[test]
    fn test_toggle_maximize() {
        let mut state = PanelState::default();
        // Can't maximize when collapsed
        state.toggle_maximize();
        assert!(!state.maximized);

        state.open(PanelId::Detail);
        state.toggle_maximize();
        assert!(state.maximized);
        state.toggle_maximize();
        assert!(!state.maximized);
    }

    #[test]
    fn test_panel_id_names() {
        assert_eq!(PanelId::Detail.name(), "Detail");
        assert_eq!(PanelId::Region.name(), "Region");
    }

    #[test]
    fn test_panel_id_all() {
        let all = PanelId::all();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0], PanelId::Detail);
        assert_eq!(all[1], PanelId::Region);
    }

    #[test]
    fn test_panel_height_defaults() {
        assert_eq!(PanelId::Detail.default_height(), PanelHeight::FitContent);
        assert_eq!(
            PanelId::Region.default_height(),
            PanelHeight::Percentage(40)
        );
    }

    #[test]
    fn test_is_content_visible() {
        let mut state = PanelState::default();
        assert!(!state.is_content_visible());
        state.open(PanelId::Detail);
        assert!(state.is_content_visible());
    }

    #[test]
    fn test_maximize_hides_log_table() {
        // When maximized, panel_state should indicate the log table should be hidden
        let mut state = PanelState::default();
        state.open(PanelId::Detail);
        assert!(state.expanded);
        assert!(!state.maximized);

        state.toggle_maximize();
        assert!(state.maximized);
        assert!(state.expanded);
        assert!(state.has_focus());

        // Verify restore
        state.toggle_maximize();
        assert!(!state.maximized);
        assert!(state.expanded);
    }

    #[test]
    fn test_close_clears_maximize() {
        let mut state = PanelState::default();
        state.open(PanelId::Detail);
        state.toggle_maximize();
        assert!(state.maximized);

        state.close();
        assert!(!state.maximized);
        assert!(!state.expanded);
    }
}
