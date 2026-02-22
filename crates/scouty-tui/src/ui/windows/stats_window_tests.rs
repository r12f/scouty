//! Tests for the statistics overlay window.

mod tests {
    use crate::app::{App, InputMode};
    use crate::text_input::TextInput;
    use crate::ui::windows::stats_window::{StatsData, StatsWindow};
    use crate::ui::{dispatch_key, ComponentResult};
    use chrono::Utc;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use scouty::record::{LogLevel, LogRecord};
    use std::sync::Arc;

    fn make_record(id: u64, level: Option<LogLevel>, component: Option<&str>) -> LogRecord {
        LogRecord {
            id,
            timestamp: Utc::now() + chrono::Duration::seconds(id as i64),
            level,
            source: "test".into(),
            pid: None,
            tid: None,
            component_name: component.map(|s| s.to_string()),
            process_name: None,
            message: format!("msg {}", id),
            hostname: None,
            container: None,
            context: None,
            function: None,
            raw: format!("msg {}", id),
            metadata: None,
            loader_id: "test".into(),
        }
    }

    fn make_test_app() -> App {
        let records: Vec<Arc<LogRecord>> = vec![
            Arc::new(make_record(0, Some(LogLevel::Info), Some("auth"))),
            Arc::new(make_record(1, Some(LogLevel::Error), Some("auth"))),
            Arc::new(make_record(2, Some(LogLevel::Info), Some("db"))),
            Arc::new(make_record(3, Some(LogLevel::Warn), Some("db"))),
            Arc::new(make_record(4, Some(LogLevel::Error), Some("api"))),
            Arc::new(make_record(5, Some(LogLevel::Info), None)),
        ];
        let n = records.len();
        App {
            records,
            total_records: n,
            filtered_indices: (0..n).collect(),
            scroll_offset: 0,
            selected: 0,
            visible_rows: 10,
            detail_open: false,
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
            status_message_at: None,
            col_widths: [19, 5, 11, 3, 3, 9],
            column_config: crate::app::ColumnConfig::default(),
            follow_mode: false,
            copy_format_cursor: 0,
            help_scroll: 0,
            command_input: TextInput::new(),
            should_quit: false,
            filter_version: 0,
            density_cache: None,
            highlight_input: TextInput::new(),
            highlight_manager_cursor: 0,
            highlight_rules: Vec::new(),
            cached_stats: None,
            bookmarks: std::collections::HashSet::new(),
            bookmark_manager_cursor: 0,
            theme: crate::config::Theme::default(),
        }
    }

    #[test]
    fn test_stats_compute_total_records() {
        let app = make_test_app();
        let stats = StatsData::compute(&app);
        assert_eq!(stats.total_records, 6);
        assert_eq!(stats.filtered_records, 6);
    }

    #[test]
    fn test_stats_compute_time_range() {
        let app = make_test_app();
        let stats = StatsData::compute(&app);
        assert!(stats.time_first.is_some());
        assert!(stats.time_last.is_some());
        assert_ne!(stats.time_first, stats.time_last);
    }

    #[test]
    fn test_stats_compute_level_distribution() {
        let app = make_test_app();
        let stats = StatsData::compute(&app);
        // 3 Info, 2 Error, 1 Warn
        let find = |l: LogLevel| {
            stats
                .level_counts
                .iter()
                .find(|(lv, _)| *lv == l)
                .map(|(_, c)| *c)
        };
        assert_eq!(find(LogLevel::Info), Some(3));
        assert_eq!(find(LogLevel::Error), Some(2));
        assert_eq!(find(LogLevel::Warn), Some(1));
    }

    #[test]
    fn test_stats_compute_top_components() {
        let app = make_test_app();
        let stats = StatsData::compute(&app);
        // auth=2, db=2, api=1
        assert_eq!(stats.top_components.len(), 3);
        // Top should be auth or db (both 2)
        assert!(stats.top_components[0].1 >= stats.top_components[1].1);
    }

    #[test]
    fn test_stats_empty_data() {
        let app = App {
            records: vec![],
            total_records: 0,
            filtered_indices: vec![],
            scroll_offset: 0,
            selected: 0,
            visible_rows: 10,
            detail_open: false,
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
            status_message_at: None,
            col_widths: [19, 5, 11, 3, 3, 9],
            column_config: crate::app::ColumnConfig::default(),
            follow_mode: false,
            copy_format_cursor: 0,
            help_scroll: 0,
            command_input: TextInput::new(),
            should_quit: false,
            filter_version: 0,
            density_cache: None,
            highlight_input: TextInput::new(),
            highlight_manager_cursor: 0,
            highlight_rules: Vec::new(),
            cached_stats: None,
            bookmarks: std::collections::HashSet::new(),
            bookmark_manager_cursor: 0,
            theme: crate::config::Theme::default(),
        };
        let stats = StatsData::compute(&app);
        assert_eq!(stats.total_records, 0);
        assert_eq!(stats.filtered_records, 0);
        assert!(stats.time_first.is_none());
        assert!(stats.level_counts.is_empty());
        assert!(stats.top_components.is_empty());
    }

    #[test]
    fn test_stats_window_closes_on_esc() {
        let mut app = make_test_app();
        app.cached_stats = Some(StatsData::compute(&app));
        let theme = crate::config::Theme::default();
        let mut window = StatsWindow {
            stats: app.cached_stats.as_ref().unwrap(),
            theme: &theme,
        };
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let result = dispatch_key(&mut window, key);
        assert_eq!(result, ComponentResult::Close);
    }

    #[test]
    fn test_stats_window_closes_on_any_char() {
        let mut app = make_test_app();
        app.cached_stats = Some(StatsData::compute(&app));
        let theme = crate::config::Theme::default();
        let mut window = StatsWindow {
            stats: app.cached_stats.as_ref().unwrap(),
            theme: &theme,
        };
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let result = dispatch_key(&mut window, key);
        assert_eq!(result, ComponentResult::Close);
    }
}
