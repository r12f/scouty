#[cfg(test)]
mod tests {
    use crate::parser::syslog_parser::SyslogParser;
    use crate::traits::LogParser;
    use chrono::{Datelike, Timelike};
    use std::sync::Arc;

    fn make_parser() -> SyslogParser {
        SyslogParser::new("test-syslog")
    }

    #[test]
    fn test_basic_syslog_line() {
        let parser = make_parser();
        let line = "Feb 19 14:23:45 myhost myapp[12345]: This is a log message";
        let record = parser
            .parse(line, "test-source", "test-loader", 1)
            .expect("should parse");

        assert_eq!(record.id, 1);
        assert_eq!(record.process_name.as_deref(), Some("myapp"));
        assert_eq!(record.hostname.as_deref(), Some("myhost"));
        assert_eq!(record.pid, Some(12345));
        assert_eq!(record.message, "This is a log message");
        assert_eq!(record.timestamp.month(), 2);
        assert_eq!(record.timestamp.day(), 19);
        assert_eq!(record.timestamp.hour(), 14);
        assert_eq!(record.timestamp.minute(), 23);
        assert_eq!(record.timestamp.second(), 45);
    }

    #[test]
    fn test_single_digit_day() {
        let parser = make_parser();
        let line = "Jan  5 03:10:22 server sshd[999]: Accepted publickey";
        let record = parser.parse(line, "s", "l", 0).expect("should parse");

        assert_eq!(record.timestamp.day(), 5);
        assert_eq!(record.timestamp.month(), 1);
        assert_eq!(record.process_name.as_deref(), Some("sshd"));
        assert_eq!(record.pid, Some(999));
        assert_eq!(record.message, "Accepted publickey");
    }

    #[test]
    fn test_no_pid() {
        let parser = make_parser();
        // Some syslog lines have no PID brackets
        let line = "Mar 12 09:00:00 host kernel: CPU0: Temperature above threshold";
        let record = parser.parse(line, "s", "l", 0).expect("should parse");

        assert_eq!(record.process_name.as_deref(), Some("kernel"));
        assert_eq!(record.pid, None);
        assert_eq!(record.message, "CPU0: Temperature above threshold");
    }

    #[test]
    fn test_all_months() {
        let parser = make_parser();
        let months = [
            ("Jan", 1),
            ("Feb", 2),
            ("Mar", 3),
            ("Apr", 4),
            ("May", 5),
            ("Jun", 6),
            ("Jul", 7),
            ("Aug", 8),
            ("Sep", 9),
            ("Oct", 10),
            ("Nov", 11),
            ("Dec", 12),
        ];
        for (name, num) in &months {
            let line = format!("{} 15 12:00:00 host app[1]: msg", name);
            let record = parser.parse(&line, "s", "l", 0).expect("should parse");
            assert_eq!(record.timestamp.month(), *num, "failed for {}", name);
        }
    }

    #[test]
    fn test_invalid_month() {
        let parser = make_parser();
        let line = "Xyz 15 12:00:00 host app[1]: msg";
        assert!(parser.parse(line, "s", "l", 0).is_none());
    }

    #[test]
    fn test_too_short() {
        let parser = make_parser();
        assert!(parser.parse("short", "s", "l", 0).is_none());
        assert!(parser.parse("", "s", "l", 0).is_none());
    }

    #[test]
    fn test_shared_arc_reuse() {
        let parser = make_parser();
        let source: Arc<str> = Arc::from("shared-source");
        let loader: Arc<str> = Arc::from("shared-loader");

        let line1 = "Feb 19 14:23:45 h1 app[1]: msg1";
        let line2 = "Feb 19 14:23:46 h2 app[2]: msg2";

        let r1 = parser
            .parse_shared(line1, &source, &loader, 1)
            .expect("parse1");
        let r2 = parser
            .parse_shared(line2, &source, &loader, 2)
            .expect("parse2");

        // Verify Arc is shared (same pointer)
        assert!(Arc::ptr_eq(&r1.source, &r2.source));
        assert!(Arc::ptr_eq(&r1.loader_id, &r2.loader_id));
    }

    #[test]
    fn test_parse_shared_owned() {
        let parser = make_parser();
        let source: Arc<str> = Arc::from("s");
        let loader: Arc<str> = Arc::from("l");
        let line = "Feb 19 14:23:45 host app[42]: hello world".to_string();

        let record = parser
            .parse_shared_owned(line, &source, &loader, 0)
            .expect("should parse");
        assert_eq!(record.message, "hello world");
        assert_eq!(record.raw, "Feb 19 14:23:45 host app[42]: hello world");
    }

    #[test]
    fn test_batch_parse() {
        let parser = make_parser();
        let source: Arc<str> = Arc::from("s");
        let loader: Arc<str> = Arc::from("l");

        let lines = vec![
            "Feb 19 14:23:45 h app[1]: msg1",
            "invalid line",
            "Mar 10 08:00:00 h2 sshd[22]: msg2",
        ];

        let records = parser.parse_batch(&lines, &source, &loader, 100);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, 100);
        assert_eq!(records[1].id, 102);
    }

    #[test]
    fn test_large_pid() {
        let parser = make_parser();
        let line = "Feb 19 14:23:45 host app[4294967295]: msg";
        let record = parser.parse(line, "s", "l", 0).expect("should parse");
        assert_eq!(record.pid, Some(4294967295));
    }

    #[test]
    fn test_metadata_is_none() {
        let parser = make_parser();
        let line = "Feb 19 14:23:45 host app[1]: msg";
        let record = parser.parse(line, "s", "l", 0).expect("should parse");
        assert!(record.metadata.is_none());
    }

    #[test]
    fn test_message_with_colons() {
        let parser = make_parser();
        let line = "Feb 19 14:23:45 host app[1]: key: value: data";
        let record = parser.parse(line, "s", "l", 0).expect("should parse");
        assert_eq!(record.message, "key: value: data");
    }
}
