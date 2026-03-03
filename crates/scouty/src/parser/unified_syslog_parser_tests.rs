#[cfg(test)]
mod tests {
    use crate::parser::unified_syslog_parser::UnifiedSyslogParser;
    use crate::record::{LogLevel, LogRecord};
    use chrono::{Datelike, Timelike};
    use std::sync::Arc;

    fn parse(line: &str) -> Option<LogRecord> {
        let parser = UnifiedSyslogParser::new_with_year("test", 2026);
        let src = Arc::from("test");
        let loader = Arc::from("test-loader");
        parser.parse_shared(line, &src, &loader, 1)
    }

    // ── BSD format ──────────────────────────────────────────────────────

    #[test]
    fn bsd_basic() {
        let r = parse("Feb 19 14:23:45 myhost myapp[12345]: This is a log message").unwrap();
        assert_eq!(r.hostname.as_deref(), Some("myhost"));
        assert_eq!(r.process_name.as_deref(), Some("myapp"));
        assert_eq!(r.pid, Some(12345));
        assert_eq!(r.message, "This is a log message");
        assert!(r.level.is_none());
        assert!(r.container.is_none());
    }

    #[test]
    fn bsd_no_pid() {
        let r = parse("Jan  5 03:00:00 server sshd: Connection closed").unwrap();
        assert_eq!(r.hostname.as_deref(), Some("server"));
        assert_eq!(r.process_name.as_deref(), Some("sshd"));
        assert!(r.pid.is_none());
        assert_eq!(r.message, "Connection closed");
    }

    #[test]
    fn bsd_single_digit_day() {
        let r = parse("Dec  1 00:00:00 host proc[1]: msg").unwrap();
        assert_eq!(r.timestamp.day(), 1);
    }

    #[test]
    fn bsd_timestamp() {
        let r = parse("Nov 24 17:56:03 host proc[1]: test").unwrap();
        assert_eq!(r.timestamp.month(), 11);
        assert_eq!(r.timestamp.day(), 24);
        assert_eq!(r.timestamp.hour(), 17);
        assert_eq!(r.timestamp.minute(), 56);
        assert_eq!(r.timestamp.second(), 3);
    }

    // ── Extended format ─────────────────────────────────────────────────

    #[test]
    fn extended_basic() {
        let r = parse(
            "2025 Nov 24 17:56:03.073872 BSL-0101 NOTICE restapi#root: message repeated 47 times",
        )
        .unwrap();
        assert_eq!(r.hostname.as_deref(), Some("BSL-0101"));
        assert_eq!(r.container.as_deref(), Some("restapi"));
        assert_eq!(r.process_name.as_deref(), Some("root"));
        assert_eq!(r.level, Some(LogLevel::Notice));
        assert_eq!(r.message, "message repeated 47 times");
    }

    #[test]
    fn extended_with_pid() {
        let r = parse("2025 Nov 24 17:56:03.073872 myhost INFO pmon#stormond[37]: storm detected")
            .unwrap();
        assert_eq!(r.container.as_deref(), Some("pmon"));
        assert_eq!(r.process_name.as_deref(), Some("stormond"));
        assert_eq!(r.pid, Some(37));
        assert_eq!(r.level, Some(LogLevel::Info));
    }

    #[test]
    fn extended_no_container() {
        let r = parse("2025 Jan 15 10:30:00.000000 host WARNING dockerd[871]: oom kill").unwrap();
        assert!(r.container.is_none());
        assert_eq!(r.process_name.as_deref(), Some("dockerd"));
        assert_eq!(r.pid, Some(871));
        assert_eq!(r.level, Some(LogLevel::Warn));
    }

    #[test]
    fn extended_no_container_no_pid() {
        let r = parse("2025 Jan 15 10:30:00.000000 host ERR memory_checker: low mem").unwrap();
        assert!(r.container.is_none());
        assert_eq!(r.process_name.as_deref(), Some("memory_checker"));
        assert!(r.pid.is_none());
        assert_eq!(r.level, Some(LogLevel::Error));
    }

    #[test]
    fn extended_microseconds() {
        let r = parse("2025 Nov 24 17:56:03.123456 host INFO proc: msg").unwrap();
        assert_eq!(r.timestamp.timestamp_subsec_micros(), 123456);
    }

    #[test]
    fn extended_single_digit_day() {
        let r = parse("2025 Jan  5 10:30:00.000000 host INFO proc: msg").unwrap();
        assert_eq!(r.timestamp.day(), 5);
    }

    // ── ISO 8601 format ─────────────────────────────────────────────────

