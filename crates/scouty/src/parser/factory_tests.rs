#[cfg(test)]
mod tests {
    use crate::parser::factory::ParserFactory;
    use crate::record::LogLevel;
    use crate::traits::{LoaderInfo, LoaderType};

    fn text_loader_info(sample_lines: Vec<String>) -> LoaderInfo {
        LoaderInfo {
            id: "test-loader".to_string(),
            loader_type: LoaderType::TextFile,
            multiline_enabled: false,
            sample_lines,
            file_mod_year: None,
        }
    }

    #[test]
    fn test_auto_group_parses_iso_level_msg() {
        let info = text_loader_info(vec!["2024-01-15 10:30:00 INFO Starting service".to_string()]);
        let group = ParserFactory::create_parser_group(&info);

        let record = group
            .parse(
                "2024-01-15 10:30:00 ERROR Something failed",
                "test",
                "loader",
                0,
            )
            .unwrap();
        assert_eq!(record.level, Some(LogLevel::Error));
        assert_eq!(record.message, "Something failed");
    }

    #[test]
    fn test_auto_group_parses_bracket_level() {
        let info = text_loader_info(vec![]);
        let group = ParserFactory::create_parser_group(&info);

        let record = group
            .parse(
                "2024-01-15T10:30:00 [WARN] disk space low",
                "test",
                "loader",
                1,
            )
            .unwrap();
        assert_eq!(record.level, Some(LogLevel::Warn));
        assert_eq!(record.message, "disk space low");
    }

    #[test]
    fn test_fallback_catches_unstructured() {
        let info = text_loader_info(vec![]);
        let group = ParserFactory::create_parser_group(&info);

        let record = group
            .parse("just a random line", "test", "loader", 2)
            .unwrap();
        assert_eq!(record.message, "just a random line");
        assert_eq!(record.level, None);
    }

    #[test]
    fn test_syslog_detection() {
        let info = text_loader_info(vec![
            "Jan 15 10:30:00 myhost sshd[1234]: Accepted publickey".to_string(),
        ]);
        let group = ParserFactory::create_parser_group(&info);

        let record = group
            .parse(
                "Jan 15 10:30:00 myhost sshd[1234]: Accepted publickey",
                "test",
                "loader",
                3,
            )
            .unwrap();
        assert_eq!(record.process_name.as_deref(), Some("sshd"));
        assert_eq!(record.pid, Some(1234));
        assert_eq!(record.hostname.as_deref(), Some("myhost"));
    }

    #[test]
    fn test_syslog_loader_type() {
        let info = LoaderInfo {
            id: "syslog-loader".to_string(),
            loader_type: LoaderType::Syslog,
            multiline_enabled: false,
            sample_lines: vec![],
            file_mod_year: None,
        };
        let group = ParserFactory::create_parser_group(&info);
        assert!(group.parsers.len() >= 2);
    }

    #[test]
    fn test_extended_syslog_detection() {
        let info = text_loader_info(vec![
            "2025 Nov 24 17:56:03.073872 BSL-0101 INFO memory_checker: Total memory usage"
                .to_string(),
        ]);
        let group = ParserFactory::create_parser_group(&info);

        let record = group
            .parse(
                "2025 Nov 24 17:56:03.073872 BSL-0101 INFO memory_checker: Total memory usage",
                "test",
                "loader",
                4,
            )
            .unwrap();
        assert_eq!(record.hostname.as_deref(), Some("BSL-0101"));
        assert_eq!(record.process_name.as_deref(), Some("memory_checker"));
        assert_eq!(record.message, "Total memory usage");
    }

    #[test]
    fn test_extended_syslog_with_container() {
        let info = text_loader_info(vec![
            "2025 Nov 24 17:55:51.558366 BSL-0101 NOTICE restapi#root: message repeated 47 times"
                .to_string(),
        ]);
        let group = ParserFactory::create_parser_group(&info);

        let record = group
            .parse(
                "2025 Nov 24 17:55:51.558366 BSL-0101 NOTICE restapi#root: message repeated 47 times",
                "test",
                "loader",
                5,
            )
            .unwrap();
        assert_eq!(record.hostname.as_deref(), Some("BSL-0101"));
        assert_eq!(record.container.as_deref(), Some("restapi"));
        assert_eq!(record.process_name.as_deref(), Some("root"));
        assert_eq!(record.message, "message repeated 47 times");
    }

