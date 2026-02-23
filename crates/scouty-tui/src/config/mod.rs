//! Configuration system for scouty-tui.
//!
//! Loads `~/.scouty/config.yaml` at startup, merging user overrides with defaults.

pub mod color;
pub mod theme;

pub use color::ThemeColor;
pub use theme::Theme;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
        }
    }
}

/// Return the scouty config directory: `~/.scouty/`.
pub fn config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".scouty"))
}

/// Load config from `~/.scouty/config.yaml`. Returns defaults if file missing or invalid.
pub fn load_config() -> Config {
    let Some(dir) = config_dir() else {
        return Config::default();
    };
    let path = dir.join("config.yaml");
    match std::fs::read_to_string(&path) {
        Ok(content) => match serde_yaml::from_str::<Config>(&content) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("warning: invalid config {}: {e}", path.display());
                Config::default()
            }
        },
        Err(_) => Config::default(),
    }
}

/// Resolve the theme based on config and optional CLI override.
pub fn resolve_theme(config: &Config, cli_theme: Option<&str>) -> Theme {
    let theme_name = cli_theme.unwrap_or(&config.theme);

    if theme_name == "default" {
        return Theme::default();
    }

    // Try loading from ~/.scouty/themes/<name>.yaml
    if let Some(dir) = config_dir() {
        let theme_path = dir.join("themes").join(format!("{theme_name}.yaml"));
        match std::fs::read_to_string(&theme_path) {
            Ok(content) => match Theme::from_yaml(&content) {
                Ok(theme) => return theme,
                Err(e) => {
                    eprintln!("warning: invalid theme file {}: {e}", theme_path.display());
                }
            },
            Err(_) => {
                eprintln!("warning: theme '{}' not found, using default", theme_name);
            }
        }
    }

    Theme::default()
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;
