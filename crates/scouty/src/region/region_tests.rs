//! Tests for region config loading.

mod tests {
    use crate::region::config::*;
    use std::collections::HashMap;

    const SAMPLE_CONFIG: &str = r#"
regions:
  - name: "port_startup"
    description: "Port initialization to oper up"
    start_points:
      - filter: 'message contains "addPort"'
        regex: '(?P<port>Ethernet\d+)'
    end_points:
      - filter: 'message contains "oper_status.*up"'
        regex: '(?P<port>Ethernet\d+).*oper_status.*(?P<status>up|down)'
    correlate:
      - "port"
    template:
      name: "Port Startup {port}"
      description: "{port} startup → {status}"
    timeout: "30s"

  - name: "http_request"
    start_points:
      - filter: 'message contains "request started"'
        regex: 'request_id=(?P<req_id>[a-f0-9-]+)'
    end_points:
      - filter: 'message contains "request completed"'
        regex: 'request_id=(?P<req_id>[a-f0-9-]+).*status=(?P<status>\d+)'
    correlate:
      - "req_id"
    template:
      name: "{req_id}"
"#;

    #[test]
    fn test_load_basic_config() {
        let defs = load_from_str(SAMPLE_CONFIG).unwrap();
        assert_eq!(defs.len(), 2);

        assert_eq!(defs[0].name, "port_startup");
        assert_eq!(
            defs[0].description.as_deref(),
            Some("Port initialization to oper up")
        );
        assert_eq!(defs[0].start_points.len(), 1);
        assert_eq!(defs[0].end_points.len(), 1);
        assert_eq!(defs[0].correlate, vec!["port"]);
        assert_eq!(defs[0].name_template, "Port Startup {port}");
        assert_eq!(defs[0].timeout.unwrap(), std::time::Duration::from_secs(30));

        assert_eq!(defs[1].name, "http_request");
        assert!(defs[1].description.is_none());
        assert!(defs[1].timeout.is_none());
    }

    #[test]
    fn test_filter_compiled() {
        let defs = load_from_str(SAMPLE_CONFIG).unwrap();
        // Should not panic — filters are compiled at load time
        assert_eq!(
            defs[0].start_points[0].filter_str,
            "message contains \"addPort\""
        );
    }

    #[test]
    fn test_regex_compiled() {
        let defs = load_from_str(SAMPLE_CONFIG).unwrap();
        let re = defs[0].start_points[0].regex.as_ref().unwrap();
        let caps = re.captures("addPort Ethernet0 speed 100G").unwrap();
        assert_eq!(&caps["port"], "Ethernet0");
    }

    #[test]
    fn test_no_regex() {
        let yaml = r#"
regions:
  - name: "simple"
    start_points:
      - filter: 'level == "ERROR"'
    end_points:
      - filter: 'level == "INFO"'
    correlate: []
    template:
      name: "Error Block"
"#;
        let defs = load_from_str(yaml).unwrap();
        assert!(defs[0].start_points[0].regex.is_none());
    }

    #[test]
    fn test_invalid_filter() {
        let yaml = r#"
regions:
  - name: "bad"
    start_points:
      - filter: 'invalid $$$ filter'
    end_points:
      - filter: 'level == "INFO"'
    correlate: []
    template:
      name: "Bad"
"#;
        let result = load_from_str(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("start_point"));
    }

    #[test]
    fn test_invalid_regex() {
        let yaml = r#"
regions:
  - name: "bad_regex"
    start_points:
      - filter: 'level == "ERROR"'
        regex: '(?P<unclosed'
    end_points:
      - filter: 'level == "INFO"'
    correlate: []
    template:
      name: "Bad"
"#;
        let result = load_from_str(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("regex"));
    }

    #[test]
    fn test_parse_timeout() {
        use std::time::Duration;
        assert_eq!(parse_timeout("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_timeout("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_timeout("1h").unwrap(), Duration::from_secs(3600));
        assert!(parse_timeout("").is_err());
        assert!(parse_timeout("10x").is_err());
        assert!(parse_timeout("abc").is_err());
    }

    #[test]
    fn test_render_template() {
        let mut meta = HashMap::new();
        meta.insert("port".into(), "Ethernet0".into());
        meta.insert("status".into(), "up".into());

        assert_eq!(
            render_template("Port Startup {port}", &meta),
            "Port Startup Ethernet0"
        );
        assert_eq!(
            render_template("{port} → {status}", &meta),
            "Ethernet0 → up"
        );
        // Missing field left as-is
        assert_eq!(
            render_template("{unknown} {port}", &meta),
            "{unknown} Ethernet0"
        );
    }

    #[test]
    fn test_load_from_nonexistent_dir() {
        let result = load_from_dir(std::path::Path::new("/nonexistent/path"));
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_multiple_start_end_points() {
        let yaml = r#"
regions:
  - name: "multi"
    start_points:
      - filter: 'message contains "start1"'
        regex: '(?P<id>\d+)'
      - filter: 'message contains "start2"'
        regex: '(?P<id>\d+)'
    end_points:
      - filter: 'message contains "end1"'
        regex: '(?P<id>\d+)'
      - filter: 'message contains "end2"'
        regex: '(?P<id>\d+)'
    correlate:
      - "id"
    template:
      name: "Region {id}"
"#;
        let defs = load_from_str(yaml).unwrap();
        assert_eq!(defs[0].start_points.len(), 2);
        assert_eq!(defs[0].end_points.len(), 2);
    }
}
