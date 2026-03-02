#[cfg(test)]
mod tests {
    use crate::parser::sairedis_parser::*;
    use crate::record::LogRecord;
    use std::sync::Arc;

    fn parse(parser: &SairedisParser, line: &str) -> Option<LogRecord> {
        let source: Arc<str> = Arc::from("test");
        let loader_id: Arc<str> = Arc::from("test");
        parser.parse_shared(line, &source, &loader_id, 0)
    }

    #[test]
    fn test_create_with_oid() {
        let p = SairedisParser::new();
        let r = parse(&p, "2025-05-18.06:38:35.610696|c|SAI_OBJECT_TYPE_HOSTIF:oid:0xd00000000137b|SAI_HOSTIF_ATTR_TYPE=SAI_HOSTIF_TYPE_NETDEV|SAI_HOSTIF_ATTR_OBJ_ID=oid:0x1000000000063|SAI_HOSTIF_ATTR_NAME=Ethernet").unwrap();
        assert_eq!(r.function.as_deref(), Some("Create"));
        assert_eq!(r.component_name.as_deref(), Some("SAI_OBJECT_TYPE_HOSTIF"));
        assert_eq!(r.context.as_deref(), Some("oid:0xd00000000137b"));
        assert_eq!(r.message, "SAI_HOSTIF_ATTR_TYPE=SAI_HOSTIF_TYPE_NETDEV|SAI_HOSTIF_ATTR_OBJ_ID=oid:0x1000000000063|SAI_HOSTIF_ATTR_NAME=Ethernet");
        assert_eq!(r.timestamp.to_rfc3339(), "2025-05-18T06:38:35.610696+00:00");
    }

