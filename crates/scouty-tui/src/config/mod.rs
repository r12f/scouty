//! Configuration system for scouty-tui.
//!
//! Supports layered config loading:
//! 1. Built-in defaults (compiled in)
//! 2. `/etc/scouty/config.yaml` (system-wide, if exists)
//! 3. `~/.scouty/config.yaml` (per-user, if exists)
//! 4. CLI flags (`--theme`, `--config`, file arguments)

pub mod color;
pub mod theme;

pub use color::ThemeColor;
pub use theme::Theme;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Top-level configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Theme name: "default" or a custom theme file name (without .yaml).
    pub theme: String,
    /// Keybinding overrides.
    #[serde(default)]
    pub keybindings: crate::keybinding::KeybindingConfig,
    /// General settings.
    pub general: GeneralConfig,
    /// Default file paths/glob patterns when no CLI arguments are provided.
    #[serde(default)]
    pub default_paths: Vec<String>,
}

/// General settings section.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Auto-enable follow mode when reading from pipe/stdin.
    pub follow_on_pipe: bool,
    /// Detail panel height ratio (0.0 - 1.0).
    pub detail_panel_ratio: f64,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            follow_on_pipe: true,
            detail_panel_ratio: 0.3,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            keybindings: crate::keybinding::KeybindingConfig::default(),
            general: GeneralConfig::default(),
            default_paths: Vec::new(),
        }
    }
}

/// Expand `default_paths` glob patterns into concrete file paths.
/// Non-matching patterns are silently skipped.
pub fn expand_default_paths(patterns: &[String]) -> Vec<String> {
    let mut results = Vec::new();
    for pattern in patterns {
        match glob::glob(pattern) {
            Ok(paths) => {
                for entry in paths.flatten() {
                    if entry.is_file() {
                        if let Some(s) = entry.to_str() {
                            results.push(s.to_string());
                        }
                    }
                }
            }
            Err(_) => {
                // Invalid glob pattern — silently skip
            }
        }
    }
    results
}

/// Return the system-wide config directory: `/etc/scouty/`.
pub fn system_config_dir() -> PathBuf {
    PathBuf::from("/etc/scouty")
}

/// Return the scouty config directory: `~/.scouty/`.
pub fn config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".scouty"))
}

/// Deep-merge two serde_yaml Values.
/// - Maps: recursively merge (later keys override).
/// - Scalars/lists: later replaces earlier entirely.
/// - Null in overlay resets the key (removes it so default applies).
fn deep_merge(base: serde_yaml::Value, overlay: serde_yaml::Value) -> serde_yaml::Value {
    use serde_yaml::Value;
    match (base, overlay) {
        (Value::Mapping(mut base_map), Value::Mapping(over_map)) => {
            for (key, over_val) in over_map {
                if over_val.is_null() {
                    base_map.remove(&key);
                } else if let Some(base_val) = base_map.remove(&key) {
                    base_map.insert(key, deep_merge(base_val, over_val));
                } else {
                    base_map.insert(key, over_val);
                }
            }
            Value::Mapping(base_map)
        }
        (_, overlay) => overlay,
    }
}

/// Load a YAML file and return it as a Value, or None if missing/invalid.
fn load_yaml_file(path: &Path) -> Option<serde_yaml::Value> {
    let content = std::fs::read_to_string(path).ok()?;
    match serde_yaml::from_str(&content) {
        Ok(v) => Some(v),
        Err(e) => {
            eprintln!("warning: invalid config {}: {e}", path.display());
            None
        }
    }
}

/// Load config with layered merge: defaults → /etc/scouty → ~/.scouty → optional CLI path.
/// `cli_config_path` corresponds to `--config <path>`.
pub fn load_config_layered(cli_config_path: Option<&str>) -> Config {
    // Start with defaults as YAML value
    let mut merged = serde_yaml::to_value(Config::default()).unwrap_or(serde_yaml::Value::Null);

    // Layer 2: system-wide
    let sys_path = system_config_dir().join("config.yaml");
    if let Some(sys_val) = load_yaml_file(&sys_path) {
        merged = deep_merge(merged, sys_val);
    }

    // Layer 3: per-user
    if let Some(dir) = config_dir() {
        let user_path = dir.join("config.yaml");
        if let Some(user_val) = load_yaml_file(&user_path) {
            merged = deep_merge(merged, user_val);
        }
    }

    // Layer 4: CLI --config override
    if let Some(cli_path) = cli_config_path {
        let path = Path::new(cli_path);
        if let Some(cli_val) = load_yaml_file(path) {
            merged = deep_merge(merged, cli_val);
        } else if !path.exists() {
            eprintln!("warning: config file not found: {cli_path}");
        }
    }

    // Deserialize merged value into Config
    match serde_yaml::from_value(merged) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("warning: failed to parse merged config: {e}");
            Config::default()
        }
    }
}

/// Load config from `~/.scouty/config.yaml`. Returns defaults if file missing or invalid.
/// Convenience wrapper that uses layered loading.
pub fn load_config() -> Config {
    load_config_layered(None)
}

/// Resolve the theme based on config and optional CLI override.
pub fn resolve_theme(config: &Config, cli_theme: Option<&str>) -> Theme {
    let theme_name = cli_theme.unwrap_or(&config.theme);

    if theme_name == "default" {
        return Theme::default();
    }

    // Built-in theme presets
    match theme_name {
        "dark" => return Theme::dark(),
        "light" => return Theme::light(),
        "solarized" => return Theme::solarized(),
        _ => {}
    }

    // Check built-in presets first
    if let Some(theme) = Theme::builtin(theme_name) {
        return theme;
    }

    // Try loading from ~/.scouty/themes/<name>.yaml, then /etc/scouty/themes/<name>.yaml
    let theme_dirs: Vec<PathBuf> = {
        let mut dirs = Vec::new();
        if let Some(dir) = config_dir() {
            dirs.push(dir.join("themes"));
        }
        dirs.push(system_config_dir().join("themes"));
        dirs
    };

    for dir in &theme_dirs {
        let theme_path = dir.join(format!("{theme_name}.yaml"));
        match std::fs::read_to_string(&theme_path) {
            Ok(content) => match Theme::from_yaml(&content) {
                Ok(theme) => return theme,
                Err(e) => {
                    eprintln!("warning: invalid theme file {}: {e}", theme_path.display());
                }
            },
            Err(_) => continue,
        }
    }

    eprintln!("warning: theme '{}' not found, using default", theme_name);
    Theme::default()
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;
