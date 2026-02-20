#[cfg(test)]
mod tests {
    use crate::filter::eval::eval;
    use crate::filter::expr::parse;
    use crate::record::{LogLevel, LogRecord};
    use chrono::Utc;
    use std::collections::HashMap;

    fn make_record(level: LogLevel, message: &str, component: Option<&str>) -> LogRecord {
        let mut metadata = HashMap::new();
        metadata.insert("env".to_string(), "prod".to_string());
        LogRecord {
            id: 42,
            timestamp: Utc::now(),
            level: Some(level),
            source: "test-source".into(),
            pid: Some(1234),
            tid: None,
            component_name: component.map(|s| s.to_string()),
            process_name: None,
            message: message.into(),
            raw: message.into(),
            metadata: Some(metadata),
            loader_id: "loader-1".into(),
        }
    }

    #[test]
    fn eval_eq_level() {
        let expr = parse(r#"level = "ERROR""#).unwrap();
        let r = make_record(LogLevel::Error, "boom", None);
        assert!(eval(&expr, &r));
    }

    #[test]
    fn eval_eq_level_no_match() {
        let expr = parse(r#"level = "ERROR""#).unwrap();
        let r = make_record(LogLevel::Info, "ok", None);
        assert!(!eval(&expr, &r));
    }

    #[test]
    fn eval_ne() {
        let expr = parse(r#"level != "DEBUG""#).unwrap();
        let r = make_record(LogLevel::Error, "boom", None);
        assert!(eval(&expr, &r));
    }

    #[test]
    fn eval_contains() {
        let expr = parse(r#"message contains "error""#).unwrap();
        let r = make_record(LogLevel::Error, "an error occurred", None);
        assert!(eval(&expr, &r));
    }

    #[test]
    fn eval_starts_with() {
        let expr = parse(r#"message starts_with "an""#).unwrap();
        let r = make_record(LogLevel::Info, "an event", None);
        assert!(eval(&expr, &r));
    }

    #[test]
    fn eval_ends_with() {
        let expr = parse(r#"message ends_with "occurred""#).unwrap();
        let r = make_record(LogLevel::Error, "error occurred", None);
        assert!(eval(&expr, &r));
    }

    #[test]
    fn eval_regex() {
        let expr = parse(r#"message regex "err(or)?\s+\d+""#).unwrap();
        let r = make_record(LogLevel::Error, "error 404", None);
        assert!(eval(&expr, &r));
    }

    #[test]
    fn eval_numeric_comparison() {
        let expr = parse("id >= 10").unwrap();
        let r = make_record(LogLevel::Info, "ok", None);
        assert!(eval(&expr, &r)); // id=42 >= 10
    }

    #[test]
    fn eval_numeric_lt() {
        let expr = parse("id < 10").unwrap();
        let r = make_record(LogLevel::Info, "ok", None);
        assert!(!eval(&expr, &r)); // id=42 < 10 is false
    }

    #[test]
    fn eval_and() {
        let expr = parse(r#"level = "ERROR" AND component = "auth""#).unwrap();
        let r = make_record(LogLevel::Error, "fail", Some("auth"));
        assert!(eval(&expr, &r));
    }

    #[test]
    fn eval_and_one_fails() {
        let expr = parse(r#"level = "ERROR" AND component = "db""#).unwrap();
        let r = make_record(LogLevel::Error, "fail", Some("auth"));
        assert!(!eval(&expr, &r));
    }

    #[test]
    fn eval_or() {
        let expr = parse(r#"level = "ERROR" OR level = "FATAL""#).unwrap();
        let r = make_record(LogLevel::Error, "fail", None);
        assert!(eval(&expr, &r));
    }

    #[test]
    fn eval_not() {
        let expr = parse(r#"NOT level = "DEBUG""#).unwrap();
        let r = make_record(LogLevel::Error, "fail", None);
        assert!(eval(&expr, &r));
    }

    #[test]
    fn eval_complex_nested() {
        let expr = parse(r#"(level = "ERROR" OR level = "FATAL") AND component = "auth""#).unwrap();
        let r = make_record(LogLevel::Error, "fail", Some("auth"));
        assert!(eval(&expr, &r));

        let r2 = make_record(LogLevel::Info, "ok", Some("auth"));
        assert!(!eval(&expr, &r2));
    }

    #[test]
    fn eval_metadata_field() {
        let expr = parse(r#"metadata.env = "prod""#).unwrap();
        let r = make_record(LogLevel::Info, "ok", None);
        assert!(eval(&expr, &r));
    }

    #[test]
    fn eval_missing_field_returns_false() {
        let expr = parse(r#"component = "auth""#).unwrap();
        let r = make_record(LogLevel::Info, "ok", None); // component is None
        assert!(!eval(&expr, &r));
    }

    #[test]
    fn eval_pid() {
        let expr = parse("pid = 1234").unwrap();
        let r = make_record(LogLevel::Info, "ok", None);
        assert!(eval(&expr, &r));
    }

    #[test]
    fn eval_source() {
        let expr = parse(r#"source = "test-source""#).unwrap();
        let r = make_record(LogLevel::Info, "ok", None);
        assert!(eval(&expr, &r));
    }
}
