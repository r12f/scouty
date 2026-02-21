//! Parser factory — auto-selects or builds a parser group based on loader info.

#[cfg(test)]
#[path = "factory_tests.rs"]
mod factory_tests;

use std::sync::OnceLock;

use crate::parser::extended_syslog_parser::ExtendedSyslogParser;
use crate::parser::group::ParserGroup;
use crate::parser::regex_parser::RegexParser;
use crate::parser::swss_parser::SwssParser;
use crate::parser::syslog_parser::SyslogParser;
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
                if Self::looks_like_swss(&info.sample_lines) {
                    Self::add_swss_parsers(&mut group);
                } else if Self::looks_like_extended_syslog(&info.sample_lines) {
                    Self::add_extended_syslog_parsers(&mut group);
                    Self::add_syslog_parsers(&mut group);
                } else if Self::looks_like_syslog(&info.sample_lines) {
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

    fn looks_like_extended_syslog(sample_lines: &[String]) -> bool {
        static RE: OnceLock<regex::Regex> = OnceLock::new();
        let re = RE.get_or_init(|| {
            regex::Regex::new(r"^\d{4} (?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec) ")
                .unwrap()
        });
        Self::majority_match(sample_lines, |l| re.is_match(l))
    }

    fn looks_like_syslog(sample_lines: &[String]) -> bool {
        static RE: OnceLock<regex::Regex> = OnceLock::new();
        let re = RE.get_or_init(|| {
            regex::Regex::new(
                r"^(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)\s+\d{1,2}\s+\d{2}:\d{2}:\d{2}",
            )
            .unwrap()
        });
        Self::majority_match(sample_lines, |l| re.is_match(l))
    }

    fn looks_like_swss(sample_lines: &[String]) -> bool {
        static RE: OnceLock<regex::Regex> = OnceLock::new();
        let re = RE.get_or_init(|| {
            regex::Regex::new(r"^\d{4}-\d{2}-\d{2}\.\d{2}:\d{2}:\d{2}\.\d+\|").unwrap()
        });
        Self::majority_match(sample_lines, |l| re.is_match(l))
    }

    /// Returns true if the majority of non-empty sample lines (up to 5) match the predicate.
    fn majority_match(sample_lines: &[String], pred: impl Fn(&str) -> bool) -> bool {
        let lines: Vec<&str> = sample_lines
            .iter()
            .take(5)
            .map(|l| l.as_str())
            .filter(|l| !l.trim().is_empty())
            .collect();
        if lines.is_empty() {
            return false;
        }
        let matched = lines.iter().filter(|l| pred(l)).count();
        matched * 2 > lines.len() // majority: more than half
    }

    fn add_swss_parsers(group: &mut ParserGroup) {
        group.add_parser(Box::new(SwssParser::new()));
    }

    fn add_extended_syslog_parsers(group: &mut ParserGroup) {
        group.add_parser(Box::new(ExtendedSyslogParser::new("extended-syslog")));
    }

    fn add_syslog_parsers(group: &mut ParserGroup) {
        // Zero-regex syslog parser (fast path)
        group.add_parser(Box::new(SyslogParser::new("syslog-zero-regex")));

        // BSD syslog format: "Jan 15 10:30:00 hostname process[pid]: message"
        if let Ok(p) = RegexParser::new(
            "syslog-bsd",
            r"^(?P<timestamp>\w{3}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2})\s+(?P<hostname>\S+)\s+(?P<process>\S+?)(?:\[(?P<pid>\d+)\])?:\s+(?P<message>.*)",
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
