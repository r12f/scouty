#[cfg(test)]
mod tests {
    use crate::app::InputMode;
    use crate::keybinding::Keymap;
    use crate::ui::framework::{Window, WindowAction};
    use crate::ui::windows::main_window::MainWindow;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl_key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    fn make_main_window() -> MainWindow {
        // Create minimal app from empty stdin
        let app = crate::app::App::load_stdin(Vec::new()).unwrap();
        let keymap = Keymap::default_keymap();
        MainWindow::new(app, keymap)
    }

    /// Create a MainWindow with multiple log records so cursor movement is possible.
    fn make_main_window_with_records() -> MainWindow {
        let lines: Vec<String> = (0..10)
            .map(|i| format!("2025-01-01T00:00:0{}Z INFO test message {}", i, i))
            .collect();
        let app = crate::app::App::load_stdin(lines).unwrap();
        let keymap = Keymap::default_keymap();
        MainWindow::new(app, keymap)
    }

    #[test]
    fn test_normal_mode_quit() {
        let mut mw = make_main_window();
        let result = mw.handle_key(key(KeyCode::Char('q')));
        assert_eq!(result, WindowAction::Close);
    }

    #[test]
    fn test_normal_mode_move_down() {
        let mut mw = make_main_window();
        let result = mw.handle_key(key(KeyCode::Char('j')));
        assert_eq!(result, WindowAction::Handled);
    }

    #[test]
    fn test_normal_mode_open_help() {
        let mut mw = make_main_window();
        let result = mw.handle_key(key(KeyCode::Char('?')));
        assert_eq!(result, WindowAction::Handled);
        assert_eq!(mw.app.input_mode, InputMode::Help);
    }

    #[test]
    fn test_normal_mode_open_filter() {
        let mut mw = make_main_window();
        let result = mw.handle_key(key(KeyCode::Char('f')));
        assert_eq!(result, WindowAction::Handled);
        assert_eq!(mw.app.input_mode, InputMode::Filter);
    }

    #[test]
    fn test_normal_mode_open_search() {
        let mut mw = make_main_window();
        // Default keymap: / for search
        let result = mw.handle_key(key(KeyCode::Char('/')));
        assert_eq!(result, WindowAction::Handled);
        assert_eq!(mw.app.input_mode, InputMode::Search);
    }

