//! Tests for FilterEngine.

#[cfg(test)]
mod tests {
    use crate::filter::engine::{FilterAction, FilterEngine};
    use crate::record::{LogLevel, LogRecord};
    use crate::traits::LogFilter;
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
            message: message.into(),
            raw: message.into(),
            metadata: HashMap::new(),
            loader_id: "test-loader".into(),
        }
    }

    #[derive(Debug)]
    struct LevelFilter(LogLevel);
    impl LogFilter for LevelFilter {
        fn matches(&self, record: &LogRecord) -> bool {
            record.level == Some(self.0)
        }
        fn description(&self) -> &str {
            "level filter"
        }
    }

    #[test]
    fn no_filters_includes_all() {
        let engine = FilterEngine::new();
        let records = vec![
            make_record(0, LogLevel::Info, "a"),
            make_record(1, LogLevel::Error, "b"),
        ];
        assert_eq!(engine.apply(&records), vec![0, 1]);
    }

    #[test]
    fn exclude_removes_matching() {
        let mut engine = FilterEngine::new();
        engine.add_filter(
            FilterAction::Exclude,
            Box::new(LevelFilter(LogLevel::Error)),
        );

        let records = vec![
            make_record(0, LogLevel::Info, "ok"),
            make_record(1, LogLevel::Error, "bad"),
            make_record(2, LogLevel::Warn, "meh"),
        ];
        assert_eq!(engine.apply(&records), vec![0, 2]);
    }

    #[test]
    fn include_only_keeps_matching() {
        let mut engine = FilterEngine::new();
        engine.add_filter(FilterAction::Include, Box::new(LevelFilter(LogLevel::Warn)));

        let records = vec![
            make_record(0, LogLevel::Info, "ok"),
            make_record(1, LogLevel::Warn, "attention"),
            make_record(2, LogLevel::Error, "bad"),
        ];
        assert_eq!(engine.apply(&records), vec![1]);
    }

    #[test]
    fn exclude_takes_priority_over_include() {
        let mut engine = FilterEngine::new();
        engine.add_filter(
            FilterAction::Exclude,
            Box::new(LevelFilter(LogLevel::Error)),
        );
        engine.add_filter(
            FilterAction::Include,
            Box::new(LevelFilter(LogLevel::Error)),
        );

        let records = vec![
            make_record(0, LogLevel::Info, "ok"),
            make_record(1, LogLevel::Error, "bad"),
        ];
        // Error is excluded first, then include has no effect on it
        assert_eq!(engine.apply(&records), Vec::<usize>::new());
    }

    #[test]
    fn clear_removes_all_filters() {
        let mut engine = FilterEngine::new();
        engine.add_filter(FilterAction::Exclude, Box::new(LevelFilter(LogLevel::Info)));
        engine.clear();

        let records = vec![make_record(0, LogLevel::Info, "ok")];
        assert_eq!(engine.apply(&records), vec![0]);
    }

    // === Expression-based filter tests ===

    #[test]
    fn expr_filter_exclude() {
        let mut engine = FilterEngine::new();
        engine
            .add_expr_filter(FilterAction::Exclude, r#"level = "ERROR""#)
            .unwrap();

        let records = vec![
            make_record(0, LogLevel::Info, "ok"),
            make_record(1, LogLevel::Error, "bad"),
            make_record(2, LogLevel::Warn, "meh"),
        ];
        assert_eq!(engine.apply(&records), vec![0, 2]);
    }

    #[test]
    fn expr_filter_include() {
        let mut engine = FilterEngine::new();
        engine
            .add_expr_filter(FilterAction::Include, r#"level = "WARN""#)
            .unwrap();

        let records = vec![
            make_record(0, LogLevel::Info, "ok"),
            make_record(1, LogLevel::Warn, "attention"),
        ];
        assert_eq!(engine.apply(&records), vec![1]);
    }

    #[test]
    fn expr_filter_complex() {
        let mut engine = FilterEngine::new();
        engine
            .add_expr_filter(
                FilterAction::Include,
                r#"level = "ERROR" OR level = "FATAL""#,
            )
            .unwrap();
        engine
            .add_expr_filter(FilterAction::Exclude, r#"message contains "ignore""#)
            .unwrap();

        let records = vec![
            make_record(0, LogLevel::Info, "ok"),
            make_record(1, LogLevel::Error, "real error"),
            make_record(2, LogLevel::Error, "ignore this error"),
            make_record(3, LogLevel::Fatal, "crash"),
        ];
        // Include: Error or Fatal → [1, 2, 3]; Exclude "ignore" → [1, 3]
        assert_eq!(engine.apply(&records), vec![1, 3]);
    }

    #[test]
    fn expr_filter_invalid_expression() {
        let mut engine = FilterEngine::new();
        let result = engine.add_expr_filter(FilterAction::Include, r#"level = "#);
        assert!(result.is_err());
    }

    #[test]
    fn expr_filter_invalid_regex() {
        let mut engine = FilterEngine::new();
        let result = engine.add_expr_filter(FilterAction::Include, r#"message regex "[bad""#);
        assert!(result.is_err());
    }
}
