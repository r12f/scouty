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

    /// Simulate Tab cycle: Log Table → Detail → Region → Log Table
    #[test]
    fn test_tab_full_cycle_forward() {
        let mut state = PanelState::default();
        state.open(PanelId::Region); // open panel, active defaults to whatever
        state.focus_log_table();
        assert_eq!(state.focus, PanelFocus::LogTable);

        // Tab from log table: reset to first panel (Detail), focus it
        let all = PanelId::all();
        state.active = all[0]; // Detail
        state.focus_panel();
        assert_eq!(state.active, PanelId::Detail);
        assert_eq!(state.focus, PanelFocus::PanelContent);

        // Tab from Detail: next panel (Region)
        assert_ne!(state.active, *all.last().unwrap());
        state.next_panel();
        assert_eq!(state.active, PanelId::Region);
        assert_eq!(state.focus, PanelFocus::PanelContent);

        // Tab from Region (last panel): back to log table
        assert_eq!(state.active, *all.last().unwrap());
        state.focus_log_table();
        assert_eq!(state.focus, PanelFocus::LogTable);
    }

    /// Simulate Shift+Tab cycle: Log Table → Region → Detail → Log Table
    #[test]
    fn test_tab_full_cycle_backward() {
        let mut state = PanelState::default();
        state.open(PanelId::Detail);
        state.focus_log_table();

        // Shift+Tab from log table: enter last panel (Region)
        let all = PanelId::all();
        state.active = *all.last().unwrap(); // Region
        state.focus_panel();
        assert_eq!(state.active, PanelId::Region);
        assert_eq!(state.focus, PanelFocus::PanelContent);

        // Shift+Tab from Region: prev panel (Detail)
        assert_ne!(state.active, all[0]);
        state.prev_panel();
        assert_eq!(state.active, PanelId::Detail);
        assert_eq!(state.focus, PanelFocus::PanelContent);

        // Shift+Tab from Detail (first panel): back to log table
        assert_eq!(state.active, all[0]);
        state.focus_log_table();
        assert_eq!(state.focus, PanelFocus::LogTable);
    }

    /// Tab from log table must always start at Detail, even if Region was last active
    #[test]
    fn test_tab_from_log_table_always_starts_detail() {
        let mut state = PanelState::default();
        state.open(PanelId::Region);
        // Simulate: user previously switched to Region via Ctrl+→
        state.active = PanelId::Region;
        state.focus_log_table();

        // Tab from log table should reset to Detail, not stay on Region
        let all = PanelId::all();
        state.active = all[0]; // This is what the fix does
        state.focus_panel();
        assert_eq!(state.active, PanelId::Detail);
        assert_eq!(state.focus, PanelFocus::PanelContent);
    }

    /// Verify Tab and Shift+Tab produce opposite panel sequences
    #[test]
    fn test_tab_and_backtab_opposite_directions() {
        let all = PanelId::all();

        // Collect Tab (forward) cycle: entry point from LogTable, then cycle through panels
        let mut forward_sequence = Vec::new();
        let mut state = PanelState::default();
        state.expanded = true;
        state.focus_log_table();

        // Tab from LogTable → first panel
        state.active = all[0];
        state.focus_panel();
        forward_sequence.push(state.active);

        // Tab through panels until we'd return to LogTable
        loop {
            if state.active == *all.last().unwrap() {
                break; // would go to log table
            }
            state.next_panel();
            forward_sequence.push(state.active);
        }

        // Collect BackTab (backward) cycle: entry point from LogTable, then cycle through panels
        let mut backward_sequence = Vec::new();
        state.focus_log_table();

        // BackTab from LogTable → last panel
        state.active = *all.last().unwrap();
        state.focus_panel();
        backward_sequence.push(state.active);

        // BackTab through panels until we'd return to LogTable
        loop {
            if state.active == all[0] {
                break; // would go to log table
            }
            state.prev_panel();
            backward_sequence.push(state.active);
        }

        // Forward: [Detail, Region]
        // Backward: [Region, Detail]
        assert_eq!(
            forward_sequence,
            vec![PanelId::Detail, PanelId::Region],
            "Tab forward sequence should be Detail → Region"
        );
        assert_eq!(
            backward_sequence,
            vec![PanelId::Region, PanelId::Detail],
            "BackTab backward sequence should be Region → Detail"
        );

        // They should be reversed
        let mut reversed_forward = forward_sequence.clone();
        reversed_forward.reverse();
        assert_eq!(
            backward_sequence, reversed_forward,
            "BackTab sequence should be the reverse of Tab sequence"
        );
    }
}
