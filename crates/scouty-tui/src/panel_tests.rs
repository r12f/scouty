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
        assert_eq!(state.active, PanelId::Stats);
        state.next_panel();
        assert_eq!(state.active, PanelId::Category);
        state.next_panel();
        assert_eq!(state.active, PanelId::Detail);
        state.prev_panel();
        assert_eq!(state.active, PanelId::Category);
        state.prev_panel();
        assert_eq!(state.active, PanelId::Stats);
        state.prev_panel();
        assert_eq!(state.active, PanelId::Region);
    }

    #[test]
    fn test_toggle_maximize() {
        let mut state = PanelState::default();
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
        assert_eq!(PanelId::Stats.name(), "Stats");
    }

    #[test]
    fn test_panel_id_all() {
        let all = PanelId::all();
        assert_eq!(all.len(), 4);
        assert_eq!(all[0], PanelId::Detail);
        assert_eq!(all[1], PanelId::Region);
        assert_eq!(all[2], PanelId::Stats);
        assert_eq!(all[3], PanelId::Category);
    }

    #[test]
    fn test_panel_height_defaults() {
        assert_eq!(PanelId::Detail.default_height(), PanelHeight::FitContent);
        assert_eq!(
            PanelId::Region.default_height(),
            PanelHeight::Percentage(40)
        );
        assert_eq!(PanelId::Stats.default_height(), PanelHeight::Percentage(40));
        assert_eq!(
            PanelId::Category.default_height(),
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
        let mut state = PanelState::default();
        state.open(PanelId::Detail);
        assert!(state.expanded);
        assert!(!state.maximized);

        state.toggle_maximize();
        assert!(state.maximized);
        assert!(state.expanded);
        assert!(state.has_focus());

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

    /// Simulate Tab cycle: Log Table → Detail → Region → Stats → Log Table
    #[test]
    fn test_tab_full_cycle_forward() {
        let mut state = PanelState::default();
        state.open(PanelId::Region);
        state.focus_log_table();
        assert_eq!(state.focus, PanelFocus::LogTable);

        let all = PanelId::all();
        state.active = all[0];
        state.focus_panel();
        assert_eq!(state.active, PanelId::Detail);

        state.next_panel();
        assert_eq!(state.active, PanelId::Region);

        state.next_panel();
        assert_eq!(state.active, PanelId::Stats);

        state.next_panel();
        assert_eq!(state.active, PanelId::Category);

        assert_eq!(state.active, *all.last().unwrap());
        state.focus_log_table();
        assert_eq!(state.focus, PanelFocus::LogTable);
    }

    /// Simulate Shift+Tab cycle: Log Table → Stats → Region → Detail → Log Table
    #[test]
    fn test_tab_full_cycle_backward() {
        let mut state = PanelState::default();
        state.open(PanelId::Detail);
        state.focus_log_table();

        let all = PanelId::all();
        state.active = *all.last().unwrap(); // Category
        state.focus_panel();
        assert_eq!(state.active, PanelId::Category);

        state.prev_panel();
        assert_eq!(state.active, PanelId::Stats);

        state.prev_panel();
        assert_eq!(state.active, PanelId::Region);

        state.prev_panel();
        assert_eq!(state.active, PanelId::Detail);

        assert_eq!(state.active, all[0]);
        state.focus_log_table();
        assert_eq!(state.focus, PanelFocus::LogTable);
    }

    #[test]
    fn test_tab_from_log_table_always_starts_detail() {
        let mut state = PanelState::default();
        state.open(PanelId::Stats);
        state.active = PanelId::Stats;
        state.focus_log_table();

        let all = PanelId::all();
        state.active = all[0];
        state.focus_panel();
        assert_eq!(state.active, PanelId::Detail);
        assert_eq!(state.focus, PanelFocus::PanelContent);
    }

    /// Verify Tab and Shift+Tab produce opposite panel sequences
    #[test]
    fn test_tab_and_backtab_opposite_directions() {
        let all = PanelId::all();

        // Forward: LogTable → Detail → Region → Stats → LogTable
        let mut forward_sequence = Vec::new();
        let mut state = PanelState::default();
        state.expanded = true;
        state.focus_log_table();

        state.active = all[0];
        state.focus_panel();
        forward_sequence.push(state.active);

        loop {
            if state.active == *all.last().unwrap() {
                break;
            }
            state.next_panel();
            forward_sequence.push(state.active);
        }

        // Backward: LogTable → Stats → Region → Detail → LogTable
        let mut backward_sequence = Vec::new();
        state.focus_log_table();

        state.active = *all.last().unwrap();
        state.focus_panel();
        backward_sequence.push(state.active);

        loop {
            if state.active == all[0] {
                break;
            }
            state.prev_panel();
            backward_sequence.push(state.active);
        }

        assert_eq!(
            forward_sequence,
            vec![
                PanelId::Detail,
                PanelId::Region,
                PanelId::Stats,
                PanelId::Category
            ],
            "Tab forward sequence should be Detail → Region → Stats → Category"
        );
        assert_eq!(
            backward_sequence,
            vec![
                PanelId::Category,
                PanelId::Stats,
                PanelId::Region,
                PanelId::Detail
            ],
            "BackTab backward sequence should be Category → Stats → Region → Detail"
        );

        let mut reversed_forward = forward_sequence.clone();
        reversed_forward.reverse();
        assert_eq!(
            backward_sequence, reversed_forward,
            "BackTab sequence should be the reverse of Tab sequence"
        );
    }

    /// Stats panel opens/closes with S key toggle
    #[test]
    fn test_stats_panel_toggle() {
        let mut state = PanelState::default();
        assert!(!state.expanded);

        // S key opens Stats panel
        state.open(PanelId::Stats);
        assert!(state.expanded);
        assert_eq!(state.active, PanelId::Stats);
        assert!(state.has_focus());

        // S key again closes panel
        state.close();
        assert!(!state.expanded);
    }

    /// toggle_expand opens panel without changing focus
    #[test]
    fn test_toggle_expand_no_focus_change() {
        let mut state = PanelState::default();
        assert_eq!(state.focus, PanelFocus::LogTable);

        // Expand Detail — focus should stay on LogTable
        state.toggle_expand(PanelId::Detail);
        assert!(state.expanded);
        assert_eq!(state.active, PanelId::Detail);
        assert_eq!(state.focus, PanelFocus::LogTable);

        // Collapse Detail
        state.toggle_expand(PanelId::Detail);
        assert!(!state.expanded);
        assert_eq!(state.focus, PanelFocus::LogTable);
    }

    /// toggle_expand switches active panel without focus change
    #[test]
    fn test_toggle_expand_switch_panel() {
        let mut state = PanelState::default();

        state.toggle_expand(PanelId::Detail);
        assert!(state.expanded);
        assert_eq!(state.active, PanelId::Detail);

        // Switch to Region — stays expanded, focus unchanged
        state.toggle_expand(PanelId::Region);
        assert!(state.expanded);
        assert_eq!(state.active, PanelId::Region);
        assert_eq!(state.focus, PanelFocus::LogTable);
    }

    /// toggle_expand while panel focused keeps focus in panel
    #[test]
    fn test_toggle_expand_preserves_panel_focus() {
        let mut state = PanelState::default();
        state.open(PanelId::Detail); // opens AND focuses
        assert_eq!(state.focus, PanelFocus::PanelContent);

        // Switch to Region via toggle_expand — focus should stay as PanelContent
        state.toggle_expand(PanelId::Region);
        assert!(state.expanded);
        assert_eq!(state.active, PanelId::Region);
        assert_eq!(state.focus, PanelFocus::PanelContent);
    }

    #[test]
    fn test_dispatch_key_all_panels_callable() {
        // Verify dispatch_key works for every PanelId without panicking.
        // This ensures adding a new panel requires implementing dispatch_key.
        let mut app = crate::app::App::load_stdin(Vec::new()).unwrap();
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('x'),
            crossterm::event::KeyModifiers::NONE,
        );
        for &panel in PanelId::all() {
            // Should not panic — every panel has a dispatch_key arm
            let _action = panel.dispatch_key(&mut app, key);
        }
    }

    #[test]
    fn test_shortcut_hints_all_panels() {
        // Every panel should return hints without panicking.
        for &panel in PanelId::all() {
            let hints = panel.shortcut_hints();
            // Hints is a vec (possibly empty for Stats)
            assert!(hints.len() <= 20, "sanity: too many hints for {:?}", panel);
        }
    }

    #[test]
    fn test_dispatch_key_detail_handles_j() {
        // Detail panel should handle 'j' key
        let mut app = crate::app::App::load_stdin(Vec::new()).unwrap();
        app.detail_open = true;
        app.detail_tree_focus = true;
        let key = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('j'),
            crossterm::event::KeyModifiers::NONE,
        );
        let action = PanelId::Detail.dispatch_key(&mut app, key);
        assert_eq!(action, crate::ui::framework::KeyAction::Handled);
    }
}
