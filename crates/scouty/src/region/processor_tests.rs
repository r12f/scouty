//! Tests for RegionProcessor.

mod tests {
    use crate::record::{LogLevel, LogRecord};
    use crate::region::config;
    use crate::region::processor::RegionProcessor;
    use chrono::{TimeZone, Utc};
    use std::collections::HashMap;
    use std::sync::Arc;

    fn make_record(id: u64, ts_secs: i64, message: &str, level: Option<LogLevel>) -> LogRecord {
        LogRecord {
            id,
            timestamp: Utc.timestamp_opt(ts_secs, 0).unwrap(),
            level,
            source: Arc::from("test.log"),
            pid: None,
            tid: None,
            component_name: None,
            process_name: None,
            hostname: None,
            container: None,
            context: None,
            function: Some("test".to_string()),
            message: message.to_string(),
            raw: message.to_string(),
            metadata: None,
            loader_id: Arc::from("test"),
            expanded: None,
        }
    }

    const BASIC_CONFIG: &str = r#"
regions:
  - name: "port_startup"
    start_points:
      - filter: 'message contains "addPort"'
        regex: '(?P<port>Ethernet\d+)'
        reason: "add {port}"
    end_points:
      - filter: 'message contains "oper_status up"'
        regex: '(?P<port>Ethernet\d+)'
        reason: "oper up {port}"
    correlate:
      - "port"
    template:
      name: "Port Startup {port}"
      description: "{start_reason} → {end_reason}"
    timeout: "30s"
"#;

    #[test]
    fn test_basic_region_detection() {
        let defs = config::load_from_str(BASIC_CONFIG).unwrap();
        let mut proc = RegionProcessor::new(defs);

        let mut records = vec![
            make_record(0, 1000, "addPort Ethernet0", None),
            make_record(1, 1001, "some log in between", None),
            make_record(2, 1002, "Ethernet0 oper_status up", None),
        ];

        proc.process_records(&mut records);

        assert_eq!(proc.region_count(), 1);
        let region = &proc.regions()[0];
        assert_eq!(region.definition_name, "port_startup");
        assert_eq!(region.name, "Port Startup Ethernet0");
        assert_eq!(region.start_index, 0);
        assert_eq!(region.end_index, 2);

        // Check record tagging
        assert_eq!(
            records[0]
                .metadata
                .as_ref()
                .unwrap()
                .get("_region_pos")
                .unwrap(),
            "start"
        );
        assert_eq!(
            records[1]
                .metadata
                .as_ref()
                .unwrap()
                .get("_region_pos")
                .unwrap(),
            "middle"
        );
        assert_eq!(
            records[2]
                .metadata
                .as_ref()
                .unwrap()
                .get("_region_pos")
                .unwrap(),
            "end"
        );
    }

    #[test]
    fn test_reason_rendering() {
        let defs = config::load_from_str(BASIC_CONFIG).unwrap();
        let mut proc = RegionProcessor::new(defs);

        let mut records = vec![
            make_record(0, 1000, "addPort Ethernet4", None),
            make_record(1, 1002, "Ethernet4 oper_status up", None),
        ];

        proc.process_records(&mut records);

        assert_eq!(proc.region_count(), 1);
        let region = &proc.regions()[0];
        assert_eq!(region.start_reason.as_deref(), Some("add Ethernet4"));
        assert_eq!(region.end_reason.as_deref(), Some("oper up Ethernet4"));
        assert_eq!(
            region.description.as_deref(),
            Some("add Ethernet4 → oper up Ethernet4")
        );
    }

    #[test]
    fn test_correlation_multiple_ports() {
        let defs = config::load_from_str(BASIC_CONFIG).unwrap();
        let mut proc = RegionProcessor::new(defs);

        let mut records = vec![
            make_record(0, 1000, "addPort Ethernet0", None),
            make_record(1, 1001, "addPort Ethernet4", None),
            make_record(2, 1002, "Ethernet4 oper_status up", None),
            make_record(3, 1003, "Ethernet0 oper_status up", None),
        ];

        proc.process_records(&mut records);

        assert_eq!(proc.region_count(), 2);
        // First completed region: Ethernet4 (end came first)
        assert_eq!(proc.regions()[0].name, "Port Startup Ethernet4");
        assert_eq!(proc.regions()[0].start_index, 1);
        assert_eq!(proc.regions()[0].end_index, 2);
        // Second: Ethernet0
        assert_eq!(proc.regions()[1].name, "Port Startup Ethernet0");
        assert_eq!(proc.regions()[1].start_index, 0);
        assert_eq!(proc.regions()[1].end_index, 3);
    }

