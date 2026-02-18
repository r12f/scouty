//! Basic integration tests for Phase 1: core types, session, store, filter engine.

use chrono::Utc;
use scouty::filter::engine::{FilterAction, FilterEngine};
use scouty::record::{LogLevel, LogRecord};
use scouty::session::{LogSession, ParserGroup};
use scouty::store::LogStore;
use scouty::traits::LogFilter;
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

#[test]
fn test_log_level_display_and_parse() {
    assert_eq!(LogLevel::Error.to_string(), "ERROR");
    assert_eq!(LogLevel::from_str_loose("warn"), Some(LogLevel::Warn));
    assert_eq!(LogLevel::from_str_loose("WARNING"), Some(LogLevel::Warn));
    assert_eq!(LogLevel::from_str_loose("unknown"), None);
}

#[test]
fn test_store_insert_ordered() {
    let mut store = LogStore::new();
    let r1 = make_record(1, LogLevel::Info, "first");
    let r2 = make_record(2, LogLevel::Error, "second");
    store.insert(r1);
    store.insert(r2);
    assert_eq!(store.len(), 2);
    // Both inserted at ~same timestamp, order preserved
    assert_eq!(store.records()[0].id, 1);
}

#[test]
fn test_filter_engine_exclude_first() {
    let mut engine = FilterEngine::new();

    // Exclude errors
    #[derive(Debug)]
    struct ExcludeErrors;
    impl LogFilter for ExcludeErrors {
        fn matches(&self, record: &LogRecord) -> bool {
            record.level == Some(LogLevel::Error)
        }
        fn description(&self) -> &str {
            "exclude errors"
        }
    }

    engine.add_filter(FilterAction::Exclude, Box::new(ExcludeErrors));

    let records = vec![
        make_record(0, LogLevel::Info, "ok"),
        make_record(1, LogLevel::Error, "bad"),
        make_record(2, LogLevel::Warn, "meh"),
    ];

    let filtered = engine.apply(&records);
    assert_eq!(filtered, vec![0, 2]);
}

#[test]
fn test_filter_engine_include_only() {
    let mut engine = FilterEngine::new();

    #[derive(Debug)]
    struct IncludeWarn;
    impl LogFilter for IncludeWarn {
        fn matches(&self, record: &LogRecord) -> bool {
            record.level == Some(LogLevel::Warn)
        }
        fn description(&self) -> &str {
            "include warn"
        }
    }

    engine.add_filter(FilterAction::Include, Box::new(IncludeWarn));

    let records = vec![
        make_record(0, LogLevel::Info, "ok"),
        make_record(1, LogLevel::Warn, "attention"),
        make_record(2, LogLevel::Error, "bad"),
    ];

    let filtered = engine.apply(&records);
    assert_eq!(filtered, vec![1]);
}

#[test]
fn test_session_creation() {
    let session = LogSession::new();
    assert!(session.store().is_empty());
    assert!(session.failing_parsing_logs.is_empty());
}

#[test]
fn test_parser_group_fallback() {
    use scouty::traits::LogParser;

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
        fn parse(&self, raw: &str, source: &str, loader_id: &str, id: u64) -> Option<LogRecord> {
            Some(make_record(id, LogLevel::Info, raw))
        }
        fn name(&self) -> &str { "always-succeed" }
    }

    let mut group = ParserGroup::new("test-group");
    group.add_parser(Box::new(AlwaysFail));
    group.add_parser(Box::new(AlwaysSucceed));

    let result = group.parse("hello", "src", "loader", 0);
    assert!(result.is_some());
    assert_eq!(result.unwrap().message, "hello");

    // All-fail group
    let mut group2 = ParserGroup::new("fail-group");
    group2.add_parser(Box::new(AlwaysFail));
    assert!(group2.parse("hello", "src", "loader", 0).is_none());
}
