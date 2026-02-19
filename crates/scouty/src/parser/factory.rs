//! Parser factory — auto-selects or builds a parser group based on loader info.

#[cfg(test)]
#[path = "factory_tests.rs"]
mod factory_tests;

use crate::parser::group::ParserGroup;
use crate::parser::regex_parser::RegexParser;
use crate::traits::{LoaderInfo, LoaderType};

/// Built-in parser definitions that the factory can produce.
pub struct ParserFactory;

impl ParserFactory {
    /// Create a default parser group for the given loader info.
    ///
    /// Uses the loader type and sample lines to pick appropriate parsers.
    /// Returns a parser group with a fallback chain.
    pub fn create_parser_group(info: &LoaderInfo) -> ParserGroup {
        let mut group = ParserGroup::new(format!("auto:{}", info.id));

        match info.loader_type {
            LoaderType::Syslog => {
                // Syslog-specific parsers
                Self::add_syslog_parsers(&mut group);
            }
            LoaderType::Otlp => {
                // OTLP records are structured — future phase
            }
            LoaderType::TextFile | LoaderType::Archive => {
                // Try to auto-detect from sample lines
                if Self::looks_like_syslog(&info.sample_lines) {
                    Self::add_syslog_parsers(&mut group);
                }
                // Add common log format parsers
                Self::add_common_parsers(&mut group);
            }
        }

        // Always add a catch-all parser as final fallback
        Self::add_fallback_parser(&mut group);

        group
    }

    fn looks_like_syslog(sample_lines: &[String]) -> bool {
        // Simple heuristic: check if lines start with month abbreviation (e.g. "Jan 15")
        let syslog_re = regex::Regex::new(
            r"^(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)\s+\d{1,2}\s+\d{2}:\d{2}:\d{2}",
        )
        .unwrap();
        sample_lines.iter().take(5).any(|l| syslog_re.is_match(l))
    }

    fn add_syslog_parsers(group: &mut ParserGroup) {
        // BSD syslog format: "Jan 15 10:30:00 hostname process[pid]: message"
        if let Ok(p) = RegexParser::new(
            "syslog-bsd",
            r"^(?P<timestamp>\w{3}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2})\s+(?P<process>\S+)\s+(?P<component>\S+?)(?:\[(?P<pid>\d+)\])?:\s+(?P<message>.*)",
            Some("%b %d %H:%M:%S".to_string()),
        ) {
            group.add_parser(Box::new(p));
        }
    }

    fn add_common_parsers(group: &mut ParserGroup) {
        // ISO timestamp + level + message: "2024-01-15 10:30:00 INFO message"
        if let Ok(p) = RegexParser::new(
            "iso-level-msg",
            r"^(?P<timestamp>\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:?\d{2})?)\s+(?P<level>TRACE|DEBUG|INFO|WARN(?:ING)?|ERROR|FATAL|CRITICAL)\s+(?P<message>.*)",
            None,
        ) {
            group.add_parser(Box::new(p));
        }

        // ISO timestamp + bracketed level: "2024-01-15 10:30:00 [INFO] message"
        if let Ok(p) = RegexParser::new(
            "iso-bracket-level-msg",
            r"^(?P<timestamp>\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:?\d{2})?)\s+\[(?P<level>\w+)\]\s+(?P<message>.*)",
            None,
        ) {
            group.add_parser(Box::new(p));
        }

        // Level first: "INFO 2024-01-15 10:30:00 message"
        if let Ok(p) = RegexParser::new(
            "level-iso-msg",
            r"^(?P<level>TRACE|DEBUG|INFO|WARN(?:ING)?|ERROR|FATAL|CRITICAL)\s+(?P<timestamp>\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:?\d{2})?)\s+(?P<message>.*)",
            None,
        ) {
            group.add_parser(Box::new(p));
        }
    }

    fn add_fallback_parser(group: &mut ParserGroup) {
        // Catch-all: entire line as message
        if let Ok(p) = RegexParser::new("fallback", r"(?P<message>.+)", None) {
            group.add_parser(Box::new(p));
        }
    }
}
