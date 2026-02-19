#[cfg(test)]
mod tests {
    use crate::parser::config::{build_groups, from_yaml, load_from_file, load_from_yaml};
    use crate::record::LogLevel;
    use crate::traits::LogParser;

    const SAMPLE_YAML: &str = r#"
groups:
  - name: syslog
    parsers:
      - name: bsd-syslog
        pattern: '^(?P<timestamp>\w{3}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2})\s+(?P<process>\S+)\s+(?P<component>\S+?)(?:\[(?P<pid>\d+)\])?:\s+(?P<message>.*)'
        timestamp_format: "%b %d %H:%M:%S"
  - name: generic
    parsers:
      - name: iso-level
        pattern: '^(?P<timestamp>\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2})\s+(?P<level>\w+)\s+(?P<message>.*)'
      - name: fallback
        pattern: '(?P<message>.+)'
"#;

    #[test]
    fn test_parse_yaml() {
        let config = from_yaml(SAMPLE_YAML).unwrap();
        assert_eq!(config.groups.len(), 2);
        assert_eq!(config.groups[0].name, "syslog");
        assert_eq!(config.groups[0].parsers.len(), 1);
        assert_eq!(config.groups[1].name, "generic");
        assert_eq!(config.groups[1].parsers.len(), 2);
    }

    #[test]
    fn test_build_groups() {
        let config = from_yaml(SAMPLE_YAML).unwrap();
        let groups = build_groups(&config).unwrap();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].name, "syslog");
        assert_eq!(groups[1].name, "generic");
    }

    #[test]
    fn test_load_from_yaml() {
        let groups = load_from_yaml(SAMPLE_YAML).unwrap();
        assert_eq!(groups.len(), 2);

        // Test that the generic group can parse a log line
        let record = groups[1]
            .parse(
                "2024-01-15 10:30:00 ERROR something broke",
                "test",
                "loader",
                0,
            )
            .unwrap();
        assert_eq!(record.level, Some(LogLevel::Error));
        assert_eq!(record.message, "something broke");
    }

    #[test]
    fn test_invalid_yaml() {
        let result = from_yaml("not: valid: yaml: [");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_regex() {
        let yaml = r#"
groups:
  - name: bad
    parsers:
      - name: broken
        pattern: '[invalid'
"#;
        let config = from_yaml(yaml).unwrap();
        let result = build_groups(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid regex"));
    }

    #[test]
    fn test_timestamp_format_optional() {
        let yaml = r#"
groups:
  - name: simple
    parsers:
      - name: catch-all
        pattern: '(?P<message>.+)'
"#;
        let groups = load_from_yaml(yaml).unwrap();
        assert_eq!(groups.len(), 1);
        let record = groups[0].parse("hello world", "test", "loader", 0).unwrap();
        assert_eq!(record.message, "hello world");
    }

    #[test]
    fn test_load_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("parsers.yaml");
        std::fs::write(&file_path, SAMPLE_YAML).unwrap();

        let groups = load_from_file(&file_path).unwrap();
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn test_load_from_nonexistent_file() {
        let result = load_from_file(std::path::Path::new("/nonexistent/parsers.yaml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_syslog_parser_from_config() {
        let groups = load_from_yaml(SAMPLE_YAML).unwrap();
        let record = groups[0]
            .parse(
                "Jan 15 10:30:00 myhost sshd[1234]: Accepted publickey",
                "test",
                "loader",
                0,
            )
            .unwrap();
        assert_eq!(record.component_name.as_deref(), Some("sshd"));
        assert_eq!(record.pid, Some(1234));
        assert_eq!(record.message, "Accepted publickey");
    }
}
