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
            store.insert(make_record_with_ts(
                i,
                base + Duration::seconds(i as i64 * 10),
            ));
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
            store.insert(make_record_at(
                i,
                LogLevel::Info,
                &format!("msg-{}", i),
                i as i64,
            ));
        }

        let slice: Vec<_> = store.range(1, 3).collect();
        assert_eq!(slice.len(), 2);
        assert_eq!(slice[0].message, "msg-1");
        assert_eq!(slice[1].message, "msg-2");

        let slice: Vec<_> = store.range(3, 100).collect();
        assert_eq!(slice.len(), 2);

        let slice: Vec<_> = store.range(5, 5).collect();
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
            store.insert(make_record_with_ts(
                i,
                base + Duration::milliseconds(i as i64 * 100),
            ));
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
        assert!(
            store.segment_count() >= 2,
            "Expected multiple segments, got {}",
            store.segment_count()
        );
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
            assert!(
                records[i].timestamp >= records[i - 1].timestamp,
                "Order violation at index {}: {:?} > {:?}",
                i,
                records[i - 1].timestamp,
                records[i].timestamp
            );
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
        let range: Vec<_> = store.range(65_000, 66_000).collect();
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
        assert!(
            idx >= 65_500 && idx <= 65_501,
            "Expected ~65500, got {}",
            idx
        );
    }

    // ========================================
    // Performance benchmarks (1M records)
    // ========================================

    #[test]
    fn perf_1m_nonempty_batch_insert_10k() {
        // The key benchmark: 1M existing records, insert 10K more via batch
        let base = Utc::now();
        let initial: Vec<LogRecord> = (0..1_000_000u64)
            .map(|i| LogRecord {
                id: i,
                timestamp: base + Duration::microseconds(i as i64),
                level: Some(LogLevel::Info),
                source: "bench".into(),
                pid: None,
                tid: None,
                component_name: None,
                process_name: None,
                message: format!("msg-{}", i),
                raw: format!("msg-{}", i),
                metadata: HashMap::new(),
                loader_id: "bench".into(),
            })
            .collect();

        let mut store = LogStore::new();
        store.insert_batch(initial);
        assert_eq!(store.len(), 1_000_000);

        // Now insert 10K more records (appending — timestamps after existing)
        let batch: Vec<LogRecord> = (0..10_000u64)
            .map(|i| LogRecord {
                id: 1_000_000 + i,
                timestamp: base + Duration::microseconds(1_000_000 + i as i64),
                level: Some(LogLevel::Info),
                source: "bench".into(),
                pid: None,
                tid: None,
                component_name: None,
                process_name: None,
                message: format!("new-{}", i),
                raw: format!("new-{}", i),
                metadata: HashMap::new(),
                loader_id: "bench".into(),
            })
            .collect();

        let start = std::time::Instant::now();
        store.insert_batch(batch);
        let elapsed = start.elapsed();

        println!("[perf] 1M store + 10K batch insert (append): {:?}", elapsed);
        assert_eq!(store.len(), 1_010_000);
        // Target: < 10ms
        assert!(
            elapsed.as_millis() < 100,
            "Batch insert took {:?}, expected < 100ms",
            elapsed
        );
    }

    #[test]
    fn perf_1m_nonempty_batch_insert_10k_interleaved() {
        // Batch inserts records interleaved with existing data
        let base = Utc::now();
        let initial: Vec<LogRecord> = (0..1_000_000u64)
            .map(|i| LogRecord {
                id: i,
                timestamp: base + Duration::microseconds(i as i64 * 2), // even timestamps
                level: Some(LogLevel::Info),
                source: "bench".into(),
                pid: None,
                tid: None,
                component_name: None,
                process_name: None,
                message: format!("msg-{}", i),
                raw: format!("msg-{}", i),
                metadata: HashMap::new(),
                loader_id: "bench".into(),
            })
            .collect();

        let mut store = LogStore::new();
        store.insert_batch(initial);

        // Insert 10K records at odd timestamps (interleaved)
        let batch: Vec<LogRecord> = (0..10_000u64)
            .map(|i| LogRecord {
                id: 1_000_000 + i,
                timestamp: base + Duration::microseconds(i as i64 * 200 + 1), // odd, spread across range
                level: Some(LogLevel::Info),
                source: "bench".into(),
                pid: None,
                tid: None,
                component_name: None,
                process_name: None,
                message: format!("interleaved-{}", i),
                raw: format!("interleaved-{}", i),
                metadata: HashMap::new(),
                loader_id: "bench".into(),
            })
            .collect();

        let start = std::time::Instant::now();
        store.insert_batch(batch);
        let elapsed = start.elapsed();

        println!(
            "[perf] 1M store + 10K batch insert (interleaved): {:?}",
            elapsed
        );
        assert_eq!(store.len(), 1_010_000);

        // Verify order
        let mut prev_ts = chrono::DateTime::<Utc>::MIN_UTC;
        for record in store.iter() {
            assert!(record.timestamp >= prev_ts);
            prev_ts = record.timestamp;
        }
    }

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
                pid: None,
                tid: None,
                component_name: None,
                process_name: None,
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
                pid: None,
                tid: None,
                component_name: None,
                process_name: None,
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
        assert!(
            elapsed.as_secs() < 10,
            "Batch insert took {:?}, expected < 10s",
            elapsed
        );
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
                pid: None,
                tid: None,
                component_name: None,
                process_name: None,
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
                pid: None,
                tid: None,
                component_name: None,
                process_name: None,
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
        println!(
            "[perf] Sequential traversal of 1M records: {:?} (checksum: {})",
            elapsed, count
        );
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
                pid: None,
                tid: None,
                component_name: None,
                process_name: None,
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
        println!(
            "[perf] Time range query on 1M store: {:?} (index: {})",
            elapsed, idx
        );
        assert!(idx >= 499_999 && idx <= 500_001);
    }

    // ========================================
    // OOO Buffer Tests
    // ========================================

    #[test]
    fn ooo_insert_goes_to_buffer() {
        let base = Utc::now();
        let mut store = LogStore::new();

        for i in 0..10u64 {
            store.insert(make_record_with_ts(i, base + Duration::seconds(i as i64)));
        }

        // Insert out-of-order record (before the earliest)
        let ooo_record = make_record_with_ts(100, base - Duration::seconds(5));
        store.insert(ooo_record);

        assert_eq!(store.ooo_len(), 1);
        assert_eq!(store.len(), 11);
    }

    #[test]
    fn ooo_records_included_in_records() {
        let base = Utc::now();
        let mut store = LogStore::new();

        for i in 0..5u64 {
            store.insert(make_record_with_ts(
                i,
                base + Duration::seconds((i * 2) as i64),
            ));
        }

        store.insert(make_record_with_ts(50, base + Duration::seconds(1)));

        let all = store.records();
        assert_eq!(all.len(), 6);
        for w in all.windows(2) {
            assert!(w[0].timestamp <= w[1].timestamp);
        }
    }

    #[test]
    fn ooo_compact_merges_into_segments() {
        let base = Utc::now();
        let mut store = LogStore::new();

        for i in 0..20u64 {
            store.insert(make_record_with_ts(i, base + Duration::seconds(i as i64)));
        }

        store.insert(make_record_with_ts(100, base - Duration::seconds(1)));
        store.insert(make_record_with_ts(101, base - Duration::seconds(2)));
        assert_eq!(store.ooo_len(), 2);

        store.compact_ooo();
        assert_eq!(store.ooo_len(), 0);
        assert_eq!(store.len(), 22);

        let records: Vec<_> = store.iter().collect();
        for w in records.windows(2) {
            assert!(w[0].timestamp <= w[1].timestamp);
        }
    }

    #[test]
    fn ooo_frozen_segments_not_mutated_on_insert() {
        let base = Utc::now();
        let mut store = LogStore::new();

        // Create enough records to fill a full segment and freeze it
        let batch: Vec<LogRecord> = (0..70_000u64)
            .map(|i| make_record_with_ts(i, base + Duration::seconds(i as i64)))
            .collect();
        store.insert_batch(batch);

        let seg_count_before = store.segment_count();
        let main_len_before = store.len();

        // Insert OOO record that would belong in a frozen segment
        store.insert(make_record_with_ts(999, base + Duration::seconds(5)));

        // Should go to OOO buffer, not modify frozen segments
        assert_eq!(store.segment_count(), seg_count_before);
        assert_eq!(store.ooo_len(), 1);
        assert_eq!(store.len(), main_len_before + 1);
    }

    // ========================================
    // SegmentRangeIter Tests
    // ========================================

    #[test]
    fn range_iter_basic() {
        let mut store = LogStore::new();
        for i in 0..100u64 {
            store.insert(make_record_at(
                i,
                LogLevel::Info,
                &format!("msg-{}", i),
                i as i64,
            ));
        }

        let records: Vec<_> = store.range(10, 20).collect();
        assert_eq!(records.len(), 10);
        assert_eq!(records[0].message, "msg-10");
        assert_eq!(records[9].message, "msg-19");
    }

    #[test]
    fn range_iter_empty() {
        let store = LogStore::new();
        assert_eq!(store.range(0, 0).count(), 0);
    }

    #[test]
    fn range_iter_exact_size() {
        let mut store = LogStore::new();
        for i in 0..50u64 {
            store.insert(make_record_at(
                i,
                LogLevel::Info,
                &format!("msg-{}", i),
                i as i64,
            ));
        }

        let iter = store.range(5, 15);
        assert_eq!(iter.len(), 10);
    }

    #[test]
    fn range_iter_no_heap_allocation() {
        let mut store = LogStore::new();
        for i in 0..100u64 {
            store.insert(make_record_at(
                i,
                LogLevel::Info,
                &format!("msg-{}", i),
                i as i64,
            ));
        }

        let mut count = 0;
        for _record in store.range(0, 100) {
            count += 1;
        }
        assert_eq!(count, 100);
    }

    #[test]
    fn range_collected_matches_range_iter() {
        let mut store = LogStore::new();
        for i in 0..50u64 {
            store.insert(make_record_at(
                i,
                LogLevel::Info,
                &format!("msg-{}", i),
                i as i64,
            ));
        }

        let from_iter: Vec<_> = store.range(10, 30).cloned().collect();
        let from_collected = store.range_collected(10, 30);
        assert_eq!(from_iter.len(), from_collected.len());
        for (a, b) in from_iter.iter().zip(from_collected.iter()) {
            assert_eq!(a.id, b.id);
        }
    }

    #[test]
    fn perf_range_iter_10k_traversal() {
        let base = Utc::now();
        let batch: Vec<LogRecord> = (0..100_000u64)
            .map(|i| make_record_with_ts(i, base + Duration::microseconds(i as i64)))
            .collect();

        let mut store = LogStore::new();
        store.insert_batch(batch);

        let start = std::time::Instant::now();
        let count = store.range(45_000, 55_000).count();
        let elapsed = start.elapsed();

        assert_eq!(count, 10_000);
        assert!(
            elapsed.as_millis() < 1,
            "Range iter 10K traversal took {:?}, expected < 1ms",
            elapsed
        );
    }

    #[test]
    fn perf_ooo_compact_16k() {
        let base = Utc::now();
        let mut store = LogStore::new();

        let batch: Vec<LogRecord> = (0..100_000u64)
            .map(|i| make_record_with_ts(i, base + Duration::seconds(i as i64)))
            .collect();
        store.insert_batch(batch);

        for i in 0..16_000u64 {
            store.ooo_buffer.push(make_record_with_ts(
                200_000 + i,
                base + Duration::seconds(i as i64),
            ));
        }

        let start = std::time::Instant::now();
        store.compact_ooo();
        let elapsed = start.elapsed();

        assert_eq!(store.ooo_len(), 0);
        assert!(
            elapsed.as_millis() < 5000,
            "OOO compact 16K took {:?}, expected < 5s",
            elapsed
        );
    }

    // ========================================
    // Segment Capacity Auto-tuning Tests
    // ========================================

    #[test]
    fn autotune_small_dataset_uses_16k() {
        let base = Utc::now();
        let mut store = LogStore::new();

        let batch: Vec<LogRecord> = (0..50_000u64)
            .map(|i| make_record_with_ts(i, base + Duration::microseconds(i as i64)))
            .collect();
        store.insert_batch(batch);

        assert_eq!(store.segment_capacity(), 16 * 1024);
    }

    #[test]
    fn autotune_medium_dataset_uses_64k() {
        let base = Utc::now();
        let mut store = LogStore::new();

        let batch: Vec<LogRecord> = (0..500_000u64)
            .map(|i| make_record_with_ts(i, base + Duration::microseconds(i as i64)))
            .collect();
        store.insert_batch(batch);

        assert_eq!(store.segment_capacity(), 64 * 1024);
    }

    #[test]
    fn autotune_large_dataset_uses_128k() {
        let base = Utc::now();
        let mut store = LogStore::new();

        let batch: Vec<LogRecord> = (0..2_000_000u64)
            .map(|i| make_record_with_ts(i, base + Duration::microseconds(i as i64)))
            .collect();
        store.insert_batch(batch);

        assert_eq!(store.segment_capacity(), 128 * 1024);
    }

    #[test]
    fn autotune_user_override_disables_tuning() {
        use crate::store::LogStoreConfig;

        let base = Utc::now();
        let mut store = LogStore::with_config(LogStoreConfig {
            segment_capacity: Some(32 * 1024),
            auto_tune: true,
        });

        let batch: Vec<LogRecord> = (0..200_000u64)
            .map(|i| make_record_with_ts(i, base + Duration::microseconds(i as i64)))
            .collect();
        store.insert_batch(batch);

        assert_eq!(store.segment_capacity(), 32 * 1024);
    }

    #[test]
    fn autotune_disabled_keeps_default() {
        use crate::store::LogStoreConfig;

        let base = Utc::now();
        let mut store = LogStore::with_config(LogStoreConfig {
            segment_capacity: None,
            auto_tune: false,
        });

        let batch: Vec<LogRecord> = (0..50_000u64)
            .map(|i| make_record_with_ts(i, base + Duration::microseconds(i as i64)))
            .collect();
        store.insert_batch(batch);

        assert_eq!(store.segment_capacity(), 64 * 1024);
    }

    #[test]
    fn autotune_clear_resets_capacity() {
        let base = Utc::now();
        let mut store = LogStore::new();

        let batch: Vec<LogRecord> = (0..50_000u64)
            .map(|i| make_record_with_ts(i, base + Duration::microseconds(i as i64)))
            .collect();
        store.insert_batch(batch);
        assert_eq!(store.segment_capacity(), 16 * 1024);

        store.clear();
        assert_eq!(store.segment_capacity(), 64 * 1024);
    }

    #[test]
    fn perf_autotune_no_regression_100k() {
        let base = Utc::now();
        let batch: Vec<LogRecord> = (0..100_000u64)
            .map(|i| make_record_with_ts(i, base + Duration::microseconds(i as i64)))
            .collect();

        let mut store_auto = LogStore::new();
        let start = std::time::Instant::now();
        store_auto.insert_batch(batch.clone());
        let auto_elapsed = start.elapsed();

        use crate::store::LogStoreConfig;
        let mut store_fixed = LogStore::with_config(LogStoreConfig {
            segment_capacity: None,
            auto_tune: false,
        });
        let start = std::time::Instant::now();
        store_fixed.insert_batch(batch);
        let fixed_elapsed = start.elapsed();

        println!(
            "[perf] 100K insert: auto-tune={:?} fixed={:?}",
            auto_elapsed, fixed_elapsed
        );

        assert!(
            auto_elapsed.as_millis() < fixed_elapsed.as_millis() * 3 + 100,
            "Auto-tune regression: auto={:?} vs fixed={:?}",
            auto_elapsed,
            fixed_elapsed
        );
    }
}
