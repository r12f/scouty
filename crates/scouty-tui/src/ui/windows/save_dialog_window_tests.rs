//! Tests for SaveDialogWindow.

mod tests {
    use super::*;
    use crate::app::{App, ExportFormat, InputMode};
    use crate::ui::ComponentResult;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn make_test_app() -> App {
        use scouty::record::{LogLevel, LogRecord};
        use std::sync::Arc;

        let records: Vec<Arc<LogRecord>> = vec![
            Arc::new(LogRecord {
                id: 0,
                timestamp: chrono::Utc::now(),
                level: Some(LogLevel::Info),
                source: "test".into(),
                process_name: None,
                pid: None,
                tid: None,
                component_name: None,
                function: None,
                hostname: None,
                container: None,
                context: None,
                metadata: None,
                message: "hello world".to_string(),
                raw: "2026-01-01 INFO hello world".to_string(),
                loader_id: "test".into(),
            }),
            Arc::new(LogRecord {
                id: 1,
                timestamp: chrono::Utc::now(),
                level: Some(LogLevel::Error),
                source: "test".into(),
                process_name: None,
                pid: None,
                tid: None,
                component_name: None,
                function: None,
                hostname: None,
                container: None,
                context: None,
                metadata: None,
                message: "something failed".to_string(),
                raw: "2026-01-01 ERROR something failed".to_string(),
                loader_id: "test".into(),
            }),
        ];

        // Use the same full init pattern as app.rs tests
        let n = records.len();
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
            input_mode: InputMode::Normal,
            filter_input: crate::text_input::TextInput::new(),
            filter_error: None,
            filters: Vec::new(),
            quick_filter_input: crate::text_input::TextInput::new(),
            field_filter: None,
            filter_manager_cursor: 0,
            search_input: crate::text_input::TextInput::new(),
            search_matches: vec![],
            search_match_idx: None,
            time_input: crate::text_input::TextInput::new(),
            goto_input: crate::text_input::TextInput::new(),
            status_message: None,
            status_message_at: None,
            col_widths: [19, 5, 11, 3, 3, 9],
            column_config: crate::app::ColumnConfig::default(),
            follow_mode: false,
            should_quit: false,
            copy_format_cursor: 0,
            save_path_input: crate::text_input::TextInput::with_text("./scouty-export.log"),
            save_format_cursor: 0,
            help_scroll: 0,
            command_input: crate::text_input::TextInput::new(),
            filter_version: 0,
            density_cache: None,
            highlight_rules: Vec::new(),
            highlight_input: crate::text_input::TextInput::new(),
            highlight_manager_cursor: 0,
            cached_stats: None,
            bookmarks: std::collections::HashSet::new(),
            bookmark_manager_cursor: 0,
            theme: crate::config::Theme::default(),
        }
    }

    #[test]
    fn test_save_dialog_opens_with_defaults() {
        let app = make_test_app();
        let window = SaveDialogWindow::from_app(&app);
        assert_eq!(window.path_input.value(), "./scouty-export.log");
        assert_eq!(window.format_cursor, 0);
        assert!(!window.confirmed);
        assert!(window.error.is_none());
    }

    #[test]
    fn test_format_selection() {
        let app = make_test_app();
        let mut window = SaveDialogWindow::from_app(&app);
        assert_eq!(window.selected_format(), ExportFormat::Raw);

        // Switch to format focus and move down
        window.on_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        window.on_down();
        assert_eq!(window.selected_format(), ExportFormat::Json);

        window.on_down();
        assert_eq!(window.selected_format(), ExportFormat::Yaml);

        // Can't go past end
        window.on_down();
        assert_eq!(window.selected_format(), ExportFormat::Yaml);

        window.on_up();
        assert_eq!(window.selected_format(), ExportFormat::Json);
    }

    #[test]
    fn test_empty_path_error() {
        let app = make_test_app();
        let mut window = SaveDialogWindow::from_app(&app);
        window.path_input.clear();
        let result = window.on_confirm();
        assert_eq!(result, ComponentResult::Consumed);
        assert!(!window.confirmed);
        assert_eq!(window.error.as_deref(), Some("Path required"));
    }

    #[test]
    fn test_confirm_with_valid_path() {
        let app = make_test_app();
        let mut window = SaveDialogWindow::from_app(&app);
        let result = window.on_confirm();
        assert_eq!(result, ComponentResult::Close);
        assert!(window.confirmed);
    }

    #[test]
    fn test_cancel() {
        let app = make_test_app();
        let mut window = SaveDialogWindow::from_app(&app);
        let result = window.on_cancel();
        assert_eq!(result, ComponentResult::Close);
        assert!(!window.confirmed);
    }

    #[test]
    fn test_tilde_expansion() {
        let app = make_test_app();
        let mut window = SaveDialogWindow::from_app(&app);
        window.path_input.set("~/logs/export.log");
        let path = window.expanded_path();
        if let Some(home) = dirs::home_dir() {
            assert_eq!(path, format!("{}/logs/export.log", home.display()));
        }
    }

    #[test]
    fn test_raw_export() {
        let app = make_test_app();
        let tmp = std::env::temp_dir().join("scouty_test_raw_save.log");
        let msg = SaveDialogWindow::execute_save(&app, tmp.to_str().unwrap(), ExportFormat::Raw);
        assert!(msg.contains("Saved 2 records"));
        let content = std::fs::read_to_string(&tmp).unwrap();
        assert!(content.contains("hello world"));
        assert!(content.contains("something failed"));
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_json_export() {
        let app = make_test_app();
        let tmp = std::env::temp_dir().join("scouty_test_save.json");
        let msg = SaveDialogWindow::execute_save(&app, tmp.to_str().unwrap(), ExportFormat::Json);
        assert!(msg.contains("Saved 2 records"));
        assert!(msg.contains("json"));
        let content = std::fs::read_to_string(&tmp).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 2);
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_yaml_export() {
        let app = make_test_app();
        let tmp = std::env::temp_dir().join("scouty_test_save.yaml");
        let msg = SaveDialogWindow::execute_save(&app, tmp.to_str().unwrap(), ExportFormat::Yaml);
        assert!(msg.contains("Saved 2 records"));
        assert!(msg.contains("yaml"));
        let content = std::fs::read_to_string(&tmp).unwrap();
        assert!(content.contains("hello world"));
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_tab_switches_focus() {
        let app = make_test_app();
        let mut window = SaveDialogWindow::from_app(&app);

        // Initially path focused
        assert!(!window.enable_jk_navigation());

        // Tab to format
        window.on_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert!(window.enable_jk_navigation());

        // Tab back to path
        window.on_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert!(!window.enable_jk_navigation());
    }
}
