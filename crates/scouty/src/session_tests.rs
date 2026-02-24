//! Tests for LogSession and ParserGroup.

#[cfg(test)]
mod tests {
    use crate::parser::group::ParserGroup;
    use crate::record::{LogLevel, LogRecord};
    use crate::session::LogSession;
    use crate::traits::{LoaderInfo, LoaderType, LogLoader, LogParser, Result};
    use chrono::Utc;
    use std::collections::HashMap;

    fn make_record(id: u64, message: &str) -> LogRecord {
        // Use a fixed timestamp to avoid non-deterministic ordering when
        // records are created across parallel threads with Utc::now().
        let ts = chrono::DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        LogRecord {
            id,
            timestamp: ts,
            level: Some(LogLevel::Info),
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

    #[derive(Debug)]
    struct AlwaysFail;
    impl LogParser for AlwaysFail {
        fn parse(
            &self,
            _raw: &str,
            _source: &str,
            _loader_id: &str,
            _id: u64,
        ) -> Option<LogRecord> {
            None
        }
        fn name(&self) -> &str {
            "always-fail"
        }
    }

    #[derive(Debug)]
    struct AlwaysSucceed;
    impl LogParser for AlwaysSucceed {
        fn parse(&self, raw: &str, _source: &str, _loader_id: &str, id: u64) -> Option<LogRecord> {
            Some(make_record(id, raw))
        }
        fn name(&self) -> &str {
            "always-succeed"
        }
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
        fn info(&self) -> &LoaderInfo {
            &self.info
        }
        fn load(&mut self) -> Result<Vec<String>> {
            Ok(self.lines.clone())
        }
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
                file_mod_year: None,
            },
            lines: vec!["line1".into(), "line2".into(), "bad-line".into()],
        };

        let mut group = ParserGroup::new("test");
        // Only parse lines starting with "line"
        #[derive(Debug)]
        struct LineParser;
        impl LogParser for LineParser {
            fn parse(
                &self,
                raw: &str,
                _source: &str,
                _loader_id: &str,
                id: u64,
            ) -> Option<LogRecord> {
                if raw.starts_with("line") {
                    Some(make_record(id, raw))
                } else {
                    None
                }
            }
            fn name(&self) -> &str {
                "line-parser"
            }
        }
        group.add_parser(Box::new(LineParser));

        session.add_loader(Box::new(loader), group);
        let filtered = session.run().unwrap();

        assert_eq!(session.store().len(), 2);
        assert_eq!(session.failing_parsing_logs.len(), 1);
        assert_eq!(session.failing_parsing_logs[0].raw, "bad-line");
        assert_eq!(filtered, vec![0, 1]);
    }

    #[test]
    fn session_run_parallel_end_to_end() {
        let mut session = LogSession::new();

        // Create two loaders
        for i in 0..2 {
            let loader = MockLoader {
                info: LoaderInfo {
                    id: format!("mock-{}", i),
                    loader_type: LoaderType::TextFile,
                    multiline_enabled: false,
                    sample_lines: vec![],
                    file_mod_year: None,
                },
                lines: vec![
                    format!("line-{}-a", i),
                    format!("line-{}-b", i),
                    format!("bad-{}", i),
                ],
            };

            let mut group = ParserGroup::new(format!("group-{}", i));
            #[derive(Debug)]
            struct LineParser;
            impl LogParser for LineParser {
                fn parse(
                    &self,
                    raw: &str,
                    _source: &str,
                    _loader_id: &str,
                    id: u64,
                ) -> Option<LogRecord> {
                    if raw.starts_with("line") {
                        Some(make_record(id, raw))
                    } else {
                        None
                    }
                }
                fn name(&self) -> &str {
                    "line-parser"
                }
            }
            group.add_parser(Box::new(LineParser));
            session.add_loader(Box::new(loader), group);
        }

        let filtered = session.run_parallel().unwrap();

        // 2 loaders × 2 good lines = 4 records
        assert_eq!(session.store().len(), 4);
        // 2 loaders × 1 bad line = 2 failures
        assert_eq!(session.failing_parsing_logs.len(), 2);
        assert_eq!(filtered.len(), 4);
    }

