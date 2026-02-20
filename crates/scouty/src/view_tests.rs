#[cfg(test)]
mod tests {
    use crate::filter::engine::{FilterAction, FilterEngine};
    use crate::record::{LogLevel, LogRecord};
    use crate::store::LogStore;
    use crate::view::{LogStoreView, ViewStatus};
    use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
    use std::sync::Arc;

    fn make_record(id: u64, level: LogLevel, message: &str) -> LogRecord {
        let ts = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(0, 0, id as u32 % 60).unwrap(),
        )
        .and_utc();
        LogRecord {
            id,
            timestamp: ts,
            level: Some(level),
            source: "test".into(),
            pid: None,
            tid: None,
            component_name: None,
            process_name: None,
            message: message.to_string(),
            raw: message.to_string(),
            metadata: None,
            loader_id: "loader".into(),
        }
    }

    fn make_store_with_records() -> LogStore {
        let mut store = LogStore::new();
        store.insert(make_record(0, LogLevel::Info, "hello world"));
        store.insert(make_record(1, LogLevel::Error, "something failed"));
        store.insert(make_record(2, LogLevel::Warn, "watch out"));
        store.insert(make_record(3, LogLevel::Info, "all good"));
        store.insert(make_record(4, LogLevel::Debug, "debug trace"));
        store
    }

    #[test]
    fn test_empty_filter_returns_all() {
        let store = make_store_with_records();
        let mut view = LogStoreView::new(FilterEngine::new());
        assert_eq!(view.status(), ViewStatus::Filtering);

        view.apply(&store);

        assert_eq!(view.status(), ViewStatus::Ready);
        assert_eq!(view.len(), 5);
        assert_eq!(view.indices(), &[0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_exclude_filter() {
        let store = make_store_with_records();
        let mut engine = FilterEngine::new();
        engine
            .add_expr_filter(FilterAction::Exclude, r#"level = "ERROR""#)
            .unwrap();

        let mut view = LogStoreView::new(engine);
        view.apply(&store);

        assert_eq!(view.len(), 4);
        // Record at index 1 (Error) should be excluded
        assert!(!view.indices().contains(&1));
    }

    #[test]
    fn test_include_filter() {
        let store = make_store_with_records();
        let mut engine = FilterEngine::new();
        engine
            .add_expr_filter(FilterAction::Include, r#"level = "INFO""#)
            .unwrap();

        let mut view = LogStoreView::new(engine);
        view.apply(&store);

        assert_eq!(view.len(), 2);
        assert_eq!(view.indices(), &[0, 3]);
    }

    #[test]
    fn test_mixed_filters() {
        let store = make_store_with_records();
        let mut engine = FilterEngine::new();
        // Include INFO and WARN
        engine
            .add_expr_filter(FilterAction::Include, r#"level = "INFO""#)
            .unwrap();
        engine
            .add_expr_filter(FilterAction::Include, r#"level = "WARN""#)
            .unwrap();
        // Exclude "all good"
        engine
            .add_expr_filter(FilterAction::Exclude, "message contains \"all good\"")
            .unwrap();

        let mut view = LogStoreView::new(engine);
        view.apply(&store);

        // INFO records: 0 ("hello world"), 3 ("all good" - excluded)
        // WARN records: 2 ("watch out")
        assert_eq!(view.len(), 2);
        assert_eq!(view.indices(), &[0, 2]);
    }

    #[test]
    fn test_get_record() {
        let store = make_store_with_records();
        let mut view = LogStoreView::new(FilterEngine::new());
        view.apply(&store);

        let r = view.get_record(1, &store).unwrap();
        assert_eq!(r.message, "something failed");

        assert!(view.get_record(100, &store).is_none());
    }

    #[test]
    fn test_reapply_after_store_change() {
        let mut store = make_store_with_records();
        let mut view = LogStoreView::new(FilterEngine::new());
        view.apply(&store);
        assert_eq!(view.len(), 5);

        // Add more records
        store.insert(make_record(5, LogLevel::Info, "new record"));
        view.apply(&store);
        assert_eq!(view.len(), 6);
    }

    #[test]
    fn test_is_empty() {
        let store = make_store_with_records();
        let mut engine = FilterEngine::new();
        engine
            .add_expr_filter(FilterAction::Include, r#"level = "FATAL""#)
            .unwrap();

        let mut view = LogStoreView::new(engine);
        view.apply(&store);

        assert!(view.is_empty());
        assert_eq!(view.len(), 0);
    }
}
