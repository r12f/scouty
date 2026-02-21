//! Parser factory — auto-selects or builds a parser group based on loader info.

#[cfg(test)]
#[path = "factory_tests.rs"]
mod factory_tests;

use crate::parser::group::ParserGroup;
use crate::parser::regex_parser::RegexParser;
use crate::parser::swss_parser::SwssParser;
use crate::parser::unified_syslog_parser::UnifiedSyslogParser;
use crate::traits::{LoaderInfo, LoaderType};
use std::sync::OnceLock;

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
                Self::add_unified_syslog_parser(&mut group);
            }
            LoaderType::Otlp => {
                // OTLP records are structured — future phase
            }
            LoaderType::TextFile | LoaderType::Archive => {
                // Try to auto-detect from sample lines
                if Self::looks_like_swss(&info.sample_lines) {
                    Self::add_swss_parsers(&mut group);
                } else if Self::looks_like_syslog(&info.sample_lines) {
                    Self::add_unified_syslog_parser(&mut group);
                }
                // Add common log format parsers
                Self::add_common_parsers(&mut group);
            }
        }

        // Always add a catch-all parser as final fallback
        Self::add_fallback_parser(&mut group);

        group
    }

    /// Unified syslog detection — matches BSD, Extended, and ISO 8601 formats.
    ///
    /// Checks first bytes of each line:
    /// - `A-Z` → BSD syslog (month name)
    /// - `0-9{4} ` → Extended syslog (year + space + month)
    /// - `0-9{4}-..T` → ISO 8601 syslog (year-month-dayT...)
    fn looks_like_syslog(sample_lines: &[String]) -> bool {
        Self::majority_match(sample_lines, |l| {
            let b = l.as_bytes();
            if b.is_empty() {
                return false;
            }
            if b[0].is_ascii_uppercase() {
                // BSD: starts with 3-letter month
                b.len() >= 16 && is_bsd_month(&b[0..3])
            } else if b[0].is_ascii_digit() && b.len() >= 11 {
                if b[4] == b' ' {
                    // Extended: "YYYY Mon ..."
                    b.len() >= 20
                        && b[0..4].iter().all(|c| c.is_ascii_digit())
                        && is_bsd_month(&b[5..8])
                } else if b[4] == b'-' && b[10] == b'T' {
                    // ISO 8601: "YYYY-MM-DDT..."
                    b.len() >= 20
                        && b[0..4].iter().all(|c| c.is_ascii_digit())
                        && b[5].is_ascii_digit() && b[6].is_ascii_digit()
                        && b[7] == b'-'
                        && b[8].is_ascii_digit() && b[9].is_ascii_digit()
                } else {
                    false
                }
            } else {
                false
            }
        })
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

    fn add_unified_syslog_parser(group: &mut ParserGroup) {
        group.add_parser(Box::new(UnifiedSyslogParser::new("unified-syslog")));
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

/// Quick check for 3-letter month name.
#[inline]
fn is_bsd_month(b: &[u8]) -> bool {
    matches!(
        (b[0], b[1], b[2]),
        (b'J', b'a', b'n')
            | (b'F', b'e', b'b')
            | (b'M', b'a', b'r')
            | (b'A', b'p', b'r')
            | (b'M', b'a', b'y')
            | (b'J', b'u', b'n')
            | (b'J', b'u', b'l')
            | (b'A', b'u', b'g')
            | (b'S', b'e', b'p')
            | (b'O', b'c', b't')
            | (b'N', b'o', b'v')
            | (b'D', b'e', b'c')
    )
}
