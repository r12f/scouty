//! Tests for SaveDialogWindow.

mod tests {
    use crate::app::{App, ColumnConfig, DensitySource, ExportFormat, InputMode};
    use crate::config::Theme;
    use crate::text_input::TextInput;
    use crate::ui::windows::save_dialog_window::SaveDialogWindow;
    use crate::ui::{ComponentResult, UiComponent};
    use chrono::Utc;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use scouty::record::{LogLevel, LogRecord};
    use std::sync::Arc;

    fn make_record(id: u64, level: Option<LogLevel>, message: &str) -> LogRecord {
        LogRecord {
            id,
            timestamp: Utc::now(),
            level,
            source: "test".into(),
            pid: None,
            tid: None,
            component_name: None,
            process_name: None,
            message: message.to_string(),
            hostname: None,
            container: None,
            context: None,
            function: None,
            raw: message.to_string(),
            metadata: None,
            loader_id: "test".into(),
            expanded: None,
        }
    }

    fn make_test_app(n: usize) -> App {
        let records: Vec<Arc<LogRecord>> = (0..n)
            .map(|i| {
                Arc::new(make_record(
                    i as u64,
                    Some(LogLevel::Info),
                    &format!("msg {}", i),
                ))
            })
            .collect();
        let filtered_indices = (0..n).collect();
        App {
            records,
            total_records: n,
            filtered_indices,
            scroll_offset: 0,
            selected: 0,
            visible_rows: 10,
            detail_open: false,
            detail_panel_ratio: 0.3,
            detail_tree_cursor: 0,
            detail_tree_collapsed: std::collections::HashSet::new(),
            detail_tree_focus: false,
            panel_state: crate::panel::PanelState::default(),
            input_mode: InputMode::Normal,
            filter_input: TextInput::new(),
            filter_error: None,
            filters: Vec::new(),
            quick_filter_input: TextInput::new(),
            field_filter: None,
            filter_manager_cursor: 0,
            search_input: TextInput::new(),
            search_matches: vec![],
            search_match_idx: None,
            time_input: TextInput::new(),
            goto_input: TextInput::new(),
            status_message: None,
            shortcut_hints_cache: Vec::new(),
            status_message_at: None,
            col_widths: [19, 5, 11, 3, 3, 9],
            column_config: ColumnConfig::default(),
            follow_mode: false,
            follow_new_count: 0,
            should_quit: false,
            copy_format_cursor: 0,
            save_path_input: TextInput::with_text("./scouty-export.log"),
            save_format_cursor: 0,
            save_dialog_focus: crate::ui::windows::save_dialog_window::Focus::Path,
            help_scroll: 0,
            command_input: TextInput::new(),
            filter_version: 0,
            density_cache: None,
            highlight_rules: Vec::new(),
            highlight_input: TextInput::new(),
            highlight_manager_cursor: 0,
            bookmarks: std::collections::HashSet::new(),
            bookmark_manager_cursor: 0,
            theme: Theme::default(),
            level_filter: None,
            level_filter_cursor: 0,
            preset_name_input: TextInput::new(),
            preset_list: Vec::new(),
            preset_list_cursor: 0,
            density_source: DensitySource::All,
            density_selector_cursor: 0,
            regions: scouty::region::store::RegionStore::default(),
            region_processor: None,
            search_regex: None,
            category_processor: None,
            category_cursor: 0,
            region_manager_cursor: 0,
            region_panel_sort: crate::ui::widgets::region_panel_widget::RegionSortMode::StartTime,
            region_panel_type_filter: None,
        }
    }

    #[test]
    fn test_save_dialog_defaults() {
        let app = make_test_app(2);
        let window = SaveDialogWindow::from_app(&app);
        assert_eq!(window.path_input.value(), "./scouty-export.log");
        assert_eq!(window.format_cursor, 0);
        assert!(!window.confirmed);
        assert!(window.error.is_none());
    }

    #[test]
    fn test_format_selection() {
        let app = make_test_app(2);
        let mut window = SaveDialogWindow::from_app(&app);
        assert_eq!(window.selected_format(), ExportFormat::Raw);

        window.on_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        window.on_down();
        assert_eq!(window.selected_format(), ExportFormat::Json);

        window.on_down();
        assert_eq!(window.selected_format(), ExportFormat::Yaml);

        window.on_down();
        assert_eq!(window.selected_format(), ExportFormat::Yaml);

        window.on_up();
        assert_eq!(window.selected_format(), ExportFormat::Json);
    }

