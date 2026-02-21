#[cfg(test)]
mod tests {
    use crate::parser::swss_parser::*;
    use crate::record::LogRecord;
    use std::sync::Arc;

    fn parse_line(line: &str) -> Option<LogRecord> {
        let source = Arc::from("test");
        let loader = Arc::from("test-loader");
        SwssParser::parse_shared(line, &source, &loader, 1)
    }

    #[test]
    fn test_pure_message() {
        let r = parse_line("2025-11-13.22:19:03.248563|recording started").unwrap();
        assert_eq!(r.message, "recording started");
        assert!(r.component_name.is_none());
        assert!(r.context.is_none());
        assert!(r.function.is_none());
        assert_eq!(
            r.timestamp.format("%Y-%m-%d %H:%M:%S%.6f").to_string(),
            "2025-11-13 22:19:03.248563"
        );
    }

    #[test]
    fn test_table_key_set_kv() {
        let r = parse_line("2025-11-13.22:19:35.512358|SWITCH_TABLE:switch|SET|ecmp_hash_offset:0|ecmp_hash_seed:0|fdb_aging_time:600").unwrap();
        assert_eq!(r.component_name.as_deref(), Some("SWITCH_TABLE"));
        assert_eq!(r.context.as_deref(), Some("switch"));
        assert_eq!(r.function.as_deref(), Some("SET"));
        assert_eq!(
            r.message,
            "ecmp_hash_offset:0|ecmp_hash_seed:0|fdb_aging_time:600"
        );
    }

    #[test]
    fn test_table_key_port() {
        let r = parse_line("2025-11-13.22:19:35.523199|PORT_TABLE:Ethernet248|SET|admin_status:up|alias:etp31|index:31").unwrap();
        assert_eq!(r.component_name.as_deref(), Some("PORT_TABLE"));
        assert_eq!(r.context.as_deref(), Some("Ethernet248"));
        assert_eq!(r.function.as_deref(), Some("SET"));
        assert_eq!(r.message, "admin_status:up|alias:etp31|index:31");
    }

    #[test]
    fn test_table_subkey_set() {
        let r = parse_line(
            "2025-11-13.22:19:38.096435|FLEX_COUNTER_TABLE|PG_DROP|SET|FLEX_COUNTER_STATUS:enable",
        )
        .unwrap();
        assert_eq!(r.component_name.as_deref(), Some("FLEX_COUNTER_TABLE"));
        assert_eq!(r.context.as_deref(), Some("PG_DROP"));
        assert_eq!(r.function.as_deref(), Some("SET"));
        assert_eq!(r.message, "FLEX_COUNTER_STATUS:enable");
    }

    #[test]
    fn test_route_del_ipv6() {
        let r = parse_line("2025-11-13.22:23:57.885443|ROUTE_TABLE:fd00::/80|DEL").unwrap();
        assert_eq!(r.component_name.as_deref(), Some("ROUTE_TABLE"));
        assert_eq!(r.context.as_deref(), Some("fd00::/80"));
        assert_eq!(r.function.as_deref(), Some("DEL"));
        assert_eq!(r.message, "");
    }

    #[test]
    fn test_neigh_table_key_with_colons() {
        let r = parse_line("2025-11-13.23:31:35.533798|NEIGH_TABLE:eth0:192.168.0.221|SET|neigh:00:15:5d:a6:3c:09|family:IPv4").unwrap();
        assert_eq!(r.component_name.as_deref(), Some("NEIGH_TABLE"));
        assert_eq!(r.context.as_deref(), Some("eth0:192.168.0.221"));
        assert_eq!(r.function.as_deref(), Some("SET"));
        assert_eq!(r.message, "neigh:00:15:5d:a6:3c:09|family:IPv4");
    }