    #[test]
    fn iso_basic() {
        let r = parse("2026-02-15T00:00:08.954827-08:00 r12f-ms01 systemd[1]: rsyslog.service: Sent signal SIGHUP").unwrap();
        assert_eq!(r.hostname.as_deref(), Some("r12f-ms01"));
        assert_eq!(r.process_name.as_deref(), Some("systemd"));
        assert_eq!(r.pid, Some(1));
        assert!(r.message.starts_with("rsyslog.service:"));
        assert!(r.level.is_none());
    }

    #[test]
    fn iso_utc_z() {
        let r = parse("2026-02-15T08:00:08.954827Z myhost proc[99]: test msg").unwrap();
        assert_eq!(r.hostname.as_deref(), Some("myhost"));
        assert_eq!(r.pid, Some(99));
        assert_eq!(r.timestamp.hour(), 8);
    }

    #[test]
    fn iso_positive_offset() {
        let r = parse("2026-02-15T17:00:00.000000+09:00 host proc[1]: msg").unwrap();
        // 17:00 +09:00 = 08:00 UTC
        assert_eq!(r.timestamp.hour(), 8);
    }

    #[test]
    fn iso_negative_offset() {
        let r = parse("2026-02-15T00:00:08.954827-08:00 host proc[1]: msg").unwrap();
        // 00:00:08 -08:00 = 08:00:08 UTC
        assert_eq!(r.timestamp.hour(), 8);
        assert_eq!(r.timestamp.second(), 8);
    }

    #[test]
    fn iso_no_fractional() {
        let r = parse("2026-02-15T08:00:00Z host proc: msg").unwrap();
        assert_eq!(r.message, "msg");
        assert_eq!(r.timestamp.timestamp_subsec_micros(), 0);
    }

    #[test]
    fn iso_no_pid() {
        let r = parse("2026-02-15T08:00:00.000000-08:00 host rsyslogd: HUPed").unwrap();
        assert_eq!(r.process_name.as_deref(), Some("rsyslogd"));
        assert!(r.pid.is_none());
    }

    // ── Dual-timestamp format ────────────────────────────────────────────

    #[test]
    fn dual_timestamp_basic() {
        let r = parse("2026-03-03 06:54:06 2026-03-01T00:00:39.241739-08:00 r12f-ms01 node[4152382]: Node.js v22.22.0").unwrap();
        assert_eq!(r.hostname.as_deref(), Some("r12f-ms01"));
        assert_eq!(r.process_name.as_deref(), Some("node"));
        assert_eq!(r.pid, Some(4152382));
        assert_eq!(r.message, "Node.js v22.22.0");
        // Should use the ISO timestamp (not the prepended one)
        assert_eq!(r.timestamp.hour(), 8); // 00:00:39 -08:00 = 08:00:39 UTC
        assert_eq!(r.timestamp.minute(), 0);
        assert_eq!(r.timestamp.second(), 39);
    }

    #[test]
    fn dual_timestamp_systemd() {
        let r = parse("2026-03-03 06:54:06 2026-02-15T00:00:08.954827-08:00 r12f-ms01 systemd[1]: rsyslog.service: Sent signal SIGHUP").unwrap();
        assert_eq!(r.hostname.as_deref(), Some("r12f-ms01"));
        assert_eq!(r.process_name.as_deref(), Some("systemd"));
        assert_eq!(r.pid, Some(1));
        assert!(r.message.starts_with("rsyslog.service:"));
    }

    #[test]
    fn dual_timestamp_no_pid() {
        let r = parse("2026-03-03 06:54:06 2026-03-01T10:30:00.000000Z myhost rsyslogd: HUPed")
            .unwrap();
        assert_eq!(r.process_name.as_deref(), Some("rsyslogd"));
        assert!(r.pid.is_none());
        assert_eq!(r.message, "HUPed");
    }

    #[test]
    fn perf_dual_timestamp() {
        perf_test(
            "DualTS",
            "2026-03-03 06:54:06 2026-03-01T00:00:39.241739-08:00 r12f-ms01 node[4152382]: Node.js v22.22.0",
        );
    }

    // ── Rejection tests ─────────────────────────────────────────────────

    #[test]
    fn reject_swss() {
        // SWSS has '.' at position 10, not 'T'
        assert!(parse("2025-11-13.22:19:03.248563|recording started").is_none());
    }

    #[test]
    fn reject_short() {
        assert!(parse("").is_none());
        assert!(parse("short").is_none());
    }

    #[test]
    fn reject_invalid() {
        assert!(parse("hello world this is not syslog").is_none());
        assert!(parse("12345 not a valid line").is_none());
    }

    // ── Parity with old parsers ─────────────────────────────────────────

