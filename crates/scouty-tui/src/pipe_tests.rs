#[cfg(test)]
mod tests {
    use crate::pipe::*;
    use scouty::record::{LogLevel, LogRecord};

    #[test]
    fn test_output_format_from_str() {
        assert_eq!(OutputFormat::from_str("raw"), Some(OutputFormat::Raw));
        assert_eq!(OutputFormat::from_str("json"), Some(OutputFormat::Json));
        assert_eq!(OutputFormat::from_str("yaml"), Some(OutputFormat::Yaml));
        assert_eq!(OutputFormat::from_str("csv"), Some(OutputFormat::Csv));
        assert_eq!(OutputFormat::from_str("JSON"), Some(OutputFormat::Json));
        assert_eq!(OutputFormat::from_str("unknown"), None);
    }

    #[test]
    fn test_level_passes() {
        assert!(level_passes(LogLevel::Error, LogLevel::Warn));
        assert!(level_passes(LogLevel::Warn, LogLevel::Warn));
        assert!(!level_passes(LogLevel::Info, LogLevel::Warn));
        assert!(level_passes(LogLevel::Fatal, LogLevel::Trace));
    }

    #[test]
    fn test_level_rank_ordering() {
        assert!(level_rank(LogLevel::Trace) < level_rank(LogLevel::Debug));
        assert!(level_rank(LogLevel::Debug) < level_rank(LogLevel::Info));
        assert!(level_rank(LogLevel::Info) < level_rank(LogLevel::Notice));
        assert!(level_rank(LogLevel::Notice) < level_rank(LogLevel::Warn));
        assert!(level_rank(LogLevel::Warn) < level_rank(LogLevel::Error));
        assert!(level_rank(LogLevel::Error) < level_rank(LogLevel::Fatal));
    }

    #[test]
    fn test_record_field_extraction() {
        let record = LogRecord {
            id: 1,
            timestamp: chrono::Utc::now(),
            level: Some(LogLevel::Error),
            source: std::sync::Arc::from("test.log"),
            pid: Some(1234),
            tid: None,
            component_name: Some("myservice".to_string()),
            process_name: None,
            hostname: Some("host01".to_string()),
            container: None,
            context: None,
            function: None,
            message: "test message".to_string(),
            raw: "raw line".to_string(),
            metadata: None,
            loader_id: std::sync::Arc::from("loader"),
            expanded: None,
        };

        assert_eq!(record_field(&record, "message"), "test message");
        assert_eq!(record_field(&record, "hostname"), "host01");
        assert_eq!(record_field(&record, "component"), "myservice");
        assert_eq!(record_field(&record, "pid"), "1234");
        assert_eq!(record_field(&record, "level"), "Error");
        assert_eq!(record_field(&record, "raw"), "raw line");
        assert_eq!(record_field(&record, "tid"), "");
        assert_eq!(record_field(&record, "unknown_field"), "");
    }

    #[test]
    fn test_write_json() {
        let record = LogRecord {
            id: 1,
            timestamp: chrono::DateTime::parse_from_rfc3339("2026-01-15T10:30:00Z")
                .unwrap()
                .with_timezone(&chrono::Utc),
            level: Some(LogLevel::Info),
            source: std::sync::Arc::from("test.log"),
            pid: None,
            tid: None,
            component_name: None,
            process_name: None,
            hostname: None,
            container: None,
            context: None,
            function: None,
            message: "hello".to_string(),
            raw: String::new(),
            metadata: None,
            loader_id: std::sync::Arc::from("loader"),
            expanded: None,
        };

        let mut buf = Vec::new();
        write_json(&mut buf, &record, &[], true).unwrap();
        let output = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(parsed["message"], "hello");
        assert_eq!(parsed["level"], "Info");
    }

    #[test]
    fn test_write_csv_escaping() {
        let record = LogRecord {
            id: 1,
            timestamp: chrono::Utc::now(),
            level: None,
            source: std::sync::Arc::from("test.log"),
            pid: None,
            tid: None,
            component_name: None,
            process_name: None,
            hostname: None,
            container: None,
            context: None,
            function: None,
            message: "hello, world".to_string(),
            raw: String::new(),
            metadata: None,
            loader_id: std::sync::Arc::from("loader"),
            expanded: None,
        };

        let mut buf = Vec::new();
        let fields = vec!["message".to_string()];
        write_csv(&mut buf, &record, &fields, false).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(
            output.contains("\"hello, world\""),
            "CSV should quote fields with commas"
        );
    }

    #[test]
    fn test_default_fields() {
        let fields = default_fields();
        assert!(fields.contains(&"timestamp".to_string()));
        assert!(fields.contains(&"level".to_string()));
        assert!(fields.contains(&"message".to_string()));
    }
}
