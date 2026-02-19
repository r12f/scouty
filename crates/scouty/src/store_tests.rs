#[cfg(test)]
mod tests {
    use crate::record::{LogLevel, LogRecord};
    use crate::store::LogStore;
    use chrono::{Duration, Utc};
    use std::collections::HashMap;

    fn make_record_at(id: u64, level: LogLevel, message: &str, offset_secs: i64) -> LogRecord {
        let base = Utc::now();
        LogRecord {
            id,
            timestamp: base + Duration::seconds(offset_secs),
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

    fn make_record_with_ts(id: u64, ts: chrono::DateTime<Utc>) -> LogRecord {
        LogRecord {
            id,
            timestamp: ts,
            level: Some(LogLevel::Info),
            source: "test".into(),
            pid: None,
            tid: None,
            component_name: None,
            process_name: None,
            message: format!("msg-{}", id),
            raw: format!("msg-{}", id),
            metadata: HashMap::new(),
            loader_id: "test-loader".into(),
        }
    }

    // ========================================
    // Basic API tests (backward compatibility)
    // ========================================

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
            store.insert(make_record_with_ts(i, base + Duration::seconds(i as i64 * 10)));
        }

        let target = base + Duration::seconds(25);
        let idx = store.find_by_timestamp(&target);
        assert_eq!(idx, 3);

        let before = base - Duration::seconds(100);
        assert_eq!(store.find_by_timestamp(&before), 0);

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

        let slice = store.range(3, 100);
        assert_eq!(slice.len(), 2);

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

    // ========================================
    // Live insert tests
    // ========================================

    #[test]
    fn live_insert_maintains_order() {
        let mut store = LogStore::new();
        let base = Utc::now();

        let batch: Vec<LogRecord> = (0..100)
            .map(|i| make_record_with_ts(i, base + Duration::milliseconds(i as i64 * 100)))
            .collect();
        store.insert_batch(batch);

        for i in 100..110 {
            store.insert(make_record_with_ts(i, base + Duration::milliseconds(i as i64 * 100)));
        }

        assert_eq!(store.len(), 110);
        let records = store.records();
        for i in 1..records.len() {
            assert!(records[i].timestamp >= records[i - 1].timestamp);
        }
    }

    #[test]
    fn batch_insert_10k_records() {
        let base = Utc::now();
        let batch: Vec<LogRecord> = (0..10_000)
            .map(|i| make_record_with_ts(i, base + Duration::milliseconds(i as i64)))
            .collect();

        let mut store = LogStore::with_capacity(10_000);
        store.insert_batch(batch);
        assert_eq!(store.len(), 10_000);

        let records = store.records();
        for i in 1..records.len() {
            assert!(records[i].timestamp >= records[i - 1].timestamp);
        }
    }

    // ========================================
    // Segmented architecture tests
    // ========================================

    #[test]
    fn segment_count_grows_with_data() {
        let mut store = LogStore::new();
        let base = Utc::now();

        // Insert more than one segment's worth (64K)
        let batch: Vec<LogRecord> = (0..70_000u64)
            .map(|i| make_record_with_ts(i, base + Duration::milliseconds(i as i64)))
            .collect();
        store.insert_batch(batch);

        assert_eq!(store.len(), 70_000);
        assert!(store.segment_count() >= 2, "Expected multiple segments, got {}", store.segment_count());
    }

    #[test]
    fn iter_matches_records() {
        let mut store = LogStore::new();
        let base = Utc::now();

        let batch: Vec<LogRecord> = (0..100)
            .map(|i| make_record_with_ts(i, base + Duration::milliseconds(i as i64)))
            .collect();
        store.insert_batch(batch);

        let records = store.records();
        let iter_records: Vec<&LogRecord> = store.iter().collect();

        assert_eq!(records.len(), iter_records.len());
        for (a, b) in records.iter().zip(iter_records.iter()) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.timestamp, b.timestamp);
        }
    }

    #[test]
    fn out_of_order_inserts() {
        let mut store = LogStore::new();
        let base = Utc::now();

        // Insert 100 records in order
        for i in 0..100u64 {
            store.insert(make_record_with_ts(i, base + Duration::seconds(i as i64)));
        }

        // Insert out-of-order records
        store.insert(make_record_with_ts(200, base + Duration::seconds(50)));
        store.insert(make_record_with_ts(201, base + Duration::seconds(10)));
        store.insert(make_record_with_ts(202, base + Duration::seconds(0)));

        assert_eq!(store.len(), 103);
        let records = store.records();
        for i in 1..records.len() {
            assert!(records[i].timestamp >= records[i - 1].timestamp,
                "Order violation at index {}: {:?} > {:?}",
                i, records[i - 1].timestamp, records[i].timestamp);
        }
    }

    #[test]
    fn range_across_segments() {
        let mut store = LogStore::new();
        let base = Utc::now();

        // Create enough records to span multiple segments
        let batch: Vec<LogRecord> = (0..70_000u64)
            .map(|i| make_record_with_ts(i, base + Duration::milliseconds(i as i64)))
            .collect();
        store.insert_batch(batch);

        // Range spanning segment boundary
        let range = store.range(65_000, 66_000);
        assert_eq!(range.len(), 1000);
        for i in 1..range.len() {
            assert!(range[i].timestamp >= range[i - 1].timestamp);
        }
    }

    #[test]
    fn get_across_segments() {
        let mut store = LogStore::new();
        let base = Utc::now();

        let batch: Vec<LogRecord> = (0..70_000u64)
            .map(|i| make_record_with_ts(i, base + Duration::milliseconds(i as i64)))
            .collect();
        store.insert_batch(batch);

        // Access records in different segments
        assert!(store.get(0).is_some());
        assert!(store.get(65_000).is_some());
        assert!(store.get(69_999).is_some());
        assert!(store.get(70_000).is_none());
    }

    #[test]
    fn find_by_timestamp_across_segments() {
        let mut store = LogStore::new();
        let base = Utc::now();

        let batch: Vec<LogRecord> = (0..70_000u64)
            .map(|i| make_record_with_ts(i, base + Duration::milliseconds(i as i64)))
            .collect();
        store.insert_batch(batch);

        // Find in second segment
        let target = base + Duration::milliseconds(65_500);
        let idx = store.find_by_timestamp(&target);
        assert!(idx >= 65_500 && idx <= 65_501, "Expected ~65500, got {}", idx);
    }

    // ========================================
    // Performance benchmarks (1M records)
    // ========================================

    #[test]
    fn perf_1m_monotonic_inserts() {
        let base = Utc::now();
        let mut store = LogStore::new();
        let start = std::time::Instant::now();

        for i in 0..1_000_000u64 {
            store.insert(LogRecord {
                id: i,
                timestamp: base + Duration::microseconds(i as i64),
                level: Some(LogLevel::Info),
                source: "bench".into(),
                pid: None, tid: None, component_name: None, process_name: None,
                message: format!("msg-{}", i),
                raw: format!("msg-{}", i),
                metadata: HashMap::new(),
                loader_id: "bench".into(),
            });
        }

        let elapsed = start.elapsed();
        println!("[perf] 1M monotonic inserts: {:?}", elapsed);
        assert_eq!(store.len(), 1_000_000);
        assert!(store.segment_count() > 1, "Expected multiple segments");
    }

    #[test]
    fn perf_1m_batch_insert() {
        let base = Utc::now();
        let batch: Vec<LogRecord> = (0..1_000_000u64)
            .map(|i| LogRecord {
                id: i,
                timestamp: base + Duration::microseconds(i as i64),
                level: Some(LogLevel::Info),
                source: "bench".into(),
                pid: None, tid: None, component_name: None, process_name: None,
                message: format!("msg-{}", i),
                raw: format!("msg-{}", i),
                metadata: HashMap::new(),
                loader_id: "bench".into(),
            })
            .collect();

        let mut store = LogStore::new();
        let start = std::time::Instant::now();
        store.insert_batch(batch);
        let elapsed = start.elapsed();

        println!("[perf] 1M batch insert: {:?}", elapsed);
        assert_eq!(store.len(), 1_000_000);
        assert!(elapsed.as_secs() < 10, "Batch insert took {:?}, expected < 10s", elapsed);
    }

    #[test]
    fn perf_1m_random_index_query() {
        let base = Utc::now();
        let batch: Vec<LogRecord> = (0..1_000_000u64)
            .map(|i| LogRecord {
                id: i,
                timestamp: base + Duration::microseconds(i as i64),
                level: Some(LogLevel::Info),
                source: "bench".into(),
                pid: None, tid: None, component_name: None, process_name: None,
                message: format!("msg-{}", i),
                raw: format!("msg-{}", i),
                metadata: HashMap::new(),
                loader_id: "bench".into(),
            })
            .collect();

        let mut store = LogStore::new();
        store.insert_batch(batch);

        let start = std::time::Instant::now();
        // Query 10K random indices
        for i in (0..1_000_000u64).step_by(100) {
            let _ = store.get(i as usize);
        }
        let elapsed = start.elapsed();
        println!("[perf] 10K random index queries on 1M store: {:?}", elapsed);
    }

    #[test]
    fn perf_1m_sequential_traversal() {
        let base = Utc::now();
        let batch: Vec<LogRecord> = (0..1_000_000u64)
            .map(|i| LogRecord {
                id: i,
                timestamp: base + Duration::microseconds(i as i64),
                level: Some(LogLevel::Info),
                source: "bench".into(),
                pid: None, tid: None, component_name: None, process_name: None,
                message: format!("msg-{}", i),
                raw: format!("msg-{}", i),
                metadata: HashMap::new(),
                loader_id: "bench".into(),
            })
            .collect();

        let mut store = LogStore::new();
        store.insert_batch(batch);

        // Simulate TUI scroll: iterate through all records
        let start = std::time::Instant::now();
        let mut count = 0u64;
        for record in store.iter() {
            count += record.id;
        }
        let elapsed = start.elapsed();
        println!("[perf] Sequential traversal of 1M records: {:?} (checksum: {})", elapsed, count);
    }

    #[test]
    fn perf_1m_time_range_query() {
        let base = Utc::now();
        let batch: Vec<LogRecord> = (0..1_000_000u64)
            .map(|i| LogRecord {
                id: i,
                timestamp: base + Duration::microseconds(i as i64),
                level: Some(LogLevel::Info),
                source: "bench".into(),
                pid: None, tid: None, component_name: None, process_name: None,
                message: format!("msg-{}", i),
                raw: format!("msg-{}", i),
                metadata: HashMap::new(),
                loader_id: "bench".into(),
            })
            .collect();

        let mut store = LogStore::new();
        store.insert_batch(batch);

        let start = std::time::Instant::now();
        let target = base + Duration::microseconds(500_000);
        let idx = store.find_by_timestamp(&target);
        let elapsed = start.elapsed();
        println!("[perf] Time range query on 1M store: {:?} (index: {})", elapsed, idx);
        assert!(idx >= 499_999 && idx <= 500_001);
    }
}