    #[test]
    fn bsd_parity_with_old_syslog_parser() {
        // These come from the existing syslog_parser_tests
        let r = parse("Feb 19 14:23:45 myhost myapp[12345]: This is a log message").unwrap();
        assert_eq!(r.hostname.as_deref(), Some("myhost"));
        assert_eq!(r.process_name.as_deref(), Some("myapp"));
        assert_eq!(r.pid, Some(12345));
        assert_eq!(r.message, "This is a log message");
    }

    #[test]
    fn extended_parity_with_old_extended_parser() {
        let r = parse(
            "2025 Nov 24 17:56:03.073872 BSL-0101 NOTICE restapi#root: message repeated 47 times",
        )
        .unwrap();
        assert_eq!(r.hostname.as_deref(), Some("BSL-0101"));
        assert_eq!(r.container.as_deref(), Some("restapi"));
        assert_eq!(r.process_name.as_deref(), Some("root"));
        assert_eq!(r.level, Some(LogLevel::Notice));
        assert_eq!(r.message, "message repeated 47 times");
    }

    // ── Performance ─────────────────────────────────────────────────────

    fn perf_test(label: &str, line: &str) {
        let parser = UnifiedSyslogParser::new_with_year("bench", 2026);
        let src = Arc::from("bench");
        let loader = Arc::from("bench");
        let n = 100_000u64;
        let start = std::time::Instant::now();
        for i in 0..n {
            let _ = parser.parse_shared(line, &src, &loader, i);
        }
        let elapsed = start.elapsed();
        let per_ns = elapsed.as_nanos() / n as u128;
        let throughput = n as f64 / elapsed.as_secs_f64();
        println!(
            "Unified {} : {}ns/record, {:.1}M/sec ({:?})",
            label,
            per_ns,
            throughput / 1_000_000.0,
            elapsed
        );
        // Debug build threshold: 200K/sec (release target: 10M/sec)
        // CI runners (especially macOS/Windows) may be slower than local dev.
        assert!(
            throughput > 100_000.0,
            "{} throughput {:.0}/sec below minimum",
            label,
            throughput
        );
    }

    #[test]
    fn perf_bsd() {
        perf_test(
            "BSD",
            "Feb 19 14:23:45 myhost myapp[12345]: This is a log message with some content",
        );
    }

    #[test]
    fn perf_extended() {
        perf_test(
            "Extended",
            "2025 Nov 24 17:56:03.073872 BSL-0101 NOTICE restapi#root: message repeated 47 times with extra content",
        );
    }

    #[test]
    fn perf_iso() {
        perf_test(
            "ISO",
            "2026-02-15T00:00:08.954827-08:00 r12f-ms01 systemd[1]: rsyslog.service: Sent signal SIGHUP to main process 1181",
        );
    }

    // ── LogParser trait ─────────────────────────────────────────────────

    #[test]
    fn trait_parse() {
        use crate::traits::LogParser;
        let parser = UnifiedSyslogParser::new_with_year("test", 2026);
        let r = parser
            .parse(
                "Feb 19 14:23:45 myhost myapp[12345]: msg",
                "src",
                "loader",
                42,
            )
            .unwrap();
        assert_eq!(r.id, 42);
        assert_eq!(r.process_name.as_deref(), Some("myapp"));
    }

    // ── Malformed input ─────────────────────────────────────────────────

    #[test]
    fn reject_malformed_pid() {
        // PID contains non-digit characters — should parse but return pid=None
        let r = parse("Feb 19 14:23:45 myhost myapp[abc]: msg").unwrap();
        assert_eq!(r.process_name.as_deref(), Some("myapp"));
        assert!(r.pid.is_none());
    }

    #[test]
    fn reject_malformed_day_bsd() {
        assert!(parse("Feb X9 14:23:45 host proc[1]: msg").is_none());
    }

    #[test]
    fn reject_malformed_day_extended() {
        assert!(parse("2025 Nov X5 10:30:00.000000 host INFO proc: msg").is_none());
    }

    #[test]
    fn bsd_year_from_constructor() {
        let parser = UnifiedSyslogParser::new_with_year("test", 2020);
        let src = Arc::from("test");
        let loader = Arc::from("test-loader");
        let r = parser
            .parse_shared("Feb 19 14:23:45 myhost myapp[12345]: msg", &src, &loader, 1)
            .unwrap();
        assert_eq!(r.timestamp.year(), 2020);
    }

    #[test]
    fn bsd_default_year_is_current() {
        let parser = UnifiedSyslogParser::new("test");
        let src = Arc::from("test");
        let loader = Arc::from("test-loader");
        let r = parser
            .parse_shared("Feb 19 14:23:45 myhost myapp[12345]: msg", &src, &loader, 1)
            .unwrap();
        assert_eq!(r.timestamp.year(), chrono::Utc::now().year());
    }
}
