//! Region configuration loading and data structures.

use crate::filter::expr::{self, Expr};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{instrument, warn};

/// A compiled region definition ready for matching.
#[derive(Debug, Clone)]
pub struct RegionDefinition {
    /// Unique name for this region type.
    pub name: String,
    /// Human-readable description.
    pub description: Option<String>,
    /// Compiled start point matchers.
    pub start_points: Vec<CompiledMatchPoint>,
    /// Compiled end point matchers.
    pub end_points: Vec<CompiledMatchPoint>,
    /// Metadata field names that must match between start and end.
    pub correlate: Vec<String>,
    /// Template for region name.
    pub name_template: String,
    /// Template for region description.
    pub description_template: Option<String>,
    /// Max duration between start and end (None = unlimited).
    pub timeout: Option<Duration>,
    /// Template for timeout end reason.
    pub timeout_reason: Option<String>,
}

/// A compiled match point (filter + optional regex).
#[derive(Debug, Clone)]
pub struct CompiledMatchPoint {
    /// Compiled filter expression.
    pub filter: Expr,
    /// Original filter string (for display).
    pub filter_str: String,
    /// Compiled regex for metadata extraction (applied to message field).
    pub regex: Option<Regex>,
    /// Reason template (supports `{field}` substitution from regex groups).
    pub reason: Option<String>,
}

// --- Raw YAML structures for deserialization ---

#[derive(Debug, Deserialize)]
pub struct RegionConfigFile {
    pub regions: Vec<RawRegionDef>,
}

#[derive(Debug, Deserialize)]
pub struct RawRegionDef {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub start_points: Vec<RawMatchPoint>,
    pub end_points: Vec<RawMatchPoint>,
    pub correlate: Vec<String>,
    pub template: RawTemplate,
    #[serde(default)]
    pub timeout: Option<String>,
    #[serde(default)]
    pub timeout_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawMatchPoint {
    pub filter: String,
    #[serde(default)]
    pub regex: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawTemplate {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Parse a timeout string like "30s", "5m", "1h".
pub(crate) fn parse_timeout(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty timeout".into());
    }

    let (num_str, unit) = if let Some(n) = s.strip_suffix('s') {
        (n, 1u64)
    } else if let Some(n) = s.strip_suffix('m') {
        (n, 60)
    } else if let Some(n) = s.strip_suffix('h') {
        (n, 3600)
    } else {
        return Err(format!("invalid timeout unit in '{}' (use s/m/h)", s));
    };

    let num: u64 = num_str
        .trim()
        .parse()
        .map_err(|_| format!("invalid timeout number in '{}'", s))?;
    Ok(Duration::from_secs(num * unit))
}

/// Compile a raw match point into a compiled one.
fn compile_match_point(raw: &RawMatchPoint) -> Result<CompiledMatchPoint, String> {
    let filter = expr::parse(&raw.filter).map_err(|e| format!("filter '{}': {}", raw.filter, e))?;
    let regex = match &raw.regex {
        Some(pattern) => {
            let re = Regex::new(pattern).map_err(|e| format!("regex '{}': {}", pattern, e))?;
            Some(re)
        }
        None => None,
    };
    Ok(CompiledMatchPoint {
        filter,
        filter_str: raw.filter.clone(),
        regex,
        reason: raw.reason.clone(),
    })
}

/// Compile a raw region definition.
fn compile_definition(raw: &RawRegionDef) -> Result<RegionDefinition, String> {
    let start_points: Vec<CompiledMatchPoint> = raw
        .start_points
        .iter()
        .map(compile_match_point)
        .collect::<Result<_, _>>()
        .map_err(|e| format!("region '{}' start_point: {}", raw.name, e))?;

    let end_points: Vec<CompiledMatchPoint> = raw
        .end_points
        .iter()
        .map(compile_match_point)
        .collect::<Result<_, _>>()
        .map_err(|e| format!("region '{}' end_point: {}", raw.name, e))?;

    let timeout = match &raw.timeout {
        Some(s) => {
            Some(parse_timeout(s).map_err(|e| format!("region '{}' timeout: {}", raw.name, e))?)
        }
        None => None,
    };

    Ok(RegionDefinition {
        name: raw.name.clone(),
        description: raw.description.clone(),
        start_points,
        end_points,
        correlate: raw.correlate.clone(),
        name_template: raw.template.name.clone(),
        description_template: raw.template.description.clone(),
        timeout,
        timeout_reason: raw.timeout_reason.clone(),
    })
}

/// Load region definitions from a YAML string.
pub fn load_from_str(yaml: &str) -> Result<Vec<RegionDefinition>, String> {
    let config: RegionConfigFile =
        serde_yaml::from_str(yaml).map_err(|e| format!("YAML parse error: {}", e))?;
    config.regions.iter().map(compile_definition).collect()
}

/// Load region definitions from a single YAML file.
#[instrument(skip(path), fields(path = %path.display()))]
pub fn load_from_file(path: &Path) -> Result<Vec<RegionDefinition>, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    load_from_str(&content).map_err(|e| format!("{}: {}", path.display(), e))
}

/// Load region definitions from a directory of YAML files.
pub fn load_from_dir(dir: &Path) -> Result<Vec<RegionDefinition>, String> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut defs = Vec::new();
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {}", dir.display(), e))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .map(|ext| ext == "yaml" || ext == "yml")
                .unwrap_or(false)
        })
        .collect();
    entries.sort();

    for path in &entries {
        match load_from_file(path) {
            Ok(mut file_defs) => defs.append(&mut file_defs),
            Err(e) => {
                eprintln!("Warning: skipping region config {}: {}", path.display(), e);
            }
        }
    }

    Ok(defs)
}

/// Built-in region definitions embedded at compile time.
static BUILTIN_REGION_CONFIGS: &[&str] = &[
    include_str!("../../../../src/scouty-config/sonic/sonic-port-operations.yaml"),
];

/// Load all region definitions from built-in presets + standard config locations.
/// Built-in → System (/etc/scouty/regions/) → User (~/.scouty/regions/) → Project (./scouty-regions/).
#[instrument]
pub fn load_all() -> Vec<RegionDefinition> {
    let mut defs = Vec::new();

    // Load built-in region definitions first (lowest precedence).
    for yaml in BUILTIN_REGION_CONFIGS {
        match load_from_str(yaml) {
            Ok(mut builtin_defs) => defs.append(&mut builtin_defs),
            Err(e) => {
                warn!("Failed to load built-in region config: {}", e);
            }
        }
    }

    let dirs = [
        PathBuf::from("/etc/scouty/regions"),
        dirs::home_dir()
            .map(|h| h.join(".scouty/regions"))
            .unwrap_or_default(),
        PathBuf::from("./scouty-regions"),
    ];

    for dir in &dirs {
        if dir.as_os_str().is_empty() {
            continue;
        }
        if let Ok(mut d) = load_from_dir(dir) {
            defs.append(&mut d);
        }
    }

    defs
}

/// Render a template string with metadata values.
/// `{field}` is replaced with the value from metadata, or left as-is if missing.
pub fn render_template(template: &str, metadata: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in metadata {
        result = result.replace(&format!("{{{}}}", key), value);
    }
    result
}