    #[test]
    fn session_parallel_matches_sequential() {
        // Verify parallel and sequential produce same counts
        let make_session = || {
            let mut session = LogSession::new();
            for i in 0..3 {
                let loader = MockLoader {
                    info: LoaderInfo {
                        id: format!("loader-{}", i),
                        loader_type: LoaderType::TextFile,
                        multiline_enabled: false,
                        sample_lines: vec![],
                        file_mod_year: None,
                    },
                    lines: (0..10).map(|j| format!("line-{}", j)).collect(),
                };
                let mut group = ParserGroup::new(format!("group-{}", i));
                group.add_parser(Box::new(AlwaysSucceed));
                session.add_loader(Box::new(loader), group);
            }
            session
        };

        let mut seq = make_session();
        let seq_filtered = seq.run().unwrap();

        let mut par = make_session();
        let par_filtered = par.run_parallel().unwrap();

        assert_eq!(seq.store().len(), par.store().len());
        assert_eq!(seq_filtered.len(), par_filtered.len());
    }

    fn make_mock_loader(id: &str, lines: Vec<String>) -> MockLoader {
        MockLoader {
            info: LoaderInfo {
                id: id.into(),
                loader_type: LoaderType::TextFile,
                multiline_enabled: false,
                sample_lines: vec![],
                file_mod_year: None,
            },
            lines,
        }
    }

    /// Helper: AlwaysSucceed parser that puts raw as message
    #[derive(Debug)]
    struct AllParser;
    impl LogParser for AllParser {
        fn parse(&self, raw: &str, _source: &str, _loader_id: &str, id: u64) -> Option<LogRecord> {
            Some(make_record(id, raw))
        }
        fn name(&self) -> &str {
            "all"
        }
    }

