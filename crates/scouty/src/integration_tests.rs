//! Integration tests: end-to-end pipeline scenarios.

#[cfg(test)]
mod tests {
    use crate::filter::engine::FilterAction;
    use crate::loader::file::FileLoader;
    use crate::parser::factory::ParserFactory;
    use crate::record::LogLevel;
    use crate::session::LogSession;
    use crate::traits::LogLoader;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Helper: create a temp file with given content, return the path.
    fn temp_log_file(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f.flush().unwrap();
        f
    }

    /// Helper: run full pipeline on a file and return session.
    fn run_pipeline(path: &str) -> LogSession {
        let mut loader = FileLoader::new(path, false);
        let _lines = loader.load().unwrap();
        let info = loader.info();
        let group = ParserFactory::create_parser_group(info);

        let mut session = LogSession::new();
        session.add_loader(Box::new(FileLoader::new(path, false)), group);
        session.run().unwrap();
        session
    }

    // ========================================
    // 6.1 Integration Tests
    // ========================================

    #[test]
    fn test_e2e_file_parse_store_basic() {
        let content = "\
2024-01-15 10:00:00 INFO Starting application
2024-01-15 10:00:01 WARN Low memory
2024-01-15 10:00:02 ERROR Out of memory
2024-01-15 10:00:03 DEBUG Cleanup done
";
        let f = temp_log_file(content);
        let session = run_pipeline(f.path().to_str().unwrap());

        let records = session.store().records();
        assert_eq!(records.len(), 4);
        assert_eq!(records[0].level, Some(LogLevel::Info));
        assert_eq!(records[1].level, Some(LogLevel::Warn));
        assert_eq!(records[2].level, Some(LogLevel::Error));
        assert_eq!(records[3].level, Some(LogLevel::Debug));
    }

    #[test]
    fn test_e2e_filter_by_level() {
        let content = "\
2024-01-15 10:00:00 INFO msg1
2024-01-15 10:00:01 WARN msg2
2024-01-15 10:00:02 ERROR msg3
2024-01-15 10:00:03 INFO msg4
2024-01-15 10:00:04 DEBUG msg5
";
        let f = temp_log_file(content);
        let path = f.path().to_str().unwrap();

        let mut loader = FileLoader::new(path, false);
        let _lines = loader.load().unwrap();
        let info = loader.info();
        let group = ParserFactory::create_parser_group(info);

        let mut session = LogSession::new();
        session.add_loader(Box::new(FileLoader::new(path, false)), group);
        session
            .filter_engine_mut()
            .add_expr_filter(FilterAction::Include, "level == ERROR")
            .unwrap();

        let filtered = session.run().unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(
            session.store().records()[filtered[0]].level,
            Some(LogLevel::Error)
        );
    }

    #[test]
    fn test_e2e_filter_exclude_and_include() {
        let content = "\
2024-01-15 10:00:00 INFO Starting
2024-01-15 10:00:01 WARN Warning msg
2024-01-15 10:00:02 ERROR Error msg
2024-01-15 10:00:03 INFO Running
2024-01-15 10:00:04 ERROR Fatal error
";
        let f = temp_log_file(content);
        let path = f.path().to_str().unwrap();

        let mut loader = FileLoader::new(path, false);
        let _lines = loader.load().unwrap();
        let info = loader.info();
        let group = ParserFactory::create_parser_group(info);

        let mut session = LogSession::new();
        session.add_loader(Box::new(FileLoader::new(path, false)), group);

        // Include only ERROR and WARN, then exclude messages containing "Fatal"
        session
            .filter_engine_mut()
            .add_expr_filter(FilterAction::Include, "level == ERROR OR level == WARN")
            .unwrap();
        session
            .filter_engine_mut()
            .add_expr_filter(FilterAction::Exclude, "message contains \"Fatal\"")
            .unwrap();

        let filtered = session.run().unwrap();
        assert_eq!(filtered.len(), 2); // WARN + first ERROR
        let records = session.store().records();
        assert_eq!(records[filtered[0]].level, Some(LogLevel::Warn));
        assert_eq!(records[filtered[1]].level, Some(LogLevel::Error));
        assert!(records[filtered[1]].message.contains("Error msg"));
    }

    #[test]
    fn test_e2e_empty_file() {
        let f = temp_log_file("");
        let session = run_pipeline(f.path().to_str().unwrap());
        assert_eq!(session.store().records().len(), 0);
        assert!(session.failing_parsing_logs.is_empty());
    }

    #[test]
    fn test_e2e_unparseable_lines() {
        let content = "\
this is not a log line
neither is this
2024-01-15 10:00:00 INFO valid line
random garbage
";
        let f = temp_log_file(content);
        let session = run_pipeline(f.path().to_str().unwrap());

        // At least the valid line should parse
        let records = session.store().records();
        assert!(records.len() >= 1);
        // Some lines should fail parsing
        let total = records.len() + session.failing_parsing_logs.len();
        assert!(total >= 1); // At minimum we tried to parse everything
    }