    #[test]
    fn test_empty_path_error() {
        let app = make_test_app(2);
        let mut window = SaveDialogWindow::from_app(&app);
        window.path_input.clear();
        let result = window.on_confirm();
        assert_eq!(result, ComponentResult::Consumed);
        assert!(!window.confirmed);
        assert_eq!(window.error.as_deref(), Some("Path required"));
    }

    #[test]
    fn test_confirm_valid_path() {
        let app = make_test_app(2);
        let mut window = SaveDialogWindow::from_app(&app);
        let result = window.on_confirm();
        assert_eq!(result, ComponentResult::Close);
        assert!(window.confirmed);
    }

    #[test]
    fn test_cancel() {
        let app = make_test_app(2);
        let mut window = SaveDialogWindow::from_app(&app);
        let result = window.on_cancel();
        assert_eq!(result, ComponentResult::Close);
        assert!(!window.confirmed);
    }

    #[test]
    fn test_tilde_expansion() {
        let app = make_test_app(2);
        let mut window = SaveDialogWindow::from_app(&app);
        window.path_input.set("~/logs/export.log");
        let path = window.expanded_path();
        if let Some(home) = dirs::home_dir() {
            let expected = home.join("logs/export.log").to_string_lossy().to_string();
            assert_eq!(path, expected);
        }
    }

    #[test]
    fn test_raw_export() {
        let app = make_test_app(3);
        let tmp = std::env::temp_dir().join("scouty_save_raw.log");
        let msg = SaveDialogWindow::execute_save(&app, tmp.to_str().unwrap(), ExportFormat::Raw);
        assert!(msg.contains("Saved 3 records"));
        assert!(msg.contains("raw"));
        let content = std::fs::read_to_string(&tmp).unwrap();
        assert!(content.contains("msg 0"));
        assert!(content.contains("msg 2"));
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_json_export() {
        let app = make_test_app(2);
        let tmp = std::env::temp_dir().join("scouty_save.json");
        let msg = SaveDialogWindow::execute_save(&app, tmp.to_str().unwrap(), ExportFormat::Json);
        assert!(msg.contains("Saved 2 records"));
        assert!(msg.contains("json"));
        let content = std::fs::read_to_string(&tmp).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 2);
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_yaml_export() {
        let app = make_test_app(2);
        let tmp = std::env::temp_dir().join("scouty_save.yaml");
        let msg = SaveDialogWindow::execute_save(&app, tmp.to_str().unwrap(), ExportFormat::Yaml);
        assert!(msg.contains("Saved 2 records"));
        assert!(msg.contains("yaml"));
        let content = std::fs::read_to_string(&tmp).unwrap();
        assert!(content.contains("msg 0"));
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_tab_switches_focus() {
        let app = make_test_app(2);
        let mut window = SaveDialogWindow::from_app(&app);
        assert!(!window.enable_jk_navigation());

        window.on_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert!(window.enable_jk_navigation());

        window.on_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert!(!window.enable_jk_navigation());
    }

    #[test]
    fn test_char_input_via_dispatch() {
        let app = make_test_app(1);
        let mut window = SaveDialogWindow::from_app(&app);
        // Clear default path
        window.path_input.clear();

        // Type via dispatch_key (same path as main.rs)
        let result = crate::ui::dispatch_key(
            &mut window,
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
        );
        assert_eq!(result, ComponentResult::Consumed);
        crate::ui::dispatch_key(
            &mut window,
            KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE),
        );
        crate::ui::dispatch_key(
            &mut window,
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE),
        );
        assert_eq!(window.path_input.value(), "abc");
    }

    #[test]
    fn test_space_in_path_input() {
        let app = make_test_app(1);
        let mut window = SaveDialogWindow::from_app(&app);
        window.path_input.clear();

        crate::ui::dispatch_key(
            &mut window,
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
        );
        crate::ui::dispatch_key(
            &mut window,
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
        );
        crate::ui::dispatch_key(
            &mut window,
            KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE),
        );
        assert_eq!(window.path_input.value(), "a b");
    }
}
