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
        assert_eq!(record.component_name.as_deref(), Some("sshd"));
        assert_eq!(record.pid, Some(1234));
    }

    #[test]
    fn test_syslog_loader_type() {
        let info = LoaderInfo {
            id: "syslog-loader".to_string(),
            loader_type: LoaderType::Syslog,
            multiline_enabled: false,
            sample_lines: vec![],
        };
        let group = ParserFactory::create_parser_group(&info);
        assert!(group.parsers.len() >= 2);
    }
}
