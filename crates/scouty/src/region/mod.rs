//! Region parsing — configurable start/end matching with correlation.

#[cfg(test)]
#[path = "region_tests.rs"]
mod region_tests;

pub mod config;
pub mod processor;

use std::collections::HashMap;

/// A detected region span linking a start and end log record.
#[derive(Debug, Clone)]
pub struct Region {
    /// Region definition name (e.g., "port_startup").
    pub definition_name: String,
    /// Rendered region name from template (e.g., "Port Startup Ethernet0").
    pub name: String,
    /// Rendered description from template.
    pub description: Option<String>,
    /// Rendered start point reason (e.g., "port add requested").
    pub start_reason: Option<String>,
    /// Rendered end point reason (e.g., "oper up").
    pub end_reason: Option<String>,
    /// LogStore index of the start record.
    pub start_index: usize,
    /// LogStore index of the end record.
    pub end_index: usize,
    /// Merged metadata from start + end.
    pub metadata: HashMap<String, String>,
}
