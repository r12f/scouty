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
        assert!(gr.component_name.is_none());
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
        // Threshold conservative for CI debug builds on slow runners
        assert!(rate > 100_000.0, "Too slow: {:.0} rec/sec", rate);
    }
}
