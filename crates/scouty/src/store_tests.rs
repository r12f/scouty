//! Tests for LogStore.

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
        // Insert out of order
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
}
