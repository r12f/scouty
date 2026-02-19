#[cfg(test)]
mod tests {
    use crate::parser::regex_parser::RegexParser;
    use crate::record::LogLevel;
    use crate::traits::LogParser;

    #[test]
    fn test_basic_parse() {
        let parser = RegexParser::new(
            "basic",
            r"(?P<timestamp>\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) (?P<level>\w+) (?P<message>.*)",
            None,
        )
        .unwrap();

        let record = parser
            .parse("2024-01-15 10:30:00 INFO Hello world", "test.log", "loader1", 0)
            .unwrap();
        assert_eq!(record.level, Some(LogLevel::Info));
        assert_eq!(record.message, "Hello world");
        assert_eq!(record.timestamp.to_string(), "2024-01-15 10:30:00 UTC");
    }

    #[test]
    fn test_no_match_returns_none() {
        let parser = RegexParser::new(
            "strict",
            r"^(?P<timestamp>\d{4}-\d{2}-\d{2}) (?P<message>.*)",
            None,
        )
        .unwrap();

        assert!(parser.parse("no date here", "test.log", "loader1", 0).is_none());
    }

    #[test]
    fn test_custom_timestamp_format() {
        let parser = RegexParser::new(
            "custom-ts",
            r"(?P<timestamp>\d{2}/\w{3}/\d{4}:\d{2}:\d{2}:\d{2}) (?P<message>.*)",
            Some("%d/%b/%Y:%H:%M:%S".to_string()),
        )
        .unwrap();

        let record = parser
            .parse("15/Jan/2024:10:30:00 request ok", "access.log", "loader1", 1)
            .unwrap();
        assert_eq!(record.message, "request ok");
    }

    #[test]
    fn test_extra_named_groups_go_to_metadata() {
        let parser = RegexParser::new(
            "with-extra",
            r"(?P<timestamp>\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) \[(?P<thread>\w+)\] (?P<message>.*)",
            None,
        )
        .unwrap();

        let record = parser
            .parse("2024-01-15 10:30:00 [main] Starting up", "app.log", "loader1", 2)
            .unwrap();
        assert_eq!(record.metadata.get("thread").unwrap(), "main");
    }

    #[test]
    fn test_pid_tid_component() {
        let parser = RegexParser::new(
            "full",
            r"(?P<timestamp>\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) (?P<pid>\d+):(?P<tid>\d+) \[(?P<component>\w+)\] (?P<level>\w+) (?P<message>.*)",
            None,
        )
        .unwrap();

        let record = parser
            .parse("2024-01-15 10:30:00 1234:5678 [networking] ERROR connection failed", "sys.log", "loader1", 3)
            .unwrap();
        assert_eq!(record.pid, Some(1234));
        assert_eq!(record.tid, Some(5678));
        assert_eq!(record.component_name.as_deref(), Some("networking"));
        assert_eq!(record.level, Some(LogLevel::Error));
    }

    #[test]
    fn test_name() {
        let parser = RegexParser::new("my-parser", r"(?P<message>.*)", None).unwrap();
        assert_eq!(parser.name(), "my-parser");
    }
}
