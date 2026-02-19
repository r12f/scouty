//! YAML config loading for parser definitions.
//!
//! Loads parser group definitions from YAML files. Each file can define
//! multiple parser groups, each containing multiple regex parsers.
//!
//! # Example YAML
//!
//! ```yaml
//! groups:
//!   - name: syslog
//!     parsers:
//!       - name: bsd-syslog
//!         pattern: '^(?P<timestamp>\w{3}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2})\s+(?P<process>\S+)\s+(?P<component>\S+?)(?:\[(?P<pid>\d+)\])?:\s+(?P<message>.*)'
//!         timestamp_format: "%b %d %H:%M:%S"
//!   - name: generic
//!     parsers:
//!       - name: iso-level
//!         pattern: '^(?P<timestamp>\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2})\s+(?P<level>\w+)\s+(?P<message>.*)'
//! ```

#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;

use crate::parser::group::ParserGroup;
use crate::parser::regex_parser::RegexParser;
use serde::Deserialize;
use std::path::Path;

/// Top-level config: a list of parser groups.
#[derive(Debug, Deserialize)]
pub struct ParserConfig {
    pub groups: Vec<ParserGroupDef>,
}

/// Definition of a single parser group.
#[derive(Debug, Deserialize)]
pub struct ParserGroupDef {
    pub name: String,
    pub parsers: Vec<ParserDef>,
}

/// Definition of a single regex parser within a group.
#[derive(Debug, Deserialize)]
pub struct ParserDef {
    pub name: String,
    pub pattern: String,
    #[serde(default)]
    pub timestamp_format: Option<String>,
}

/// Load parser config from a YAML string.
pub fn from_yaml(yaml: &str) -> Result<ParserConfig, String> {
    serde_yaml::from_str(yaml).map_err(|e| format!("Failed to parse YAML config: {}", e))
}

/// Load parser config from a YAML file.
pub fn from_file(path: &Path) -> Result<ParserConfig, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    from_yaml(&content)
}

/// Build parser groups from a config.
pub fn build_groups(config: &ParserConfig) -> Result<Vec<ParserGroup>, String> {
    let mut groups = Vec::new();

    for group_def in &config.groups {
        let mut group = ParserGroup::new(&group_def.name);

        for parser_def in &group_def.parsers {
            let parser = RegexParser::new(
                &parser_def.name,
                &parser_def.pattern,
                parser_def.timestamp_format.clone(),
            )
            .map_err(|e| {
                format!(
                    "Invalid regex in parser '{}' of group '{}': {}",
                    parser_def.name, group_def.name, e
                )
            })?;
            group.add_parser(Box::new(parser));
        }

        groups.push(group);
    }

    Ok(groups)
}

/// Convenience: load and build parser groups from a YAML string.
pub fn load_from_yaml(yaml: &str) -> Result<Vec<ParserGroup>, String> {
    let config = from_yaml(yaml)?;
    build_groups(&config)
}

/// Convenience: load and build parser groups from a YAML file.
pub fn load_from_file(path: &Path) -> Result<Vec<ParserGroup>, String> {
    let config = from_file(path)?;
    build_groups(&config)
}