    #[test]
    fn test_timeout_discards_stale_starts() {
        let defs = config::load_from_str(BASIC_CONFIG).unwrap();
        let mut proc = RegionProcessor::new(defs);

        let mut records = vec![
            make_record(0, 1000, "addPort Ethernet0", None),
            // 60s later — beyond 30s timeout
            make_record(1, 1060, "addPort Ethernet0", None),
            make_record(2, 1062, "Ethernet0 oper_status up", None),
        ];

        proc.process_records(&mut records);

        assert_eq!(proc.region_count(), 1);
        let region = &proc.regions()[0];
        // Should correlate with the second start (fresh), not the first (stale)
        assert_eq!(region.start_index, 1);
        assert_eq!(region.end_index, 2);
    }

    #[test]
    fn test_no_correlate_uses_lifo() {
        let config = r#"
regions:
  - name: "request"
    start_points:
      - filter: 'message contains "start"'
    end_points:
      - filter: 'message contains "end"'
    correlate: []
    template:
      name: "Request"
"#;
        let defs = config::load_from_str(config).unwrap();
        let mut proc = RegionProcessor::new(defs);

        let mut records = vec![
            make_record(0, 1000, "start A", None),
            make_record(1, 1001, "start B", None),
            make_record(2, 1002, "end X", None),
        ];

        proc.process_records(&mut records);

        assert_eq!(proc.region_count(), 1);
        // LIFO: should match with the most recent start (B at index 1)
        assert_eq!(proc.regions()[0].start_index, 1);
    }

    #[test]
    fn test_start_plus_end_same_record() {
        // If a record matches both start and end of different definitions,
        // or if start_index == end_index
        let config = r#"
regions:
  - name: "instant"
    start_points:
      - filter: 'message contains "instant"'
    end_points:
      - filter: 'message contains "instant"'
    correlate: []
    template:
      name: "Instant"
"#;
        let defs = config::load_from_str(config).unwrap();
        let mut proc = RegionProcessor::new(defs);

        // The end check happens first, but no pending start exists yet.
        // Then start check adds it. Next matching record will end it.
        let mut records = vec![
            make_record(0, 1000, "instant event", None),
            make_record(1, 1001, "instant event", None),
        ];

        proc.process_records(&mut records);

        assert_eq!(proc.region_count(), 1);
        assert_eq!(proc.regions()[0].start_index, 0);
        assert_eq!(proc.regions()[0].end_index, 1);
    }

    #[test]
    fn test_no_match_no_region() {
        let defs = config::load_from_str(BASIC_CONFIG).unwrap();
        let mut proc = RegionProcessor::new(defs);

        let mut records = vec![
            make_record(0, 1000, "unrelated log line", None),
            make_record(1, 1001, "another unrelated line", None),
        ];

        proc.process_records(&mut records);

        assert_eq!(proc.region_count(), 0);
        assert!(records[0].metadata.is_none());
    }

    #[test]
    fn test_incremental_processing() {
        let defs = config::load_from_str(BASIC_CONFIG).unwrap();
        let mut proc = RegionProcessor::new(defs);

        // First batch: start only
        let mut batch1 = vec![make_record(0, 1000, "addPort Ethernet0", None)];
        proc.process_records(&mut batch1);
        assert_eq!(proc.region_count(), 0);
        assert_eq!(proc.pending_count(), 1);

        // Second batch: end
        let mut batch2 = vec![make_record(1, 1002, "Ethernet0 oper_status up", None)];
        proc.process_records(&mut batch2);
        assert_eq!(proc.region_count(), 1);
        // Note: the middle records from batch1 won't be tagged since they're
        // in a previous batch. In practice, the caller re-tags from the store.
    }

    #[test]
    fn test_region_metadata_fields() {
        let defs = config::load_from_str(BASIC_CONFIG).unwrap();
        let mut proc = RegionProcessor::new(defs);

        let mut records = vec![
            make_record(0, 1000, "addPort Ethernet0", None),
            make_record(1, 1002, "Ethernet0 oper_status up", None),
        ];

        proc.process_records(&mut records);

        let region = &proc.regions()[0];
        assert_eq!(region.metadata.get("port").unwrap(), "Ethernet0");
    }
}