    #[test]
    fn test_extended_syslog_does_not_match_bsd() {
        // BSD syslog should not trigger extended detection
        let info = text_loader_info(vec![
            "Jan 15 10:30:00 myhost sshd[1234]: Accepted publickey".to_string(),
        ]);
        let group = ParserFactory::create_parser_group(&info);

        // Should still parse as BSD syslog
        let record = group
            .parse(
                "Jan 15 10:30:00 myhost sshd[1234]: Accepted publickey",
                "test",
                "loader",
                6,
            )
            .unwrap();
        assert_eq!(record.hostname.as_deref(), Some("myhost"));
        assert_eq!(record.process_name.as_deref(), Some("sshd"));
    }

    #[test]
    fn test_swss_auto_detection() {
        let info = text_loader_info(vec![
            "2025-11-13.22:19:03.248563|recording started".to_string(),
            "2025-11-13.22:19:35.512358|SWITCH_TABLE:switch|SET|k:v".to_string(),
            "2025-11-13.22:19:38.096435|FLEX_COUNTER_TABLE|PG_DROP|SET|v:1".to_string(),
        ]);
        let group = ParserFactory::create_parser_group(&info);
        let record = group
            .parse(
                "2025-11-13.22:19:35.512358|SWITCH_TABLE:switch|SET|ecmp_hash_offset:0",
                "test",
                "loader",
                1,
            )
            .unwrap();
        assert_eq!(record.component_name.as_deref(), Some("SWITCH_TABLE"));
        assert_eq!(record.context.as_deref(), Some("switch"));
        assert_eq!(record.function.as_deref(), Some("SET"));
    }

    #[test]
    fn test_swss_does_not_match_syslog() {
        // Syslog lines should not trigger SWSS detection
        let info = text_loader_info(vec![
            "Jan 15 10:30:00 myhost sshd[1234]: Accepted publickey".to_string(),
        ]);
        let group = ParserFactory::create_parser_group(&info);
        let record = group
            .parse(
                "Jan 15 10:30:00 myhost sshd[1234]: Accepted publickey",
                "test",
                "loader",
                1,
            )
            .unwrap();
        // Should parse as syslog, not SWSS
        assert_eq!(record.hostname.as_deref(), Some("myhost"));
        assert!(record.context.is_none());
    }

    #[test]
    fn test_swss_does_not_match_extended_syslog() {
        let info = text_loader_info(vec![
            "2025 Jan 15 10:30:00.123456 BSL-0101 NOTICE restapi#root: test message".to_string(),
        ]);
        let group = ParserFactory::create_parser_group(&info);
        let record = group
            .parse(
                "2025 Jan 15 10:30:00.123456 BSL-0101 NOTICE restapi#root: test message",
                "test",
                "loader",
                1,
            )
            .unwrap();
        assert_eq!(record.hostname.as_deref(), Some("BSL-0101"));
        assert!(record.context.is_none());
    }

    #[test]
    fn test_iso_syslog_auto_detection() {
        let info = text_loader_info(vec![
            "2026-02-15T00:00:08.954827-08:00 r12f-ms01 systemd[1]: rsyslog.service: Sent signal SIGHUP".to_string(),
            "2026-02-15T00:00:08.955061-08:00 r12f-ms01 rsyslogd: rsyslogd was HUPed".to_string(),
        ]);
        let group = ParserFactory::create_parser_group(&info);
        let record = group
            .parse(
                "2026-02-15T00:00:08.954827-08:00 r12f-ms01 systemd[1]: test message",
                "test",
                "loader",
                1,
            )
            .unwrap();
        assert_eq!(record.hostname.as_deref(), Some("r12f-ms01"));
        assert_eq!(record.process_name.as_deref(), Some("systemd"));
        assert_eq!(record.pid, Some(1));
        assert_eq!(record.message, "test message");
    }

    #[test]
    fn test_iso_syslog_does_not_match_swss() {
        let info = text_loader_info(vec![
            "2025-11-13.22:19:03.248563|recording started".to_string(),
            "2025-11-13.22:19:35.512358|SWITCH_TABLE:switch|SET|k:v".to_string(),
        ]);
        let group = ParserFactory::create_parser_group(&info);
        let record = group
            .parse(
                "2025-11-13.22:19:35.512358|SWITCH_TABLE:switch|SET|k:v",
                "test",
                "loader",
                1,
            )
            .unwrap();
        assert_eq!(record.component_name.as_deref(), Some("SWITCH_TABLE"));
    }
}
