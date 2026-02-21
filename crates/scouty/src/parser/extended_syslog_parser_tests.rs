#[cfg(test)]
mod tests {
    use crate::parser::extended_syslog_parser::ExtendedSyslogParser;
    use crate::record::LogLevel;
    use crate::traits::LogParser;

    fn parse(line: &str) -> Option<crate::record::LogRecord> {
        let parser = ExtendedSyslogParser::new("test");
        parser.parse(line, "test-source", "test-loader", 0)
    }

    #[test]
    fn parse_memory_checker() {
        let r = parse(
            "2025 Nov 24 17:56:03.073872 BSL-0101-0101-01LT0 INFO memory_checker: [memory_checker] ...",
        )
        .unwrap();
        assert_eq!(r.hostname.as_deref(), Some("BSL-0101-0101-01LT0"));
        assert_eq!(r.level, Some(LogLevel::Info));
        assert_eq!(r.process_name.as_deref(), Some("memory_checker"));
        assert_eq!(r.container, None);
        assert_eq!(r.pid, None);
        assert_eq!(r.message, "[memory_checker] ...");
        assert_eq!(
            r.timestamp.format("%Y-%m-%d %H:%M:%S%.6f").to_string(),
            "2025-11-24 17:56:03.073872"
        );
    }

    #[test]
    fn parse_container_process() {
        let r = parse(
            "2025 Nov 24 17:55:51.558366 BSL-0101-0101-01LT0 NOTICE restapi#root: message repeated ...",
        )
        .unwrap();
        assert_eq!(r.level, Some(LogLevel::Notice));
        assert_eq!(r.container.as_deref(), Some("restapi"));
        assert_eq!(r.process_name.as_deref(), Some("root"));
        assert_eq!(r.pid, None);
        assert_eq!(r.message, "message repeated ...");
    }

    #[test]
    fn parse_container_process_pid() {
        let r = parse(
            "2025 Nov 24 17:56:24.314970 BSL-0101-0101-01LT0 NOTICE pmon#stormond[37]: FSIO JSON ...",
        )
        .unwrap();
        assert_eq!(r.level, Some(LogLevel::Notice));
        assert_eq!(r.container.as_deref(), Some("pmon"));
        assert_eq!(r.process_name.as_deref(), Some("stormond"));
        assert_eq!(r.pid, Some(37));
        assert_eq!(r.message, "FSIO JSON ...");
    }

    #[test]
    fn parse_simple_process() {
        let r = parse(
            "2025 Nov 24 17:56:02.947896 BSL-0101-0101-01LT0 NOTICE python3: :- publish: ...",
        )
        .unwrap();
        assert_eq!(r.level, Some(LogLevel::Notice));
        assert_eq!(r.process_name.as_deref(), Some("python3"));
        assert_eq!(r.container, None);
        assert_eq!(r.pid, None);
        assert_eq!(r.message, ":- publish: ...");
    }

    #[test]
    fn parse_process_with_pid() {
        let r = parse(
            "2025 Nov 24 17:55:30.363194 BSL-0101-0101-01LT0 INFO dockerd[871]: time=\"...\" ...",
        )
        .unwrap();
        assert_eq!(r.level, Some(LogLevel::Info));
        assert_eq!(r.process_name.as_deref(), Some("dockerd"));
        assert_eq!(r.container, None);
        assert_eq!(r.pid, Some(871));
        assert_eq!(r.message, "time=\"...\" ...");
    }

    #[test]
    fn parse_container_script() {
        let r = parse(
            "2025 Nov 24 18:08:29.174202 BSL-0101-0101-01LT0 INFO acms#start.py: start: main: ...",
        )
        .unwrap();
        assert_eq!(r.level, Some(LogLevel::Info));
        assert_eq!(r.container.as_deref(), Some("acms"));
        assert_eq!(r.process_name.as_deref(), Some("start.py"));
        assert_eq!(r.pid, None);
        assert_eq!(r.message, "start: main: ...");
    }

    #[test]
    fn reject_too_short() {
        assert!(parse("short").is_none());
    }

    #[test]
    fn reject_invalid_month() {
        assert!(parse("2025 Xyz 24 17:56:03.073872 HOST INFO proc: msg").is_none());
    }

    #[test]
    fn parse_without_microseconds() {
        let r = parse("2025 Nov 24 17:56:03 HOST INFO proc: msg").unwrap();
        assert_eq!(r.process_name.as_deref(), Some("proc"));
        assert_eq!(r.message, "msg");
    }

    #[test]
    fn notice_is_independent_level() {
        let r = parse("2025 Nov 24 17:55:51.558366 HOST NOTICE proc: msg").unwrap();
        assert_eq!(r.level, Some(LogLevel::Notice));
        assert_ne!(r.level, Some(LogLevel::Info));
    }

    #[test]
    fn hostname_extracted() {
        let r = parse("2025 Nov 24 17:56:03.073872 my-special-host INFO proc: msg").unwrap();
        assert_eq!(r.hostname.as_deref(), Some("my-special-host"));
    }

    #[test]
    fn all_six_sample_lines_parse() {
        let lines = vec![
            "2025 Nov 24 17:56:03.073872 BSL-0101-0101-01LT0 INFO memory_checker: [memory_checker] ...",
            "2025 Nov 24 17:55:51.558366 BSL-0101-0101-01LT0 NOTICE restapi#root: message repeated ...",
            "2025 Nov 24 17:56:24.314970 BSL-0101-0101-01LT0 NOTICE pmon#stormond[37]: FSIO JSON ...",
            "2025 Nov 24 17:56:02.947896 BSL-0101-0101-01LT0 NOTICE python3: :- publish: ...",
            "2025 Nov 24 17:55:30.363194 BSL-0101-0101-01LT0 INFO dockerd[871]: time=\"...\" ...",
            "2025 Nov 24 18:08:29.174202 BSL-0101-0101-01LT0 INFO acms#start.py: start: main: ...",
        ];
        for line in lines {
            assert!(parse(line).is_some(), "Failed to parse: {}", line);
        }
    }
}