    #[test]
    fn test_overlay_filter_esc_returns_normal() {
        let mut mw = make_main_window();
        mw.app.input_mode = InputMode::Filter;
        let result = mw.handle_key(key(KeyCode::Esc));
        assert_eq!(result, WindowAction::Handled);
        assert_eq!(mw.app.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_overlay_help_esc_returns_normal() {
        let mut mw = make_main_window();
        mw.app.input_mode = InputMode::Help;
        let result = mw.handle_key(key(KeyCode::Esc));
        assert_eq!(result, WindowAction::Handled);
        assert_eq!(mw.app.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_tab_handled() {
        let mut mw = make_main_window();
        let result = mw.handle_key(key(KeyCode::Tab));
        assert_eq!(result, WindowAction::Handled);
    }

    #[test]
    fn test_window_name() {
        let mw = make_main_window();
        assert_eq!(mw.name(), "MainWindow");
    }

    #[test]
    fn test_ctrl_arrows_no_longer_focus_panel() {
        let mut mw = make_main_window();
        // Ctrl+Down should NOT focus panel anymore
        let _result = mw.handle_key(ctrl_key(KeyCode::Down));
        assert!(!mw.app.panel_state.has_focus());
    }

    #[test]
    fn test_detail_tree_nav_when_focused() {
        let mut mw = make_main_window();
        // Set panel focus to Detail via panel system
        mw.app.panel_state.expanded = true;
        mw.app.panel_state.active = crate::panel::PanelId::Detail;
        mw.app.panel_state.focus = crate::panel::PanelFocus::PanelContent;

        // j should move down in detail tree
        let result = mw.handle_key(key(KeyCode::Char('j')));
        assert_eq!(result, WindowAction::Handled);

        // Right arrow should toggle/expand node
        let result = mw.handle_key(key(KeyCode::Right));
        assert_eq!(result, WindowAction::Handled);

        // Left arrow should collapse/go to parent
        let result = mw.handle_key(key(KeyCode::Left));
        assert_eq!(result, WindowAction::Handled);

        // Esc should exit panel focus
        let result = mw.handle_key(key(KeyCode::Esc));
        assert_eq!(result, WindowAction::Handled);
        assert!(!mw.app.panel_state.has_focus());
    }

    #[test]
    fn test_detail_focus_blocks_log_table_navigation() {
        let mut mw = make_main_window_with_records();
        assert!(
            mw.app.filtered_indices.len() > 1,
            "need multiple records so cursor can move"
        );

        // First, verify j DOES move cursor when log table has focus (no panel focus)
        let selected_before = mw.app.selected;
        mw.handle_key(key(KeyCode::Char('j')));
        assert_ne!(
            mw.app.selected, selected_before,
            "sanity check: j should move cursor when log table has focus"
        );

        // Reset cursor
        mw.app.selected = 0;

        // Now focus the detail panel
        mw.app.panel_state.expanded = true;
        mw.app.panel_state.active = crate::panel::PanelId::Detail;
        mw.app.panel_state.focus = crate::panel::PanelFocus::PanelContent;

        let selected_before = mw.app.selected;

        // j should go to detail panel, NOT move log table cursor
        mw.handle_key(key(KeyCode::Char('j')));
        assert_eq!(
            mw.app.selected, selected_before,
            "log table cursor should not move when detail panel has focus"
        );
    }

    #[test]
    fn test_stats_panel_focus_blocks_log_table_keys() {
        // Stats panel is read-only — all keys return Unhandled.
        // Verify that unhandled keys do NOT leak to the log table.
        let mut mw = make_main_window_with_records();
        assert!(mw.app.filtered_indices.len() > 1);

        // Sanity: j moves cursor when log table focused
        let before = mw.app.selected;
        mw.handle_key(key(KeyCode::Char('j')));
        assert_ne!(mw.app.selected, before);
        mw.app.selected = 0;

        // Focus Stats panel
        mw.app.panel_state.active = crate::panel::PanelId::Stats;
        mw.app.panel_state.focus_panel();

        let before = mw.app.selected;
        let result = mw.handle_normal_key(key(KeyCode::Char('j')));
        assert_eq!(result, WindowAction::Handled);
        assert_eq!(
            mw.app.selected, before,
            "log table cursor must not move when Stats panel has focus"
        );
    }

    #[test]
    fn test_search_input_mode_typing() {
        let mut mw = make_main_window();
        mw.app.input_mode = InputMode::Search;
        // Type a character
        let result = mw.handle_key(key(KeyCode::Char('a')));
        assert_eq!(result, WindowAction::Handled);
        // Esc returns to normal
        let result = mw.handle_key(key(KeyCode::Esc));
        assert_eq!(result, WindowAction::Handled);
        assert_eq!(mw.app.input_mode, InputMode::Normal);
    }
}

#[cfg(test)]
mod quit_from_panel_tests {
    use crate::keybinding::Keymap;
    use crate::panel::{PanelFocus, PanelId};
    use crate::ui::framework::{Window, WindowAction};
    use crate::ui::windows::main_window::MainWindow;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn make_main_window() -> MainWindow {
        let app = crate::app::App::load_stdin(Vec::new()).unwrap();
        let keymap = Keymap::default_keymap();
        MainWindow::new(app, keymap)
    }

    #[test]
    fn test_quit_from_detail_panel_focus() {
        let mut mw = make_main_window();
        mw.app.panel_state.expanded = true;
        mw.app.panel_state.active = PanelId::Detail;
        mw.app.panel_state.focus = PanelFocus::PanelContent;
        assert!(mw.app.panel_state.has_focus());

        let result = mw.handle_key(key(KeyCode::Char('q')));
        assert_eq!(result, WindowAction::Close, "q should quit even when detail panel is focused");
    }

    #[test]
    fn test_quit_from_region_panel_focus() {
        let mut mw = make_main_window();
        mw.app.panel_state.expanded = true;
        mw.app.panel_state.active = PanelId::Region;
        mw.app.panel_state.focus = PanelFocus::PanelContent;
        assert!(mw.app.panel_state.has_focus());

        let result = mw.handle_key(key(KeyCode::Char('q')));
        assert_eq!(result, WindowAction::Close, "q should quit even when region panel is focused");
    }

    #[test]
    fn test_quit_from_category_panel_focus() {
        let mut mw = make_main_window();
        mw.app.panel_state.expanded = true;
        mw.app.panel_state.active = PanelId::Category;
        mw.app.panel_state.focus = PanelFocus::PanelContent;
        assert!(mw.app.panel_state.has_focus());

        let result = mw.handle_key(key(KeyCode::Char('q')));
        assert_eq!(result, WindowAction::Close, "q should quit even when category panel is focused");
    }
}