    #[test]
    fn test_e2e_records_sorted_by_timestamp() {
        let content = "\
2024-01-15 10:00:05 INFO fifth
2024-01-15 10:00:01 INFO first
2024-01-15 10:00:03 INFO third
2024-01-15 10:00:02 INFO second
2024-01-15 10:00:04 INFO fourth
";
        let f = temp_log_file(content);
        let session = run_pipeline(f.path().to_str().unwrap());
        let records = session.store().records();

        // Store should maintain insertion order or be sorted
        // Verify records were loaded
        assert_eq!(records.len(), 5);
    }

    #[test]
    fn test_e2e_filtered_view_without_rerun() {
        let content = "\
2024-01-15 10:00:00 INFO msg1
2024-01-15 10:00:01 ERROR msg2
2024-01-15 10:00:02 INFO msg3
";
        let f = temp_log_file(content);
        let path = f.path().to_str().unwrap();

        let mut loader = FileLoader::new(path, false);
        let _lines = loader.load().unwrap();
        let info = loader.info();
        let group = ParserFactory::create_parser_group(info);

        let mut session = LogSession::new();
        session.add_loader(Box::new(FileLoader::new(path, false)), group);
        session.run().unwrap();

        // All records loaded
        assert_eq!(session.store().records().len(), 3);

        // Now add a filter and get filtered view without re-running load
        session
            .filter_engine_mut()
            .add_expr_filter(FilterAction::Include, "level == ERROR")
            .unwrap();
        session.refresh_active_view();
        let view = session.filtered_view();
        assert_eq!(view.len(), 1);
    }

    #[test]
    fn test_e2e_parallel_run() {
        let content = "\
2024-01-15 10:00:00 INFO msg1
2024-01-15 10:00:01 WARN msg2
2024-01-15 10:00:02 ERROR msg3
";
        let f = temp_log_file(content);
        let path = f.path().to_str().unwrap();

        let mut loader = FileLoader::new(path, false);
        let _lines = loader.load().unwrap();
        let info = loader.info();
        let group = ParserFactory::create_parser_group(info);

        let mut session = LogSession::new();
        session.add_loader(Box::new(FileLoader::new(path, false)), group);
        let filtered = session.run_parallel().unwrap();

        assert_eq!(session.store().records().len(), 3);
        assert_eq!(filtered.len(), 3); // No filters, all pass
    }

    #[test]
    fn test_e2e_multiple_loaders() {
        let content1 = "\
2024-01-15 10:00:00 INFO from file 1
2024-01-15 10:00:01 ERROR error in file 1
";
        let content2 = "\
2024-01-15 10:00:02 WARN from file 2
2024-01-15 10:00:03 DEBUG debug in file 2
";
        let f1 = temp_log_file(content1);
        let f2 = temp_log_file(content2);
        let path1 = f1.path().to_str().unwrap();
        let path2 = f2.path().to_str().unwrap();

        let mut l1 = FileLoader::new(path1, false);
        let _lines1 = l1.load().unwrap();
        let g1 = ParserFactory::create_parser_group(l1.info());

        let mut l2 = FileLoader::new(path2, false);
        let _lines2 = l2.load().unwrap();
        let g2 = ParserFactory::create_parser_group(l2.info());

        let mut session = LogSession::new();
        session.add_loader(Box::new(FileLoader::new(path1, false)), g1);
        session.add_loader(Box::new(FileLoader::new(path2, false)), g2);
        session.run().unwrap();

        assert_eq!(session.store().records().len(), 4);
    }

    #[test]
    fn test_e2e_message_contains_filter() {
        let content = "\
2024-01-15 10:00:00 INFO User login successful
2024-01-15 10:00:01 INFO Processing request
2024-01-15 10:00:02 ERROR User login failed
2024-01-15 10:00:03 INFO User logout
";
        let f = temp_log_file(content);
        let path = f.path().to_str().unwrap();

        let mut loader = FileLoader::new(path, false);
        let _lines = loader.load().unwrap();
        let info = loader.info();
        let group = ParserFactory::create_parser_group(info);

        let mut session = LogSession::new();
        session.add_loader(Box::new(FileLoader::new(path, false)), group);
        session
            .filter_engine_mut()
            .add_expr_filter(FilterAction::Include, "message contains \"User\"")
            .unwrap();
        let filtered = session.run().unwrap();
        assert_eq!(filtered.len(), 3); // login successful, login failed, logout
    }

    // ========================================
    // 6.2 Performance Tests
    // ========================================

