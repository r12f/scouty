#[cfg(test)]
mod tests {
    use crate::parser::json_parser::JsonParser;
    use crate::record::{ExpandedField, ExpandedValue, LogLevel};
    use crate::traits::LogParser;

    fn parse(line: &str) -> Option<crate::record::LogRecord> {
        let parser = JsonParser::new();
        parser.parse(line, "test.log", "loader-1", 1)
    }

    #[test]
    fn test_basic_json_log() {
        let line = r#"{"timestamp":"2026-01-15T10:30:00Z","level":"INFO","message":"hello world"}"#;
        let rec = parse(line).unwrap();
        assert_eq!(rec.level, Some(LogLevel::Info));
        assert_eq!(rec.message, "hello world");
        assert_eq!(rec.timestamp.to_rfc3339(), "2026-01-15T10:30:00+00:00");
    }

    #[test]
    fn test_well_known_field_variants() {
        let line = r#"{"ts":"2026-01-15T10:30:00Z","severity":"ERROR","msg":"fail","host":"web01","service":"api","pid":1234,"tid":5678}"#;
        let rec = parse(line).unwrap();
        assert_eq!(rec.level, Some(LogLevel::Error));
        assert_eq!(rec.message, "fail");
        assert_eq!(rec.hostname.as_deref(), Some("web01"));
        assert_eq!(rec.component_name.as_deref(), Some("api"));
        assert_eq!(rec.pid, Some(1234));
        assert_eq!(rec.tid, Some(5678));
    }

    #[test]
    fn test_non_json_returns_none() {
        assert!(parse("Nov 24 10:30:00 host proc: msg").is_none());
        assert!(parse("just plain text").is_none());
        assert!(parse("").is_none());
    }

    #[test]
    fn test_invalid_json_returns_none() {
        assert!(parse("{invalid json}").is_none());
        assert!(parse(r#"{"key": }"#).is_none());
    }

    #[test]
    fn test_expanded_excludes_mapped_fields() {
        let line = r#"{"timestamp":"2026-01-15T10:30:00Z","level":"INFO","message":"hi","extra":"data","nested":{"a":1}}"#;
        let rec = parse(line).unwrap();
        let expanded = rec.expanded.as_ref().unwrap();
        assert_eq!(expanded.len(), 1);
        assert_eq!(expanded[0].label, "Payload");
        if let ExpandedValue::KeyValue(pairs) = &expanded[0].value {
            let keys: Vec<&str> = pairs.iter().map(|(k, _)| k.as_str()).collect();
            assert!(keys.contains(&"extra"));
            assert!(keys.contains(&"nested"));
            // Well-known fields should NOT be in expanded
            assert!(!keys.contains(&"timestamp"));
            assert!(!keys.contains(&"level"));
            assert!(!keys.contains(&"message"));
        } else {
            panic!("expected KeyValue");
        }
    }

    #[test]
    fn test_nested_objects_and_arrays() {
        let line = r#"{"message":"test","data":{"items":[1,2,3],"flag":true}}"#;
        let rec = parse(line).unwrap();
        let expanded = rec.expanded.as_ref().unwrap();
        if let ExpandedValue::KeyValue(pairs) = &expanded[0].value {
            assert_eq!(pairs[0].0, "data");
            if let ExpandedValue::KeyValue(inner) = &pairs[0].1 {
                // BTreeMap sorts alphabetically: flag before items
                let keys: Vec<&str> = inner.iter().map(|(k, _)| k.as_str()).collect();
                assert!(keys.contains(&"items"));
                assert!(keys.contains(&"flag"));
                let items_val = &inner.iter().find(|(k, _)| k == "items").unwrap().1;
                assert!(matches!(items_val, ExpandedValue::List(_)));
                let flag_val = &inner.iter().find(|(k, _)| k == "flag").unwrap().1;
                assert_eq!(*flag_val, ExpandedValue::Text("true".to_string()));
            } else {
                panic!("expected nested KeyValue");
            }
        } else {
            panic!("expected KeyValue");
        }
    }

    #[test]
    fn test_unix_timestamp_seconds() {
        let line = r#"{"timestamp":1737000000,"message":"unix"}"#;
        let rec = parse(line).unwrap();
        assert_eq!(rec.timestamp.timestamp(), 1737000000);
    }

    #[test]
    fn test_unix_timestamp_millis() {
        let line = r#"{"timestamp":1737000000000,"message":"unix ms"}"#;
        let rec = parse(line).unwrap();
        assert_eq!(rec.timestamp.timestamp(), 1737000000);
    }

    #[test]
    fn test_metadata_for_unknown_fields() {
        let line = r#"{"message":"hi","custom_field":"value","count":42}"#;
        let rec = parse(line).unwrap();
        let meta = rec.metadata.as_ref().unwrap();
        assert_eq!(meta.get("custom_field").unwrap(), "value");
        assert_eq!(meta.get("count").unwrap(), "42");
    }

    #[test]
    fn test_no_expanded_when_only_mapped_fields() {
        let line = r#"{"timestamp":"2026-01-15T10:30:00Z","level":"INFO","message":"hi"}"#;
        let rec = parse(line).unwrap();
        assert!(rec.expanded.is_none());
    }

    #[test]
    fn test_parser_name() {
        let parser = JsonParser::new();
        assert_eq!(parser.name(), "json");
    }

    #[test]
    fn test_case_insensitive_field_names() {
        let line = r#"{"Timestamp":"2026-01-15T10:30:00Z","Level":"WARN","Message":"warning"}"#;
        let rec = parse(line).unwrap();
        assert_eq!(rec.level, Some(LogLevel::Warn));
        assert_eq!(rec.message, "warning");
    }

    #[test]
    fn test_at_timestamp() {
        let line = r#"{"@timestamp":"2026-01-15T10:30:00Z","message":"elk style"}"#;
        let rec = parse(line).unwrap();
        assert_eq!(rec.timestamp.to_rfc3339(), "2026-01-15T10:30:00+00:00");
    }

    #[test]
    fn test_looks_like_json() {
        use crate::parser::json_parser::looks_like_json;
        assert!(looks_like_json(r#"{"key":"value"}"#));
        assert!(!looks_like_json("plain text"));
        assert!(!looks_like_json("{unclosed"));
        assert!(looks_like_json(r#"  {"key":"value"}  "#));
    }
}
