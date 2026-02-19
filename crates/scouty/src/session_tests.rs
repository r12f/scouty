//! Tests for LogSession and ParserGroup.

#[cfg(test)]
mod tests {
    use crate::record::{LogLevel, LogRecord};
    use crate::session::{LogSession, ParserGroup};
    use crate::traits::{LoaderInfo, LoaderType, LogLoader, LogParser, Result};
    use chrono::Utc;
    use std::collections::HashMap;

    fn make_record(id: u64, message: &str) -> LogRecord {
        LogRecord {
            id,
            timestamp: Utc::now(),
            level: Some(LogLevel::Info),
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
    struct AlwaysFail;
    impl LogParser for AlwaysFail {
        fn parse(&self, _raw: &str, _source: &str, _loader_id: &str, _id: u64) -> Option<LogRecord> {
            None
        }
        fn name(&self) -> &str { "always-fail" }
    }

    #[derive(Debug)]
    struct AlwaysSucceed;
    impl LogParser for AlwaysSucceed {
        fn parse(&self, raw: &str, _source: &str, _loader_id: &str, id: u64) -> Option<LogRecord> {
            Some(make_record(id, raw))
        }
        fn name(&self) -> &str { "always-succeed" }
    }

    #[test]
    fn parser_group_fallback() {
        let mut group = ParserGroup::new("test-group");
        group.add_parser(Box::new(AlwaysFail));
        group.add_parser(Box::new(AlwaysSucceed));

        let result = group.parse("hello", "src", "loader", 0);
        assert!(result.is_some());
        assert_eq!(result.unwrap().message, "hello");
    }

    #[test]
    fn parser_group_all_fail() {
        let mut group = ParserGroup::new("fail-group");
        group.add_parser(Box::new(AlwaysFail));
        assert!(group.parse("hello", "src", "loader", 0).is_none());
    }

    #[test]
    fn session_new_is_empty() {
        let session = LogSession::new();
        assert!(session.store().is_empty());
        assert!(session.failing_parsing_logs.is_empty());
    }

    #[derive(Debug)]
    struct MockLoader {
        info: LoaderInfo,
        lines: Vec<String>,
    }
    impl LogLoader for MockLoader {
        fn info(&self) -> &LoaderInfo { &self.info }
        fn load(&mut self) -> Result<Vec<String>> { Ok(self.lines.clone()) }
    }

    #[test]
    fn session_run_end_to_end() {
        let mut session = LogSession::new();

        let loader = MockLoader {
            info: LoaderInfo {
                id: "mock".into(),
                loader_type: LoaderType::TextFile,
                multiline_enabled: false,
                sample_lines: vec![],
            },
            lines: vec!["line1".into(), "line2".into(), "bad-line".into()],
        };

        let mut group = ParserGroup::new("test");
        // Only parse lines starting with "line"
        #[derive(Debug)]
        struct LineParser;
        impl LogParser for LineParser {
            fn parse(&self, raw: &str, _source: &str, _loader_id: &str, id: u64) -> Option<LogRecord> {
                if raw.starts_with("line") {
                    Some(make_record(id, raw))
                } else {
                    None
                }
            }
            fn name(&self) -> &str { "line-parser" }
        }
        group.add_parser(Box::new(LineParser));

        session.add_loader(Box::new(loader), group);
        let filtered = session.run().unwrap();

        assert_eq!(session.store().len(), 2);
        assert_eq!(session.failing_parsing_logs.len(), 1);
        assert_eq!(session.failing_parsing_logs[0].raw, "bad-line");
        assert_eq!(filtered, vec![0, 1]);
    }
}
