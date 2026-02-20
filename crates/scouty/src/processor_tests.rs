#[cfg(test)]
mod tests {
    use crate::parser::group::ParserGroup;
    use crate::processor::{CountingProcessor, NoOpProcessor};
    use crate::record::{LogLevel, LogRecord};
    use crate::session::LogSession;
    use crate::traits::{LoaderInfo, LoaderType, LogLoader, LogParser, LogProcessor, Result};
    use chrono::Utc;
    use std::collections::HashMap;

    fn make_record(id: u64, level: LogLevel, message: &str) -> LogRecord {
        LogRecord {
            id,
            timestamp: Utc::now(),
            level: Some(level),
            source: "test".into(),
            pid: None,
            tid: None,
            component_name: None,
            process_name: None,
            message: message.into(),
            raw: message.into(),
            metadata: None,
            loader_id: "test-loader".into(),
        }
    }

    #[test]
    fn test_noop_processor() {
        let processor = NoOpProcessor::new("noop");
        assert_eq!(processor.name(), "noop");
        let records = vec![make_record(0, LogLevel::Info, "test")];
        assert!(processor.process(&records).is_ok());
    }

    #[test]
    fn test_noop_empty_records() {
        let processor = NoOpProcessor::new("noop");
        assert!(processor.process(&[]).is_ok());
    }

    #[test]
    fn test_counting_processor() {
        let processor = CountingProcessor::new("counter");
        assert_eq!(processor.name(), "counter");
        assert!(processor.process(&[]).is_ok());
    }

    #[test]
    fn test_count_by_level() {
        let records = vec![
            make_record(0, LogLevel::Info, "a"),
            make_record(1, LogLevel::Error, "b"),
            make_record(2, LogLevel::Info, "c"),
            make_record(3, LogLevel::Error, "d"),
            make_record(4, LogLevel::Error, "e"),
        ];
        let counts = CountingProcessor::count_by_level(&records);
        assert_eq!(counts[&Some(LogLevel::Info)], 2);
        assert_eq!(counts[&Some(LogLevel::Error)], 3);
    }

    #[test]
    fn test_processor_in_session_pipeline() {
        let mut session = LogSession::new();

        #[derive(Debug)]
        struct MockLoader {
            info: LoaderInfo,
        }
        impl LogLoader for MockLoader {
            fn info(&self) -> &LoaderInfo {
                &self.info
            }
            fn load(&mut self) -> Result<Vec<String>> {
                Ok(vec!["line1".into(), "line2".into()])
            }
        }

        #[derive(Debug)]
        struct EchoParser;
        impl LogParser for EchoParser {
            fn parse(
                &self,
                raw: &str,
                _source: &str,
                _loader_id: &str,
                id: u64,
            ) -> Option<LogRecord> {
                Some(make_record(id, LogLevel::Info, raw))
            }
            fn name(&self) -> &str {
                "echo"
            }
        }

        let loader = MockLoader {
            info: LoaderInfo {
                id: "mock".into(),
                loader_type: LoaderType::TextFile,
                multiline_enabled: false,
                sample_lines: vec![],
            },
        };

        let mut group = ParserGroup::new("test");
        group.add_parser(Box::new(EchoParser));

        session.add_loader(Box::new(loader), group);
        session.add_processor(Box::new(NoOpProcessor::new("pipeline-noop")));

        let filtered = session.run().unwrap();
        assert_eq!(filtered.len(), 2);
        assert_eq!(session.store().len(), 2);
    }
}