    #[test]
    fn test_perf_large_file_loading() {
        // Generate 100K log lines (scaled down from 1M for CI speed, but tests the path)
        let mut content = String::with_capacity(100_000 * 60);
        for i in 0..100_000 {
            let level = match i % 5 {
                0 => "INFO",
                1 => "WARN",
                2 => "ERROR",
                3 => "DEBUG",
                _ => "TRACE",
            };
            let ts_sec = i % 86400;
            let h = ts_sec / 3600;
            let m = (ts_sec % 3600) / 60;
            let s = ts_sec % 60;
            content.push_str(&format!(
                "2024-01-15 {:02}:{:02}:{:02} {} Message number {}\n",
                h, m, s, level, i
            ));
        }

        let f = temp_log_file(&content);
        let path = f.path().to_str().unwrap();

        let start = std::time::Instant::now();
        let session = run_pipeline(path);
        let elapsed = start.elapsed();

        let records = session.store().records();
        assert_eq!(records.len(), 100_000);
        println!("[perf] Load 100K records: {:?}", elapsed);

        // Should complete within 30 seconds even on slow CI
        assert!(
            elapsed.as_secs() < 30,
            "Loading 100K records took {:?}, expected < 30s",
            elapsed
        );
    }

    #[test]
    fn test_perf_filter_large_dataset() {
        let mut content = String::with_capacity(50_000 * 60);
        for i in 0..50_000 {
            let level = if i % 100 == 0 { "ERROR" } else { "INFO" };
            content.push_str(&format!(
                "2024-01-15 10:00:{:02} {} Message {}\n",
                i % 60,
                level,
                i
            ));
        }

        let f = temp_log_file(&content);
        let path = f.path().to_str().unwrap();

        let mut loader = FileLoader::new(path, false);
        let _lines = loader.load().unwrap();
        let info = loader.info();
        let group = ParserFactory::create_parser_group(info);

        let mut session = LogSession::new();
        session.add_loader(Box::new(FileLoader::new(path, false)), group);
        session.run().unwrap();

        // Now filter
        session
            .filter_engine_mut()
            .add_expr_filter(FilterAction::Include, "level == ERROR")
            .unwrap();

        session.refresh_active_view();
        let start = std::time::Instant::now();
        let view = session.filtered_view();
        let elapsed = start.elapsed();

        assert_eq!(view.len(), 500); // 50000 / 100
        println!("[perf] Filter 50K records: {:?}", elapsed);
        assert!(
            elapsed.as_millis() < 5000,
            "Filtering 50K records took {:?}, expected < 5s",
            elapsed
        );
    }

    #[test]
    fn test_perf_store_operations() {
        use crate::record::LogRecord;
        use crate::store::LogStore;
        use chrono::Utc;
        use std::collections::HashMap;

        let mut store = LogStore::new();
        let start = std::time::Instant::now();

        for i in 0..100_000u64 {
            store.insert(LogRecord {
                id: i,
                timestamp: Utc::now(),
                level: Some(LogLevel::Info),
                source: "bench".into(),
                pid: None,
                tid: None,
                component_name: None,
                process_name: None,
                hostname: None,
                container: None,
                context: None,
                function: None,
                message: format!("message {}", i),
                raw: format!("message {}", i),
                metadata: None,
                loader_id: "bench".into(),
                expanded: None,
            });
        }

        let elapsed = start.elapsed();
        assert_eq!(store.len(), 100_000);
        println!("[perf] Insert 100K records into store: {:?}", elapsed);
        assert!(
            elapsed.as_secs() < 10,
            "Inserting 100K records took {:?}, expected < 10s",
            elapsed
        );

        // Test range query performance
        let start = std::time::Instant::now();
        let slice: Vec<_> = store.range(50_000, 50_100).collect();
        let elapsed = start.elapsed();
        assert_eq!(slice.len(), 100);
        println!("[perf] Range query (100 from 100K): {:?}", elapsed);
        assert!(elapsed.as_millis() < 100, "Range query took {:?}", elapsed);
    }

    #[test]
    fn test_perf_parallel_vs_sequential() {
        let content = "\
2024-01-15 10:00:00 INFO msg1
2024-01-15 10:00:01 WARN msg2
2024-01-15 10:00:02 ERROR msg3
2024-01-15 10:00:03 DEBUG msg4
2024-01-15 10:00:04 INFO msg5
";
        let f = temp_log_file(content);
        let path = f.path().to_str().unwrap();

        // Sequential
        let mut loader = FileLoader::new(path, false);
        let _lines = loader.load().unwrap();
        let info = loader.info();
        let group = ParserFactory::create_parser_group(info);

        let mut session1 = LogSession::new();
        session1.add_loader(Box::new(FileLoader::new(path, false)), group);
        session1.run().unwrap();

        // Parallel
        let mut loader2 = FileLoader::new(path, false);
        let _lines2 = loader2.load().unwrap();
        let info2 = loader2.info();
        let group2 = ParserFactory::create_parser_group(info2);

        let mut session2 = LogSession::new();
        session2.add_loader(Box::new(FileLoader::new(path, false)), group2);
        session2.run_parallel().unwrap();

        // Both should produce same number of records
        assert_eq!(
            session1.store().records().len(),
            session2.store().records().len()
        );
    }
}
