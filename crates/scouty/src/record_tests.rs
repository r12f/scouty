//! Tests for LogRecord and LogLevel.

#[cfg(test)]
mod tests {
    use crate::record::{LogLevel, LogRecord};
    use chrono::Utc;
    use std::collections::HashMap;

    fn make_record(id: u64, level: LogLevel, message: &str) -> LogRecord {
        LogRecord {
            id,
            timestamp: Utc::now(),
            level: Some(level),
            source: "test".into(),
            pid: None,
            tid: None,
            component_name: None,
            process_name: None,
            hostname: None,
            container: None,
            context: None,
            function: None,
            message: message.into(),
            raw: message.into(),
            metadata: None,
            loader_id: "test-loader".into(),
            expanded: None,
        }
    }

    #[test]
    fn log_level_display() {
        assert_eq!(LogLevel::Trace.to_string(), "TRACE");
        assert_eq!(LogLevel::Debug.to_string(), "DEBUG");
        assert_eq!(LogLevel::Info.to_string(), "INFO");
        assert_eq!(LogLevel::Warn.to_string(), "WARN");
        assert_eq!(LogLevel::Error.to_string(), "ERROR");
        assert_eq!(LogLevel::Fatal.to_string(), "FATAL");
    }

    #[test]
    fn log_level_from_str_loose() {
        assert_eq!(LogLevel::from_str_loose("warn"), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_str_loose("WARNING"), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_str_loose("error"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str_loose("CRITICAL"), Some(LogLevel::Fatal));
        assert_eq!(LogLevel::from_str_loose("unknown"), None);
        assert_eq!(LogLevel::from_str_loose(""), None);
    }

    #[test]
    fn log_level_ordering() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
        assert!(LogLevel::Error < LogLevel::Fatal);
    }

    #[test]
    fn log_record_fields() {
        let r = make_record(42, LogLevel::Error, "something broke");
        assert_eq!(r.id, 42);
        assert_eq!(r.level, Some(LogLevel::Error));
        assert_eq!(r.message, "something broke");
        assert_eq!(&*r.loader_id, "test-loader");
        assert!(r.metadata.is_none());
    }
}
