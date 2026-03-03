//! Filter preset persistence — save/load filter sets as YAML files.

#[cfg(test)]
#[path = "filter_preset_tests.rs"]
mod filter_preset_tests;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Serializable filter preset (saved to ~/.scouty/filters/<name>.yaml).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterPreset {
    /// List of filter expressions.
    #[serde(default)]
    pub filters: Vec<FilterPresetEntry>,
    /// Active level filter (None = ALL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level_filter: Option<String>,
}

/// Single filter entry in a preset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterPresetEntry {
    /// The filter expression string.
    pub expr: String,
    /// Whether this is an exclude filter.
    #[serde(default)]
    pub exclude: bool,
}

/// Directory for filter presets.
fn presets_dir() -> PathBuf {
    super::config_dir()
        .unwrap_or_else(|| PathBuf::from(".").join(".scouty"))
        .join("filters")
}

/// Ensure the presets directory exists.
fn ensure_presets_dir() -> std::io::Result<PathBuf> {
    let dir = presets_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Save a filter preset to disk.
pub fn save_preset(name: &str, preset: &FilterPreset) -> Result<PathBuf, String> {
    let dir = ensure_presets_dir().map_err(|e| format!("Failed to create presets dir: {}", e))?;
    let path = dir.join(format!("{}.yaml", sanitize_name(name)));
    let yaml =
        serde_yaml::to_string(preset).map_err(|e| format!("Failed to serialize preset: {}", e))?;
    std::fs::write(&path, yaml).map_err(|e| format!("Failed to write preset: {}", e))?;
    Ok(path)
}

/// Load a filter preset from disk.
pub fn load_preset(name: &str) -> Result<FilterPreset, String> {
    let path = presets_dir().join(format!("{}.yaml", name));
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read preset: {}", e))?;
    serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse preset: {}", e))
}

/// List available preset names (without .yaml extension).
pub fn list_presets() -> Vec<String> {
    let dir = presets_dir();
    let mut names = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    names.push(stem.to_string());
                }
            }
        }
    }
    names.sort();
    names
}

/// Delete a preset file.
pub fn delete_preset(name: &str) -> Result<(), String> {
    let path = presets_dir().join(format!("{}.yaml", name));
    std::fs::remove_file(&path).map_err(|e| format!("Failed to delete preset: {}", e))
}

/// Sanitize preset name for use as filename.
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