    #[test]
    fn test_dual_view_active_not_affected_by_pending() {
        use crate::filter::engine::{FilterAction, FilterEngine};

        let mut session = LogSession::new();
        let mut group = ParserGroup::new("test");
        group.add_parser(Box::new(AllParser));

        session.add_loader(
            Box::new(make_mock_loader(
                "l1",
                vec!["msg1".into(), "msg2".into(), "msg3".into()],
            )),
            group,
        );
        session.run().unwrap();

        // Active view has all 3 records
        assert_eq!(session.active_view().len(), 3);

        // Create pending view with a filter
        let mut new_engine = FilterEngine::new();
        new_engine
            .add_expr_filter(FilterAction::Include, r#"message contains "msg1""#)
            .unwrap();
        session.update_filter(new_engine);

        // Active view still has 3 records (not affected by pending)
        assert_eq!(session.active_view().len(), 3);
        assert!(session.has_pending_view());

        // Apply pending → active now has 1 record
        session.apply_pending();
        assert_eq!(session.active_view().len(), 1);
        assert!(!session.has_pending_view());
    }

    #[test]
    fn test_dual_view_pending_replaced_on_new_filter() {
        use crate::filter::engine::{FilterAction, FilterEngine};

        let mut session = LogSession::new();
        let mut group = ParserGroup::new("test");
        group.add_parser(Box::new(AllParser));

        session.add_loader(
            Box::new(make_mock_loader("l1", vec!["msg1".into(), "msg2".into()])),
            group,
        );
        session.run().unwrap();

        // Create first pending
        let mut engine1 = FilterEngine::new();
        engine1
            .add_expr_filter(FilterAction::Include, r#"message contains "msg1""#)
            .unwrap();
        session.update_filter(engine1);

        // Create second pending — should discard first
        let mut engine2 = FilterEngine::new();
        engine2
            .add_expr_filter(FilterAction::Include, r#"message contains "msg2""#)
            .unwrap();
        session.update_filter(engine2);

        // Apply — should apply engine2 (msg2 only)
        session.apply_pending();
        assert_eq!(session.active_view().len(), 1);
        let record = session
            .active_view()
            .get_record(0, session.store())
            .unwrap();
        assert_eq!(record.message, "msg2");
    }

    #[test]
    fn test_refresh_active_view() {
        use crate::filter::engine::FilterAction;

        let mut session = LogSession::new();
        let mut group = ParserGroup::new("test");
        group.add_parser(Box::new(AllParser));

        session.add_loader(
            Box::new(make_mock_loader(
                "l1",
                vec!["msg1".into(), "msg2".into(), "msg3".into()],
            )),
            group,
        );
        session.run().unwrap();
        assert_eq!(session.filtered_view().len(), 3);

        // Modify active filter directly
        session
            .filter_engine_mut()
            .add_expr_filter(FilterAction::Include, r#"message contains "msg1""#)
            .unwrap();

        // filtered_view() still returns cached (3 records)
        assert_eq!(session.filtered_view().len(), 3);

        // After refresh, it's updated
        session.refresh_active_view();
        assert_eq!(session.filtered_view().len(), 1);
    }

    #[test]
    fn test_async_filter_basic() {
        use crate::filter::engine::{FilterAction, FilterEngine};

        let mut session = LogSession::new();
        let mut group = ParserGroup::new("test");
        group.add_parser(Box::new(AllParser));

        session.add_loader(
            Box::new(make_mock_loader(
                "l1",
                vec!["msg1".into(), "msg2".into(), "msg3".into()],
            )),
            group,
        );
        session.run().unwrap();
        assert_eq!(session.active_view().len(), 3);

        // Start async filter
        let mut engine = FilterEngine::new();
        engine
            .add_expr_filter(FilterAction::Include, r#"message contains "msg1""#)
            .unwrap();
        session.update_filter_async(engine);
        assert!(session.is_filtering());

        // Poll until done
        while !session.poll_pending() {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        assert!(!session.is_filtering());
        assert_eq!(session.active_view().len(), 1);
        let record = session
            .active_view()
            .get_record(0, session.store())
            .unwrap();
        assert_eq!(record.message, "msg1");
    }

    #[test]
    fn test_async_filter_cancel() {
        use crate::filter::engine::{FilterAction, FilterEngine};

        let mut session = LogSession::new();
        let mut group = ParserGroup::new("test");
        group.add_parser(Box::new(AllParser));

        session.add_loader(
            Box::new(make_mock_loader(
                "l1",
                vec!["msg1".into(), "msg2".into(), "msg3".into()],
            )),
            group,
        );
        session.run().unwrap();

        // Start first async filter (include msg1)
        let mut engine1 = FilterEngine::new();
        engine1
            .add_expr_filter(FilterAction::Include, r#"message contains "msg1""#)
            .unwrap();
        session.update_filter_async(engine1);

        // Cancel by starting a new async filter (include msg2)
        let mut engine2 = FilterEngine::new();
        engine2
            .add_expr_filter(FilterAction::Include, r#"message contains "msg2""#)
            .unwrap();
        session.update_filter_async(engine2);

        // Poll until done — should get msg2 (second filter)
        while !session.poll_pending() {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        assert_eq!(session.active_view().len(), 1);
        let record = session
            .active_view()
            .get_record(0, session.store())
            .unwrap();
        assert_eq!(record.message, "msg2");
    }

    #[test]
    fn test_async_filter_does_not_block_active_view() {
        use crate::filter::engine::{FilterAction, FilterEngine};

        let mut session = LogSession::new();
        let mut group = ParserGroup::new("test");
        group.add_parser(Box::new(AllParser));

        session.add_loader(
            Box::new(make_mock_loader(
                "l1",
                vec!["msg1".into(), "msg2".into(), "msg3".into()],
            )),
            group,
        );
        session.run().unwrap();
        assert_eq!(session.active_view().len(), 3);

        // Start async filter
        let mut engine = FilterEngine::new();
        engine
            .add_expr_filter(FilterAction::Include, r#"message contains "msg1""#)
            .unwrap();
        session.update_filter_async(engine);

        // Active view still shows all 3 while filtering
        assert_eq!(session.active_view().len(), 3);
        assert!(session.is_filtering());

        // Wait for completion
        while !session.poll_pending() {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // Now active view is updated
        assert_eq!(session.active_view().len(), 1);
    }

    #[test]
    fn test_async_cancels_sync_pending() {
        use crate::filter::engine::{FilterAction, FilterEngine};

        let mut session = LogSession::new();
        let mut group = ParserGroup::new("test");
        group.add_parser(Box::new(AllParser));

        session.add_loader(
            Box::new(make_mock_loader("l1", vec!["msg1".into(), "msg2".into()])),
            group,
        );
        session.run().unwrap();

        // Create a sync pending
        let mut engine1 = FilterEngine::new();
        engine1
            .add_expr_filter(FilterAction::Include, r#"message contains "msg1""#)
            .unwrap();
        session.update_filter(engine1);
        assert!(session.has_pending_view());

        // Async replaces it
        let mut engine2 = FilterEngine::new();
        engine2
            .add_expr_filter(FilterAction::Include, r#"message contains "msg2""#)
            .unwrap();
        session.update_filter_async(engine2);

        // Sync apply_pending should be no-op now
        session.apply_pending();
        assert_eq!(session.active_view().len(), 2); // unchanged

        // Async finishes
        while !session.poll_pending() {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert_eq!(session.active_view().len(), 1);
        let record = session
            .active_view()
            .get_record(0, session.store())
            .unwrap();
        assert_eq!(record.message, "msg2");
    }
}
