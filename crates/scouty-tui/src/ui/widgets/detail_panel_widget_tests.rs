#[cfg(test)]
mod tests {
    use crate::ui::widgets::detail_panel_widget::{
        build_field_lines, DetailPanelWidget, MIN_SPLIT_WIDTH,
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
    fn test_build_field_lines_includes_required_fields() {
        let record = sample_record();
        let lines = build_field_lines(&record);
        let text: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
        assert!(text.iter().any(|l| l.contains("Timestamp:")));
        assert!(text.iter().any(|l| l.contains("Level:")));
        assert!(text.iter().any(|l| l.contains("Source:")));
        assert!(text.iter().any(|l| l.contains("Hostname:")));
        assert!(text.iter().any(|l| l.contains("Component:")));
        assert!(text.iter().any(|l| l.contains("PID:")));
        assert!(text.iter().any(|l| l.contains("Context:")));
        assert!(text.iter().any(|l| l.contains("Function:")));
    }

    #[test]
    fn test_build_field_lines_omits_none_fields() {
        let record = sample_record();
        let lines = build_field_lines(&record);
        let text: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
        // container and tid are None
        assert!(!text.iter().any(|l| l.contains("Container:")));
        assert!(!text.iter().any(|l| l.contains("TID:")));
    }

    #[test]
    fn test_min_split_width_constant() {
        assert_eq!(MIN_SPLIT_WIDTH, 80);
    }
}