    #[test]
    fn test_create_with_json_context() {
        let p = SairedisParser::new();
        let r = parse(&p, r#"2025-05-18.06:38:34.056792|c|SAI_OBJECT_TYPE_ROUTE_ENTRY:{"dest":"fe80::/10","switch_id":"oid:0x21000000000000","vr":"oid:0x3000000000083"}|SAI_ROUTE_ENTRY_ATTR_PACKET_ACTION=SAI_PACKET_ACTION_FORWARD|SAI_ROUTE_ENTRY_ATTR_NEXT_HOP_ID=oid:0x1000000000093"#).unwrap();
        assert_eq!(r.function.as_deref(), Some("Create"));
        assert_eq!(
            r.component_name.as_deref(),
            Some("SAI_OBJECT_TYPE_ROUTE_ENTRY")
        );
        assert!(r.context.as_deref().unwrap().contains("fe80::/10"));
        assert!(r.message.contains("SAI_ROUTE_ENTRY_ATTR_PACKET_ACTION"));
    }

    #[test]
    fn test_remove() {
        let p = SairedisParser::new();
        let r = parse(
            &p,
            "2025-05-17.21:58:30.286883|r|SAI_OBJECT_TYPE_VLAN_MEMBER:oid:0x2700000000062e",
        )
        .unwrap();
        assert_eq!(r.function.as_deref(), Some("Remove"));
        assert_eq!(
            r.component_name.as_deref(),
            Some("SAI_OBJECT_TYPE_VLAN_MEMBER")
        );
        assert_eq!(r.context.as_deref(), Some("oid:0x2700000000062e"));
        assert!(r.message.is_empty());
    }

    #[test]
    fn test_set() {
        let p = SairedisParser::new();
        let r = parse(&p, "2025-05-17.18:42:03.233241|s|SAI_OBJECT_TYPE_PORT:oid:0x100000000001e|SAI_PORT_ATTR_ADMIN_STATE=true").unwrap();
        assert_eq!(r.function.as_deref(), Some("Set"));
        assert_eq!(r.component_name.as_deref(), Some("SAI_OBJECT_TYPE_PORT"));
        assert_eq!(r.context.as_deref(), Some("oid:0x100000000001e"));
        assert_eq!(r.message, "SAI_PORT_ATTR_ADMIN_STATE=true");
    }

    #[test]
    fn test_get_and_get_response_stateful() {
        let p = SairedisParser::new();

        // Parse 'g' first — saves context
        let g = parse(&p, "2025-05-17.18:49:14.280510|g|SAI_OBJECT_TYPE_PORT:oid:0x1000000000011|SAI_PORT_ATTR_REMOTE_ADVERTISED_SPEED=16:0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0").unwrap();
        assert_eq!(g.function.as_deref(), Some("Get"));
        assert_eq!(g.context.as_deref(), Some("oid:0x1000000000011"));

        // Parse 'G' — should inherit context from last 'g'
        let gr = parse(&p, "2025-05-17.18:49:14.282097|G|SAI_STATUS_SUCCESS|SAI_PORT_ATTR_REMOTE_ADVERTISED_SPEED=0:null").unwrap();
        assert_eq!(gr.function.as_deref(), Some("GetResponse"));
        assert_eq!(gr.context.as_deref(), Some("oid:0x1000000000011")); // stateful!
        assert_eq!(
            gr.message,
            "SAI_STATUS_SUCCESS|SAI_PORT_ATTR_REMOTE_ADVERTISED_SPEED=0:null"
        );
        assert_eq!(gr.component_name.as_deref(), Some("SAI_OBJECT_TYPE_PORT")); // stateful component propagation
    }

    #[test]
    fn test_get_response_without_prior_get() {
        let p = SairedisParser::new();
        let r = parse(
            &p,
            "2025-05-17.18:49:14.282097|G|SAI_STATUS_SUCCESS|attr=val",
        )
        .unwrap();
        assert_eq!(r.function.as_deref(), Some("GetResponse"));
        assert!(r.context.is_none()); // no prior 'g'
    }

    #[test]
    fn test_bulk_create() {
        let p = SairedisParser::new();
        let r = parse(&p, r#"2025-05-17.18:42:03.242891|C|SAI_OBJECT_TYPE_ROUTE_ENTRY||{"dest":"fc00::78/126","switch_id":"oid:0x21000000000000","vr":"oid:0x3000000000083"}|SAI_ROUTE_ENTRY_ATTR_NEXT_HOP_ID=oid:0x60000000015fe||{"dest":"10.0.0.60/31","switch_id":"oid:0x21000000000000","vr":"oid:0x3000000000083"}|SAI_ROUTE_ENTRY_ATTR_NEXT_HOP_ID=oid:0x60000000015fe"#).unwrap();
        assert_eq!(r.function.as_deref(), Some("BulkCreate"));
        assert_eq!(
            r.component_name.as_deref(),
            Some("SAI_OBJECT_TYPE_ROUTE_ENTRY")
        );
        assert!(r.context.is_none()); // bulk ops have no single context
        assert!(r.message.contains("||")); // message includes entire detail after type
    }

    #[test]
    fn test_bulk_remove() {
        let p = SairedisParser::new();
        let r = parse(&p, r#"2025-05-17.18:41:58.557700|R|SAI_OBJECT_TYPE_ROUTE_ENTRY||{"dest":"fc00::78/126","switch_id":"oid:0x21000000000000","vr":"oid:0x3000000000083"}||{"dest":"10.0.0.60/31","switch_id":"oid:0x21000000000000","vr":"oid:0x3000000000083"}"#).unwrap();
        assert_eq!(r.function.as_deref(), Some("BulkRemove"));
        assert_eq!(
            r.component_name.as_deref(),
            Some("SAI_OBJECT_TYPE_ROUTE_ENTRY")
        );
        assert!(r.message.contains("||"));
    }

    #[test]
    fn test_query_and_query_response_stateful() {
        let p = SairedisParser::new();

        // Parse 'q' first
        let q = parse(&p, "2025-05-17.18:49:14.501934|q|object_type_get_availability|SAI_OBJECT_TYPE_SWITCH:oid:0x21000000000000|SAI_ROUTE_ENTRY_ATTR_IP_ADDR_FAMILY=SAI_IP_ADDR_FAMILY_IPV4").unwrap();
        assert_eq!(
            q.function.as_deref(),
            Some("Query: object_type_get_availability")
        );
        assert_eq!(q.context.as_deref(), Some("oid:0x21000000000000"));
        assert_eq!(
            q.message,
            "SAI_ROUTE_ENTRY_ATTR_IP_ADDR_FAMILY=SAI_IP_ADDR_FAMILY_IPV4"
        );

        // Parse 'Q' — should inherit context from last 'q'
        let qr = parse(&p, "2025-05-17.18:49:14.502430|Q|object_type_get_availability|SAI_STATUS_NOT_IMPLEMENTED|COUNT=0").unwrap();
        assert_eq!(
            qr.function.as_deref(),
            Some("QueryResponse: object_type_get_availability")
        );
        assert_eq!(qr.context.as_deref(), Some("oid:0x21000000000000")); // stateful!
        assert_eq!(qr.message, "SAI_STATUS_NOT_IMPLEMENTED|COUNT=0");
    }

    #[test]
    fn test_notification() {
        let p = SairedisParser::new();
        let r = parse(&p, r#"2025-05-17.18:41:58.563631|n|port_state_change|[{"port_error_status":"SAI_PORT_ERROR_STATUS_CLEAR","port_id":"oid:0x100000000001e","port_state":"SAI_PORT_OPER_STATUS_DOWN"}]|"#).unwrap();
        assert_eq!(
            r.function.as_deref(),
            Some("Notification: port_state_change")
        );
        assert!(r.message.contains("port_error_status"));
        assert!(!r.message.ends_with('|')); // trailing pipe should be trimmed
    }

    #[test]
    fn test_counter_poll() {
        let p = SairedisParser::new();
        let r = parse(&p, "2025-05-17.18:42:03.233241|p|SAI_OBJECT_TYPE_PORT:oid:0x100000000001e|SAI_PORT_ATTR_ADMIN_STATE=true").unwrap();
        assert_eq!(r.function.as_deref(), Some("CounterPoll"));
        assert_eq!(r.component_name.as_deref(), Some("SAI_OBJECT_TYPE_PORT"));
    }

    #[test]
    fn test_unknown_op_code_fallback() {
        let p = SairedisParser::new();
        let r = parse(&p, "2025-05-17.18:42:03.233241|x|some random content here").unwrap();
        assert_eq!(r.function.as_deref(), Some("x"));
        assert_eq!(r.message, "some random content here");
        assert!(r.component_name.is_none());
    }

    #[test]
    fn test_auto_detect_sairedis() {
        assert!(looks_like_sairedis(
            "2025-05-18.06:38:35.610696|c|SAI_OBJECT_TYPE_HOSTIF:oid:0x137b"
        ));
        assert!(looks_like_sairedis(
            "2025-05-17.21:58:30.286883|r|SAI_OBJECT_TYPE_VLAN_MEMBER:oid:0x62e"
        ));
        assert!(looks_like_sairedis(
            "2025-05-17.18:49:14.282097|G|SAI_STATUS_SUCCESS"
        ));
    }

    #[test]
    fn test_auto_detect_rejects_swss() {
        // SWSS has multi-char second segment (TABLE_NAME)
        assert!(!looks_like_sairedis(
            "2025-05-17.18:42:03.233241|SWITCH_TABLE:switch|SET|k:v"
        ));
    }

    #[test]
    fn test_auto_detect_rejects_too_short() {
        assert!(!looks_like_sairedis("short"));
        assert!(!looks_like_sairedis(""));
    }

    #[test]
    fn test_create_json_context_with_colons() {
        // JSON context contains ':' characters — parser should handle correctly
        let p = SairedisParser::new();
        let r = parse(&p, r#"2025-05-18.06:38:34.056792|c|SAI_OBJECT_TYPE_ROUTE_ENTRY:{"dest":"fc00::78/126","switch_id":"oid:0x21000000000000"}|SAI_ROUTE_ENTRY_ATTR_NEXT_HOP_ID=oid:0x60000000015fe"#).unwrap();
        assert_eq!(
            r.component_name.as_deref(),
            Some("SAI_OBJECT_TYPE_ROUTE_ENTRY")
        );
        // Context should be the full JSON string
        assert!(r.context.as_deref().unwrap().starts_with('{'));
        assert!(r.context.as_deref().unwrap().contains("fc00::78/126"));
    }

    #[test]
    fn test_bulk_set_and_bulk_get() {
        let p = SairedisParser::new();
        let r = parse(&p, "2025-05-17.18:42:03.242891|S|SAI_OBJECT_TYPE_PORT||oid:0x1|attr=val||oid:0x2|attr=val2").unwrap();
        assert_eq!(r.function.as_deref(), Some("BulkSet"));

        let r = parse(&p, "2025-05-17.18:42:03.242891|B|SAI_OBJECT_TYPE_PORT||oid:0x1|attr=val||oid:0x2|attr=val2").unwrap();
        assert_eq!(r.function.as_deref(), Some("BulkGet"));
    }

    #[test]
    fn test_multiple_get_response_updates_context() {
        let p = SairedisParser::new();

        // First get
        parse(
            &p,
            "2025-05-17.18:49:14.280510|g|SAI_OBJECT_TYPE_PORT:oid:0x1|attr=val",
        )
        .unwrap();

        // First get response — should have context from first get
        let r1 = parse(&p, "2025-05-17.18:49:14.282097|G|SAI_STATUS_SUCCESS").unwrap();
        assert_eq!(r1.context.as_deref(), Some("oid:0x1"));

        // Second get with different context
        parse(
            &p,
            "2025-05-17.18:49:15.280510|g|SAI_OBJECT_TYPE_PORT:oid:0x2|attr=val",
        )
        .unwrap();

        // Second get response — should have NEW context
        let r2 = parse(&p, "2025-05-17.18:49:15.282097|G|SAI_STATUS_FAILURE").unwrap();
        assert_eq!(r2.context.as_deref(), Some("oid:0x2"));
    }

    #[test]
    fn test_timestamp_parsing_various() {
        let p = SairedisParser::new();

        // Short fractional
        let r = parse(
            &p,
            "2025-01-01.00:00:00.1|c|SAI_OBJECT_TYPE_X:oid:0x1|attr=val",
        )
        .unwrap();
        assert_eq!(r.timestamp.timestamp_micros(), 1735689600_100000);

        // Full 6-digit fractional
        let r = parse(
            &p,
            "2025-12-31.23:59:59.999999|c|SAI_OBJECT_TYPE_X:oid:0x1|attr=val",
        )
        .unwrap();
        assert_eq!(r.timestamp.timestamp_micros(), 1767225599_999999);
    }

    // ── LogLevel mapping tests ──────────────────────────────────────────

    #[test]
    fn test_notification_has_notice_level() {
        let p = SairedisParser::new();
        let r = parse(
            &p,
            r#"2025-05-17.18:41:58.563631|n|port_state_change|[{"port_id":"oid:0x1"}]|"#,
        )
        .unwrap();
        assert_eq!(r.level, Some(crate::record::LogLevel::Notice));
    }

    #[test]
    fn test_create_has_info_level() {
        let p = SairedisParser::new();
        let r = parse(
            &p,
            "2025-05-17.18:42:03.233241|c|SAI_OBJECT_TYPE_PORT:oid:0x1|attr=val",
        )
        .unwrap();
        assert_eq!(r.level, Some(crate::record::LogLevel::Info));
    }

    #[test]
    fn test_get_response_has_info_level() {
        let p = SairedisParser::new();
        let r = parse(&p, "2025-05-17.18:49:14.282097|G|SAI_STATUS_SUCCESS").unwrap();
        assert_eq!(r.level, Some(crate::record::LogLevel::Info));
    }

    #[test]
    fn test_unknown_op_has_info_level() {
        let p = SairedisParser::new();
        let r = parse(&p, "2025-05-17.18:42:03.233241|x|some content").unwrap();
        assert_eq!(r.level, Some(crate::record::LogLevel::Info));
    }

    #[test]
    fn test_performance_parse_rate() {
        let lines: Vec<String> = (0..10_000)
            .map(|i| {
                format!(
                    "2025-05-18.06:38:{:02}.{:06}|c|SAI_OBJECT_TYPE_HOSTIF:oid:0x{:08x}|SAI_HOSTIF_ATTR_TYPE=SAI_HOSTIF_TYPE_NETDEV",
                    i % 60, i % 1_000_000, i
                )
            })
            .collect();

        let p = SairedisParser::new();
        let source: Arc<str> = Arc::from("bench");
        let loader_id: Arc<str> = Arc::from("bench");
        let start = std::time::Instant::now();
        let mut count = 0;
        for (i, line) in lines.iter().enumerate() {
            if p.parse_shared(line, &source, &loader_id, i as u64)
                .is_some()
            {
                count += 1;
            }
        }
        let elapsed = start.elapsed();
        let rate = count as f64 / elapsed.as_secs_f64();
        eprintln!(
            "Sairedis parse: {} rec in {:?} = {:.1}M rec/sec",
            count,
            elapsed,
            rate / 1e6
        );
        assert_eq!(count, 10_000);
        // Only enforce performance threshold in non-debug builds to avoid flaky tests.
        if !cfg!(debug_assertions) {
            assert!(rate > 500_000.0, "Too slow: {:.0} rec/sec", rate);
        }
    }

    #[test]
    fn test_expanded_create_op() {
        let p = SairedisParser::new();
        let r = p
            .parse(
                "2025-01-15.10:30:45.123456|c|SAI_OBJECT_TYPE_ROUTE_ENTRY:oid:0x1234|SAI_ROUTE_ENTRY_ATTR_NEXT_HOP_ID=oid:0x5678",
                "test",
                "loader",
                1,
            )
            .unwrap();
        let expanded = r.expanded.as_ref().unwrap();
        assert_eq!(expanded[0].label, "Operation");
        assert_eq!(
            expanded[0].value,
            crate::record::ExpandedValue::Text("Create".to_string())
        );
        assert_eq!(expanded[1].label, "Object Type");
        assert_eq!(
            expanded[1].value,
            crate::record::ExpandedValue::Text("SAI_OBJECT_TYPE_ROUTE_ENTRY".to_string())
        );
        assert_eq!(expanded[2].label, "OID");
        assert_eq!(
            expanded[2].value,
            crate::record::ExpandedValue::Text("oid:0x1234".to_string())
        );
        assert_eq!(expanded[3].label, "Attributes");
        if let crate::record::ExpandedValue::KeyValue(pairs) = &expanded[3].value {
            assert_eq!(pairs[0].0, "SAI_ROUTE_ENTRY_ATTR_NEXT_HOP_ID");
            assert_eq!(
                pairs[0].1,
                crate::record::ExpandedValue::Text("oid:0x5678".to_string())
            );
        } else {
            panic!("expected KeyValue for Attributes");
        }
    }

    #[test]
    fn test_expanded_remove_no_attrs() {
        let p = SairedisParser::new();
        let r = p
            .parse(
                "2025-01-15.10:30:45.123456|r|SAI_OBJECT_TYPE_ROUTE_ENTRY:oid:0x1234",
                "test",
                "loader",
                1,
            )
            .unwrap();
        let expanded = r.expanded.as_ref().unwrap();
        assert_eq!(expanded[0].label, "Operation");
        assert_eq!(expanded[1].label, "Object Type");
        assert_eq!(expanded[2].label, "OID");
        // No Attributes field
        assert_eq!(expanded.len(), 3);
    }

    #[test]
    fn test_expanded_get_response_stateful() {
        let p = SairedisParser::new();
        // First parse a 'g' to set context
        p.parse(
            "2025-01-15.10:30:45.123456|g|SAI_OBJECT_TYPE_SWITCH:oid:0xABCD|SAI_SWITCH_ATTR_SRC_MAC_ADDRESS=",
            "test",
            "loader",
            1,
        );
        // Then parse 'G' response
        let r = p
            .parse(
                "2025-01-15.10:30:45.123457|G|SAI_STATUS_SUCCESS|SAI_SWITCH_ATTR_SRC_MAC_ADDRESS=00:11:22:33:44:55",
                "test",
                "loader",
                2,
            )
            .unwrap();
        let expanded = r.expanded.as_ref().unwrap();
        assert_eq!(expanded[0].label, "Operation");
        assert_eq!(
            expanded[0].value,
            crate::record::ExpandedValue::Text("GetResponse".to_string())
        );
        // Should have OID from stateful context
        let has_oid = expanded.iter().any(|f| f.label == "OID");
        assert!(has_oid);
        // Should have Request Context
        let has_request_ctx = expanded.iter().any(|f| f.label == "Request Context");
        assert!(has_request_ctx);
    }

    #[test]
    fn test_expanded_notification() {
        let p = SairedisParser::new();
        let r = p
            .parse(
                "2025-01-15.10:30:45.123456|n|fdb_event|[{\"fdb_entry\":\"{}\",\"fdb_event\":\"SAI_FDB_EVENT_LEARNED\"}]|",
                "test",
                "loader",
                1,
            )
            .unwrap();
        let expanded = r.expanded.as_ref().unwrap();
        assert_eq!(expanded[0].label, "Operation");
        assert!(expanded[0].value.eq(&crate::record::ExpandedValue::Text(
            "Notification: fdb_event".to_string()
        )));
    }

    #[test]
    fn test_notify_syncd_request() {
        let p = SairedisParser::new();
        let r = p
            .parse(
                "2025-01-15.10:30:45.123456|a|INIT_VIEW",
                "test",
                "loader",
                1,
            )
            .unwrap();
        assert_eq!(r.function.as_deref(), Some("NotifySyncd"));
        assert_eq!(r.context.as_deref(), Some("INIT_VIEW"));
        assert_eq!(r.message, "INIT_VIEW");
    }

    #[test]
    fn test_notify_syncd_request_apply_view() {
        let p = SairedisParser::new();
        let r = p
            .parse(
                "2025-01-15.10:30:45.123456|a|APPLY_VIEW",
                "test",
                "loader",
                1,
            )
            .unwrap();
        assert_eq!(r.function.as_deref(), Some("NotifySyncd"));
        assert_eq!(r.context.as_deref(), Some("APPLY_VIEW"));
    }

    #[test]
    fn test_notify_syncd_response() {
        let p = SairedisParser::new();
        let r = p
            .parse(
                "2025-01-15.10:30:45.123456|A|SAI_STATUS_SUCCESS",
                "test",
                "loader",
                1,
            )
            .unwrap();
        assert_eq!(r.function.as_deref(), Some("NotifySyncdResponse"));
        assert_eq!(r.message, "SAI_STATUS_SUCCESS");
        assert!(r.context.is_none());
    }

    #[test]
    fn test_notify_syncd_response_empty() {
        let p = SairedisParser::new();
        let r = p
            .parse("2025-01-15.10:30:45.123456|A|", "test", "loader", 1)
            .unwrap();
        assert_eq!(r.function.as_deref(), Some("NotifySyncdResponse"));
        assert_eq!(r.message, "");
    }

    #[test]
    fn test_looks_like_sairedis_notify_syncd() {
        assert!(looks_like_sairedis(
            "2025-01-15.10:30:45.123456|a|INIT_VIEW"
        ));
        assert!(looks_like_sairedis(
            "2025-01-15.10:30:45.123456|A|SAI_STATUS_SUCCESS"
        ));
    }

    #[test]
    fn test_get_response_status_extraction() {
        let p = SairedisParser::new();
        // First send a get request to set context
        p.parse(
            "2025-01-15.10:30:45.123456|g|SAI_OBJECT_TYPE_PORT:oid:0x100000000000d|SAI_PORT_ATTR_HW_LANE_LIST=4",
            "test", "loader", 1,
        );
        // GetResponse: first attr is status
        let r = p.parse(
            "2025-01-15.10:30:45.123457|G|SAI_STATUS_SUCCESS|SAI_PORT_ATTR_HW_LANE_LIST=4:1,2,3,4|SAI_PORT_ATTR_SPEED=100000",
            "test", "loader", 2,
        ).unwrap();

        let expanded = r.expanded.as_ref().unwrap();
        // Should have: Operation, OID (from context), Status, Attributes, Request Context
        let status_field = expanded
            .iter()
            .find(|f| f.label == "Status")
            .expect("Status field missing");
        assert_eq!(
            status_field.value,
            crate::record::ExpandedValue::Text("SAI_STATUS_SUCCESS".to_string())
        );

        // Remaining attributes should NOT include the status
        let attrs_field = expanded
            .iter()
            .find(|f| f.label == "Attributes")
            .expect("Attributes field missing");
        if let crate::record::ExpandedValue::KeyValue(pairs) = &attrs_field.value {
            assert_eq!(pairs.len(), 2);
            assert_eq!(pairs[0].0, "SAI_PORT_ATTR_HW_LANE_LIST");
            assert_eq!(pairs[1].0, "SAI_PORT_ATTR_SPEED");
        } else {
            panic!("Expected KeyValue for Attributes");
        }
    }

    #[test]
    fn test_notify_syncd_response_inherits_context() {
        let p = SairedisParser::new();
        // First parse a NotifySyncd request to set context
        let r1 = p
            .parse(
                "2025-01-15.10:30:45.123456|a|INIT_VIEW",
                "test",
                "loader",
                1,
            )
            .unwrap();
        assert_eq!(r1.context.as_deref(), Some("INIT_VIEW"));

        // Then parse a NotifySyncdResponse - should inherit context
        let r2 = p
            .parse(
                "2025-01-15.10:30:45.123457|A|SAI_STATUS_SUCCESS",
                "test",
                "loader",
                2,
            )
            .unwrap();
        assert_eq!(r2.function.as_deref(), Some("NotifySyncdResponse"));
        assert_eq!(r2.context.as_deref(), Some("INIT_VIEW"));
        assert_eq!(r2.message, "SAI_STATUS_SUCCESS");
    }
    #[test]

    fn test_notify_syncd_response_status_extraction() {
        let p = SairedisParser::new();
        let r = p
            .parse(
                "2025-01-15.10:30:45.123456|A|SAI_STATUS_SUCCESS",
                "test",
                "loader",
                1,
            )
            .unwrap();

        let expanded = r.expanded.as_ref().unwrap();
        let status_field = expanded
            .iter()
            .find(|f| f.label == "Status")
            .expect("Status field missing");
        assert_eq!(
            status_field.value,
            crate::record::ExpandedValue::Text("SAI_STATUS_SUCCESS".to_string())
        );

        // No Attributes field since status was the only content
        assert!(expanded.iter().all(|f| f.label != "Attributes"));
    }

    #[test]
    fn test_query_response_status_extraction() {
        let p = SairedisParser::new();
        // First send a query to set context
        p.parse(
            "2025-01-15.10:30:45.123456|q|SAI_OBJECT_TYPE_PORT:oid:0x100000000000d|SAI_PORT_ATTR_SPEED=0",
            "test", "loader", 1,
        );
        // QueryResponse: first attr after query_name is status
        let r = p.parse(
            "2025-01-15.10:30:45.123457|Q|SAI_OBJECT_TYPE_PORT|SAI_STATUS_SUCCESS|SAI_PORT_ATTR_SPEED=100000",
            "test", "loader", 2,
        ).unwrap();

        let expanded = r.expanded.as_ref().unwrap();
        let status_field = expanded
            .iter()
            .find(|f| f.label == "Status")
            .expect("Status field missing");
        assert_eq!(
            status_field.value,
            crate::record::ExpandedValue::Text("SAI_STATUS_SUCCESS".to_string())
        );

        let attrs_field = expanded
            .iter()
            .find(|f| f.label == "Attributes")
            .expect("Attributes field missing");
        if let crate::record::ExpandedValue::KeyValue(pairs) = &attrs_field.value {
            assert_eq!(pairs.len(), 1);
            assert_eq!(pairs[0].0, "SAI_PORT_ATTR_SPEED");
        } else {
            panic!("Expected KeyValue for Attributes");
        }
    }

    #[test]
    fn test_get_response_failure_status_no_attrs() {
        let p = SairedisParser::new();
        p.parse(
            "2025-01-15.10:30:45.123456|g|SAI_OBJECT_TYPE_PORT:oid:0x100000000000d",
            "test",
            "loader",
            1,
        );
        // GetResponse with only status, no attributes
        let r = p
            .parse(
                "2025-01-15.10:30:45.123457|G|SAI_STATUS_FAILURE",
                "test",
                "loader",
                2,
            )
            .unwrap();

        let expanded = r.expanded.as_ref().unwrap();
        let status_field = expanded
            .iter()
            .find(|f| f.label == "Status")
            .expect("Status field missing");
        assert_eq!(
            status_field.value,
            crate::record::ExpandedValue::Text("SAI_STATUS_FAILURE".to_string())
        );

        // No Attributes field
        assert!(expanded.iter().all(|f| f.label != "Attributes"));
    }

    #[test]
    fn test_create_op_no_status_extraction() {
        // Non-response ops should NOT have Status field
        let p = SairedisParser::new();
        let r = p.parse(
            "2025-01-15.10:30:45.123456|c|SAI_OBJECT_TYPE_PORT:oid:0x100000000000d|SAI_PORT_ATTR_SPEED=100000",
            "test", "loader", 1,
        ).unwrap();

        let expanded = r.expanded.as_ref().unwrap();
        assert!(
            expanded.iter().all(|f| f.label != "Status"),
            "Non-response ops should not have Status field"
        );

        // First attribute should be in Attributes, not extracted as Status
        let attrs_field = expanded
            .iter()
            .find(|f| f.label == "Attributes")
            .expect("Attributes field missing");
        if let crate::record::ExpandedValue::KeyValue(pairs) = &attrs_field.value {
            assert_eq!(pairs[0].0, "SAI_PORT_ATTR_SPEED");
        } else {
            panic!("Expected KeyValue for Attributes");
        }
    }

    #[test]
    fn test_get_response_status_key_value_format() {
        // Status in key=value format: the value part should be extracted as status
        let p = SairedisParser::new();
        p.parse(
            "2025-01-15.10:30:45.123456|g|SAI_OBJECT_TYPE_PORT:oid:0x100000000000d",
            "test",
            "loader",
            1,
        );
        let r = p
            .parse(
                "2025-01-15.10:30:45.123457|G|SAI_STATUS=SAI_STATUS_NOT_SUPPORTED|SAI_PORT_ATTR_SPEED=100000",
                "test",
                "loader",
                2,
            )
            .unwrap();

        let expanded = r.expanded.as_ref().unwrap();
        let status_field = expanded
            .iter()
            .find(|f| f.label == "Status")
            .expect("Status field missing");
        assert_eq!(
            status_field.value,
            crate::record::ExpandedValue::Text("SAI_STATUS_NOT_SUPPORTED".to_string())
        );

        let attrs_field = expanded
            .iter()
            .find(|f| f.label == "Attributes")
            .expect("Attributes field missing");
        if let crate::record::ExpandedValue::KeyValue(pairs) = &attrs_field.value {
            assert_eq!(pairs.len(), 1);
            assert_eq!(pairs[0].0, "SAI_PORT_ATTR_SPEED");
        } else {
            panic!("Expected KeyValue for Attributes");
        }
    }

    #[test]
    fn test_get_response_component_propagation() {
        let p = SairedisParser::new();

        // Parse 'g' — saves component and context
        let g = parse(&p, "2025-05-17.18:49:14.280510|g|SAI_OBJECT_TYPE_SWITCH:oid:0x21000000000000|SAI_SWITCH_ATTR_SRC_MAC_ADDRESS=0:").unwrap();
        assert_eq!(g.component_name.as_deref(), Some("SAI_OBJECT_TYPE_SWITCH"));

        // Parse 'G' — should inherit component from last 'g'
        let gr = parse(&p, "2025-05-17.18:49:14.282097|G|SAI_STATUS_SUCCESS|SAI_SWITCH_ATTR_SRC_MAC_ADDRESS=50:6B:4B:B0:5D:80").unwrap();
        assert_eq!(gr.component_name.as_deref(), Some("SAI_OBJECT_TYPE_SWITCH"));
        assert_eq!(gr.context.as_deref(), Some("oid:0x21000000000000"));
    }

    #[test]
    fn test_get_response_component_cleared_without_prior_get() {
        let p = SairedisParser::new();

        // No prior 'g' — component should be None
        let gr = parse(
            &p,
            "2025-05-17.18:49:14.282097|G|SAI_STATUS_SUCCESS|attr=val",
        )
        .unwrap();
        assert!(gr.component_name.is_none());
    }

    #[test]
    fn test_query_response_component_cleared_without_prior_query() {
        let p = SairedisParser::new();

        // No prior 'q' — component should be None
        let qr = parse(
            &p,
            "2025-05-18.06:38:35.611000|Q|SAI_STATUS_SUCCESS|attr=val",
        )
        .unwrap();
        assert!(qr.component_name.is_none());
    }

    #[test]
    fn test_query_response_component_propagation() {
        let p = SairedisParser::new();

        // Parse 'q' — saves component and context
        let q = parse(&p, "2025-05-18.06:38:35.610696|q|SAI_OBJECT_TYPE_QUERY_ATTRIBUTE_CAPABILITY|SAI_OBJECT_TYPE_LAG:oid:0x21000000000000|SAI_LAG_ATTR_SYSTEM_PORT_AGGREGATE_ID").unwrap();
        assert_eq!(q.component_name.as_deref(), Some("SAI_OBJECT_TYPE_LAG"));

        // Parse 'Q' — should inherit component from last 'q'
        let qr = parse(&p, "2025-05-18.06:38:35.611000|Q|SAI_OBJECT_TYPE_QUERY_ATTRIBUTE_CAPABILITY|SAI_STATUS_SUCCESS").unwrap();
        assert_eq!(qr.component_name.as_deref(), Some("SAI_OBJECT_TYPE_LAG"));
    }
}
