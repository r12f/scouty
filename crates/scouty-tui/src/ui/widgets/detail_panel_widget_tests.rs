#[cfg(test)]
mod tests {
    use crate::ui::widgets::detail_panel_widget::{
        build_field_lines, build_field_pairs, DetailPanelWidget, MIN_SPLIT_WIDTH,
    };
    use crate::ui::{dispatch_key, ComponentResult, UiComponent};
    use chrono::Utc;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use scouty::record::{LogLevel, LogRecord};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    fn sample_record() -> LogRecord {
        LogRecord {
            id: 1,
            timestamp: Utc::now(),
            level: Some(LogLevel::Info),
            source: "test.log".into(),
            pid: Some(1234),
            tid: None,
            component_name: Some("orchagent".into()),
            process_name: Some("orchagent".into()),
            message: "hello world".into(),
            hostname: Some("switch01".into()),
            container: None,
            context: Some("oid:0x1234".into()),
            function: Some("doTask".into()),
            raw: "2025-05-18 INFO orchagent hello world".into(),
            metadata: None,
            loader_id: "test".into(),
        }
    }

    #[test]
    fn test_enable_jk_navigation() {
        let widget = DetailPanelWidget;
        assert!(!widget.enable_jk_navigation());
    }

    #[test]
    fn test_esc_closes() {
        let mut widget = DetailPanelWidget;
        assert_eq!(
            dispatch_key(&mut widget, key(KeyCode::Esc)),
            ComponentResult::Close
        );
    }

    #[test]
    fn test_navigation_ignored() {
        let mut widget = DetailPanelWidget;
        assert_eq!(
            dispatch_key(&mut widget, key(KeyCode::Up)),
            ComponentResult::Ignored
        );
        assert_eq!(
            dispatch_key(&mut widget, key(KeyCode::Down)),
            ComponentResult::Ignored
        );
    }

    #[test]
    fn test_build_field_pairs_includes_required_fields() {
        let record = sample_record();
        let pairs = build_field_pairs(&record);
        let keys: Vec<&str> = pairs.iter().map(|(k, _)| *k).collect();
        assert!(keys.contains(&"Timestamp"));
        assert!(keys.contains(&"Level"));
        assert!(keys.contains(&"Source"));
        assert!(keys.contains(&"Hostname"));
        assert!(keys.contains(&"Component"));
        assert!(keys.contains(&"PID"));
        assert!(keys.contains(&"Context"));
        assert!(keys.contains(&"Function"));
    }

    #[test]
    fn test_build_field_pairs_omits_none_fields() {
        let record = sample_record();
        let pairs = build_field_pairs(&record);
        let keys: Vec<&str> = pairs.iter().map(|(k, _)| *k).collect();
        // container and tid are None
        assert!(!keys.contains(&"Container"));
        assert!(!keys.contains(&"TID"));
    }

    #[test]
    fn test_build_field_lines_from_pairs() {
        let record = sample_record();
        let theme = crate::config::Theme::default();
        let lines = build_field_lines(&record, &theme);
        let text: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
        assert!(text.iter().any(|l| l.contains("Timestamp:")));
        assert!(text.iter().any(|l| l.contains("Level:")));
    }

    #[test]
    fn test_min_split_width_constant() {
        assert_eq!(MIN_SPLIT_WIDTH, 80);
    }
}
