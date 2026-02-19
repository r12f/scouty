#[cfg(test)]
mod tests {
    use crate::parser::group::ParserGroup;
    use crate::parser::regex_parser::RegexParser;
    use crate::record::LogLevel;

    #[test]
    fn test_first_parser_matches() {
        let mut group = ParserGroup::new("test-group");
        group.add_parser(Box::new(
            RegexParser::new(
                "syslog",
                r"(?P<timestamp>\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) (?P<level>\w+) (?P<message>.*)",
                None,
            )
            .unwrap(),
        ));

        let record = group
            .parse("2024-01-15 10:30:00 INFO hello", "test", "loader", 0)
            .unwrap();
        assert_eq!(record.level, Some(LogLevel::Info));
    }

    #[test]
    fn test_fallback_to_second_parser() {
        let mut group = ParserGroup::new("fallback-group");
        group.add_parser(Box::new(
            RegexParser::new("strict", r"^STRICT (?P<message>.*)", None).unwrap(),
        ));
        group.add_parser(Box::new(
            RegexParser::new("loose", r"(?P<message>.+)", None).unwrap(),
        ));

        let record = group.parse("random log line", "test", "loader", 0).unwrap();
        assert_eq!(record.message, "random log line");
    }

    #[test]
    fn test_no_parser_matches() {
        let mut group = ParserGroup::new("empty-group");
        group.add_parser(Box::new(
            RegexParser::new("strict", r"^NEVER_MATCH$", None).unwrap(),
        ));

        assert!(group.parse("some log line", "test", "loader", 0).is_none());
    }

    #[test]
    fn test_empty_group() {
        let group = ParserGroup::new("empty");
        assert!(group.parse("anything", "test", "loader", 0).is_none());
    }
}
