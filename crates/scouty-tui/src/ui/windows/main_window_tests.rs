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
        mw.app.detail_open = true;
        mw.app.detail_tree_focus = true;

        // j should move down in detail tree
        let result = mw.handle_key(key(KeyCode::Char('j')));
        assert_eq!(result, WindowAction::Handled);

        // Right arrow should toggle/expand node
        let result = mw.handle_key(key(KeyCode::Right));
        assert_eq!(result, WindowAction::Handled);

        // Left arrow should collapse/go to parent
        let result = mw.handle_key(key(KeyCode::Left));
        assert_eq!(result, WindowAction::Handled);

        // Esc should exit detail tree focus
        let result = mw.handle_key(key(KeyCode::Esc));
        assert_eq!(result, WindowAction::Handled);
        assert!(!mw.app.detail_tree_focus);
    }

    #[test]
    fn test_panel_focus_blocks_log_table_keys() {
        // When a panel has focus, j/k should NOT move the log table cursor.
        let mut mw = make_main_window();
        // Tab into panel system (first panel = Detail)
        mw.handle_key(key(KeyCode::Tab));
        assert!(mw.app.panel_state.has_focus());
        assert_eq!(
            mw.app.panel_state.active,
            crate::panel::PanelId::Detail
        );

        // 'q' should NOT quit when panel has focus
        let result = mw.handle_normal_key(key(KeyCode::Char('q')));
        assert_ne!(result, WindowAction::Close);
    }

    #[test]
    fn test_tab_into_detail_sets_tree_focus() {
        let mut mw = make_main_window();
        // Tab into panels — first panel is Detail
        mw.handle_key(key(KeyCode::Tab));
        assert!(mw.app.panel_state.has_focus());
        assert_eq!(
            mw.app.panel_state.active,
            crate::panel::PanelId::Detail
        );
        assert!(
            mw.app.detail_tree_focus,
            "detail_tree_focus should be set when Tab enters Detail panel"
        );
    }

    #[test]
    fn test_backtab_into_detail_sets_tree_focus() {
        let mut mw = make_main_window();
        // Tab into panels, then navigate with Tab until we wrap to Detail
        // BackTab from log table goes to last panel (Category), then prev...
        // Simpler: Tab in, Tab to Region, BackTab back to Detail
        mw.handle_key(key(KeyCode::Tab)); // → Detail (focus)
        mw.handle_key(key(KeyCode::Tab)); // → Region
        assert_eq!(
            mw.app.panel_state.active,
            crate::panel::PanelId::Region
        );
        assert!(!mw.app.detail_tree_focus);

        // BackTab back to Detail
        let backtab = KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT);
        mw.handle_key(backtab);
        assert_eq!(
            mw.app.panel_state.active,
            crate::panel::PanelId::Detail
        );
        assert!(
            mw.app.detail_tree_focus,
            "detail_tree_focus should be set when BackTab enters Detail panel"
        );
    }

    #[test]
    fn test_stats_panel_focus_blocks_log_keys() {
        // Stats panel is read-only but should still block log table keys
        let mut mw = make_main_window();
        // Manually set focus to Stats panel
        mw.app.panel_state.active = crate::panel::PanelId::Stats;
        mw.app.panel_state.focus_panel();
        assert!(mw.app.panel_state.has_focus());

        // j should NOT move the log table
        let selected_before = mw.app.selected;
        let result = mw.handle_normal_key(key(KeyCode::Char('j')));
        assert_eq!(result, WindowAction::Handled);
        assert_eq!(mw.app.selected, selected_before);
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
