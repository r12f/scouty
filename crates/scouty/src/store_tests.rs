#[cfg(test)]
mod tests {
    use crate::record::{LogLevel, LogRecord};
    use crate::store::LogStore;
    use chrono::{Duration, Utc};
    use std::collections::HashMap;

    fn make_record_at(id: u64, level: LogLevel, message: &str, offset_secs: i64) -> LogRecord {
        LogRecord {
            id,
            timestamp: Utc::now() + Duration::seconds(offset_secs),
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
    fn insert_maintains_order() {
        let mut store = LogStore::new();
        store.insert(make_record_at(2, LogLevel::Info, "second", 10));
        store.insert(make_record_at(1, LogLevel::Info, "first", 0));
        store.insert(make_record_at(3, LogLevel::Info, "third", 20));

        let records = store.records();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].message, "first");
        assert_eq!(records[1].message, "second");
        assert_eq!(records[2].message, "third");
    }

    #[test]
    fn insert_batch_sorts() {
        let mut store = LogStore::new();
        let batch = vec![
            make_record_at(3, LogLevel::Warn, "c", 20),
            make_record_at(1, LogLevel::Info, "a", 0),
            make_record_at(2, LogLevel::Error, "b", 10),
        ];
        store.insert_batch(batch);

        assert_eq!(store.len(), 3);
        assert_eq!(store.records()[0].message, "a");
        assert_eq!(store.records()[1].message, "b");
        assert_eq!(store.records()[2].message, "c");
    }

    #[test]
    fn empty_store() {
        let store = LogStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn with_capacity() {
        let store = LogStore::with_capacity(1000);
        assert!(store.is_empty());
    }

    #[test]
    fn get_by_index() {
        let mut store = LogStore::new();
        store.insert(make_record_at(1, LogLevel::Info, "first", 0));
        store.insert(make_record_at(2, LogLevel::Info, "second", 10));

        assert_eq!(store.get(0).unwrap().message, "first");
        assert_eq!(store.get(1).unwrap().message, "second");
        assert!(store.get(2).is_none());
    }

    #[test]
    fn find_by_timestamp() {
        let mut store = LogStore::new();
        let base = Utc::now();
        for i in 0..10 {
            store.insert(LogRecord {
                id: i,
                timestamp: base + Duration::seconds(i as i64 * 10),
                level: Some(LogLevel::Info),
                source: "test".into(),
                pid: None,
                tid: None,
                component_name: None,
                process_name: None,
                message: format!("msg-{}", i),
                raw: format!("msg-{}", i),
                metadata: HashMap::new(),
                loader_id: "loader".into(),
            });
        }

        // Jump to timestamp at offset 25s — should land on record at 30s (index 3)
        let target = base + Duration::seconds(25);
        let idx = store.find_by_timestamp(&target);
        assert_eq!(idx, 3);

        // Before all records
        let before = base - Duration::seconds(100);
        assert_eq!(store.find_by_timestamp(&before), 0);

        // After all records
        let after = base + Duration::seconds(1000);
        assert_eq!(store.find_by_timestamp(&after), 10);
    }

    #[test]
    fn range_query() {
        let mut store = LogStore::new();
        for i in 0..5 {
            store.insert(make_record_at(i, LogLevel::Info, &format!("msg-{}", i), i as i64));
        }

        let slice = store.range(1, 3);
        assert_eq!(slice.len(), 2);
        assert_eq!(slice[0].message, "msg-1");
        assert_eq!(slice[1].message, "msg-2");

        // Out of bounds clamped
        let slice = store.range(3, 100);
        assert_eq!(slice.len(), 2);

        // Empty range
        let slice = store.range(5, 5);
        assert_eq!(slice.len(), 0);
    }

    #[test]
    fn clear() {
        let mut store = LogStore::new();
        store.insert(make_record_at(1, LogLevel::Info, "a", 0));
        assert_eq!(store.len(), 1);
        store.clear();
        assert!(store.is_empty());
    }

    #[test]
    fn live_insert_maintains_order() {
        // Simulate live inserts arriving slightly out of order
        let mut store = LogStore::new();
        let base = Utc::now();

        // Batch load historical data
        let batch: Vec<LogRecord> = (0..100).map(|i| LogRecord {
            id: i,
            timestamp: base + Duration::milliseconds(i as i64 * 100),
            level: Some(LogLevel::Info),
            source: "file".into(),
            pid: None, tid: None, component_name: None, process_name: None,
            message: format!("historical-{}", i),
            raw: format!("historical-{}", i),
            metadata: HashMap::new(),
            loader_id: "file-loader".into(),
        }).collect();
        store.insert_batch(batch);

        // Live inserts at the end (most common case)
        for i in 100..110 {
            store.insert(LogRecord {
                id: i,
                timestamp: base + Duration::milliseconds(i as i64 * 100),
                level: Some(LogLevel::Info),
                source: "live".into(),
                pid: None, tid: None, component_name: None, process_name: None,
                message: format!("live-{}", i),
                raw: format!("live-{}", i),
                metadata: HashMap::new(),
                loader_id: "otlp-loader".into(),
            });
        }

        assert_eq!(store.len(), 110);

        // Verify order
        for i in 1..store.len() {
            assert!(store.records()[i].timestamp >= store.records()[i - 1].timestamp);
        }
    }

    #[test]
    fn batch_insert_10k_records() {
        let base = Utc::now();
        let batch: Vec<LogRecord> = (0..10_000).map(|i| LogRecord {
            id: i,
            timestamp: base + Duration::milliseconds(i as i64),
            level: Some(LogLevel::Info),
            source: "bench".into(),
            pid: None, tid: None, component_name: None, process_name: None,
            message: format!("log line {}", i),
            raw: format!("log line {}", i),
            metadata: HashMap::new(),
            loader_id: "bench-loader".into(),
        }).collect();

        let mut store = LogStore::with_capacity(10_000);
        store.insert_batch(batch);
        assert_eq!(store.len(), 10_000);

        // Verify sorted
        for i in 1..store.len() {
            assert!(store.records()[i].timestamp >= store.records()[i - 1].timestamp);
        }
    }
}