    #[test]
    fn test_route_set_with_kv() {
        let r = parse_line("2025-11-13.23:29:44.847720|ROUTE_TABLE:fe80::/64|SET|protocol:kernel|nexthop:::|ifname:Bridge|weight:1").unwrap();
        assert_eq!(r.component_name.as_deref(), Some("ROUTE_TABLE"));
        assert_eq!(r.context.as_deref(), Some("fe80::/64"));
        assert_eq!(r.function.as_deref(), Some("SET"));
        assert!(r.message.contains("protocol:kernel"));
    }

    #[test]
    fn test_invalid_timestamp() {
        assert!(parse_line("not-a-timestamp|something").is_none());
        assert!(parse_line("").is_none());
        assert!(parse_line("2025").is_none());
        // Correct prefix but no pipe after timestamp
        assert!(parse_line("2025-11-13.22:19:03.248563").is_none());
        // Invalid month
        assert!(parse_line("2025-13-13.22:19:03.248563|test").is_none());
        // Wrong separators
        assert!(parse_line("2025/11/13.22:19:03.248563|test").is_none());
    }

    #[test]
    fn test_timestamp_parsing() {
        let r = parse_line("2025-11-13.22:19:03.248563|test").unwrap();
        assert_eq!(r.timestamp.year(), 2025);
        assert_eq!(r.timestamp.month(), 11);
        assert_eq!(r.timestamp.day(), 13);
        assert_eq!(r.timestamp.hour(), 22);
        assert_eq!(r.timestamp.minute(), 19);
        assert_eq!(r.timestamp.second(), 3);
    }

    #[test]
    fn test_parser_trait() {
        let parser = SwssParser::new();
        let r = parser
            .parse(
                "2025-11-13.22:19:35.512358|SWITCH_TABLE:switch|SET|k:v",
                "src",
                "loader",
                42,
            )
            .unwrap();
        assert_eq!(r.id, 42);
        assert_eq!(r.component_name.as_deref(), Some("SWITCH_TABLE"));
    }

    #[test]
    fn test_perf_parse_100k() {
        let line = "2025-11-13.22:19:35.523199|PORT_TABLE:Ethernet248|SET|admin_status:up|alias:etp31|index:31|lanes:528,529,530,531|mtu:9100|speed:400000";
        let source = Arc::from("bench");
        let loader = Arc::from("bench");
        let start = std::time::Instant::now();
        let n = 100_000;
        for i in 0..n {
            let _ = SwssParser::parse_shared(line, &source, &loader, i);
        }
        let elapsed = start.elapsed();
        let per_record_ns = elapsed.as_nanos() / n as u128;
        let throughput = n as f64 / elapsed.as_secs_f64();
        println!(
            "SWSS parser: {}ns/record, {:.0} records/sec ({} records in {:?})",
            per_record_ns, throughput, n, elapsed
        );
        // Target: >= 1M/sec in release builds
        // Debug builds with parallel tests can be ~10x slower
        assert!(
            throughput > 200_000.0,
            "Throughput {:.0}/sec below 200K/sec minimum (release target: 1M/sec)",
            throughput
        );
    }

    use chrono::{Datelike, Timelike};

    #[test]
    fn test_fractional_micros() {
        assert_eq!(parse_fractional_micros("248563"), 248563);
        assert_eq!(parse_fractional_micros("1"), 100000);
        assert_eq!(parse_fractional_micros("12"), 120000);
        assert_eq!(parse_fractional_micros("123456789"), 123456); // truncate to 6
        assert_eq!(parse_fractional_micros(""), 0); // empty
        assert_eq!(parse_fractional_micros("abc"), 0); // non-digits
        assert_eq!(parse_fractional_micros("12abc"), 120000); // partial digits
    }

    #[test]
    fn test_is_known_op() {
        assert!(is_known_op("SET"));
        assert!(is_known_op("DEL"));
        assert!(is_known_op("HSET"));
        assert!(!is_known_op("UNKNOWN"));
        assert!(!is_known_op("PG_DROP"));
    }
}
