//! Theme: centralized color definitions for the TUI.

use super::color::ThemeColor;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Style entry: foreground, background, bold.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StyleEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fg: Option<ThemeColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg: Option<ThemeColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
}

impl Default for StyleEntry {
    fn default() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: None,
        }
    }
}

impl StyleEntry {
    pub fn fg(color: Color) -> Self {
        Self {
            fg: Some(ThemeColor(color)),
            bg: None,
            bold: None,
        }
    }

    pub fn fg_bg(fg: Color, bg: Color) -> Self {
        Self {
            fg: Some(ThemeColor(fg)),
            bg: Some(ThemeColor(bg)),
            bold: None,
        }
    }

    pub fn fg_bold(fg: Color) -> Self {
        Self {
            fg: Some(ThemeColor(fg)),
            bg: None,
            bold: Some(true),
        }
    }

    pub fn bg(bg: Color) -> Self {
        Self {
            fg: None,
            bg: Some(ThemeColor(bg)),
            bold: None,
        }
    }

    /// Convert to ratatui Style.
    pub fn to_style(&self) -> ratatui::style::Style {
        let mut s = ratatui::style::Style::default();
        if let Some(fg) = self.fg {
            s = s.fg(fg.0);
        }
        if let Some(bg) = self.bg {
            s = s.bg(bg.0);
        }
        if self.bold == Some(true) {
            s = s.add_modifier(ratatui::style::Modifier::BOLD);
        }
        s
    }

    /// Get the fg color or a fallback.
    pub fn fg_color(&self) -> Color {
        self.fg.map(|c| c.0).unwrap_or(Color::Reset)
    }

    /// Get the bg color or a fallback.
    pub fn bg_color(&self) -> Color {
        self.bg.map(|c| c.0).unwrap_or(Color::Reset)
    }
}

/// Log level colors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LogLevelTheme {
    pub fatal: StyleEntry,
    pub error: StyleEntry,
    pub warn: StyleEntry,
    pub notice: StyleEntry,
    pub info: StyleEntry,
    pub debug: StyleEntry,
    pub trace: StyleEntry,
}

impl Default for LogLevelTheme {
    fn default() -> Self {
        Self {
            fatal: StyleEntry::fg_bold(Color::Red),
            error: StyleEntry::fg(Color::Red),
            warn: StyleEntry::fg(Color::Yellow),
            notice: StyleEntry::fg(Color::Cyan),
            info: StyleEntry::fg(Color::Green),
            debug: StyleEntry::fg(Color::Gray),
            trace: StyleEntry::fg(Color::DarkGray),
        }
    }
}

/// Table (log list) colors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TableTheme {
    pub header: StyleEntry,
    pub selected: StyleEntry,
    pub selected_search: StyleEntry,
    pub selected_highlight: StyleEntry,
    pub search_match: StyleEntry,
    pub bookmark: StyleEntry,
}

impl Default for TableTheme {
    fn default() -> Self {
        Self {
            header: StyleEntry::fg_bg(Color::White, Color::DarkGray),
            selected: StyleEntry::bg(Color::Rgb(40, 40, 60)),
            selected_search: StyleEntry::bg(Color::Rgb(120, 120, 0)),
            selected_highlight: StyleEntry::bg(Color::Rgb(40, 60, 80)),
            search_match: StyleEntry::bg(Color::Rgb(80, 80, 0)),
            bookmark: StyleEntry::bg(Color::Rgb(20, 40, 60)),
        }
    }
}

/// Status bar colors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StatusBarTheme {
    pub line1_bg: StyleEntry,
    pub line2_bg: StyleEntry,
    pub density_hot: StyleEntry,
    pub density_normal: StyleEntry,
    pub position: StyleEntry,
    pub mode_follow: StyleEntry,
    pub mode_view: StyleEntry,
    pub mode_label: StyleEntry,
    pub command_mode_label: StyleEntry,
    pub search_mode_label: StyleEntry,
    pub shortcut_key: StyleEntry,
    pub shortcut_sep: StyleEntry,
}

impl Default for StatusBarTheme {
    fn default() -> Self {
        Self {
            line1_bg: StyleEntry::bg(Color::Rgb(20, 20, 40)),
            line2_bg: StyleEntry::bg(Color::Rgb(30, 30, 30)),
            density_hot: StyleEntry::fg_bg(Color::Yellow, Color::Rgb(40, 40, 60)),
            density_normal: StyleEntry::fg(Color::Cyan),
            position: StyleEntry::fg_bg(Color::White, Color::DarkGray),
            mode_follow: StyleEntry::fg_bg(Color::Black, Color::Green),
            mode_view: StyleEntry::fg_bg(Color::Black, Color::Cyan),
            mode_label: StyleEntry::fg_bg(Color::Black, Color::Magenta),
            command_mode_label: StyleEntry::fg_bg(Color::Black, Color::Magenta),
            search_mode_label: StyleEntry::fg_bg(Color::Black, Color::Magenta),
            shortcut_key: StyleEntry::fg(Color::Yellow),
            shortcut_sep: StyleEntry::fg(Color::DarkGray),
        }
    }
}

/// Search colors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchTheme {
    pub match_highlight: StyleEntry,
    pub current_match: StyleEntry,
}

impl Default for SearchTheme {
    fn default() -> Self {
        Self {
            match_highlight: StyleEntry::fg_bg(Color::Black, Color::Yellow),
            current_match: StyleEntry::fg_bg(Color::Black, Color::Rgb(255, 102, 0)),
        }
    }
}

/// Dialog / window colors (for popups like help, filter manager, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DialogTheme {
    pub border: StyleEntry,
    pub title: StyleEntry,
    pub selected: StyleEntry,
    pub text: StyleEntry,
    pub muted: StyleEntry,
    pub background: StyleEntry,
    pub accent: StyleEntry,
}

impl Default for DialogTheme {
    fn default() -> Self {
        Self {
            border: StyleEntry::fg(Color::Cyan),
            title: StyleEntry::fg(Color::Cyan),
            selected: StyleEntry::fg_bg(Color::White, Color::DarkGray),
            text: StyleEntry::fg(Color::White),
            muted: StyleEntry::fg(Color::DarkGray),
            background: StyleEntry::bg(Color::Black),
            accent: StyleEntry::fg(Color::Yellow),
        }
    }
}

/// Detail panel colors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DetailPanelTheme {
    pub field_name: StyleEntry,
    pub field_value: StyleEntry,
    pub border: StyleEntry,
    pub section_header: StyleEntry,
}

impl Default for DetailPanelTheme {
    fn default() -> Self {
        Self {
            field_name: StyleEntry::fg(Color::Cyan),
            field_value: StyleEntry::fg(Color::White),
            border: StyleEntry::fg(Color::DarkGray),
            section_header: StyleEntry::fg(Color::Cyan),
        }
    }
}

/// Input bar colors (search, filter input).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InputTheme {
    pub prompt: StyleEntry,
    pub cursor: StyleEntry,
    pub text: StyleEntry,
    pub error: StyleEntry,
    pub background: StyleEntry,
}

impl Default for InputTheme {
    fn default() -> Self {
        Self {
            prompt: StyleEntry::fg(Color::Yellow),
            cursor: StyleEntry::fg(Color::White),
            text: StyleEntry::fg(Color::White),
            error: StyleEntry::fg(Color::Red),
            background: StyleEntry::bg(Color::DarkGray),
        }
    }
}

/// General / misc colors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralTheme {
    pub border: StyleEntry,
    pub accent: StyleEntry,
    pub muted: StyleEntry,
}

impl Default for GeneralTheme {
    fn default() -> Self {
        Self {
            border: StyleEntry::fg(Color::Rgb(51, 51, 102)),
            accent: StyleEntry::fg(Color::Cyan),
            muted: StyleEntry::fg(Color::DarkGray),
        }
    }
}

/// The full theme.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Theme {
    pub log_levels: LogLevelTheme,
    pub table: TableTheme,
    pub status_bar: StatusBarTheme,
    pub search: SearchTheme,
    pub dialog: DialogTheme,
    pub detail_panel: DetailPanelTheme,
    pub input: InputTheme,
    pub highlight_palette: Vec<ThemeColor>,
    pub general: GeneralTheme,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            log_levels: LogLevelTheme::default(),
            table: TableTheme::default(),
            status_bar: StatusBarTheme::default(),
            search: SearchTheme::default(),
            dialog: DialogTheme::default(),
            detail_panel: DetailPanelTheme::default(),
            input: InputTheme::default(),
            highlight_palette: vec![
                ThemeColor(Color::Red),
                ThemeColor(Color::Green),
                ThemeColor(Color::Blue),
                ThemeColor(Color::Yellow),
                ThemeColor(Color::Magenta),
                ThemeColor(Color::Cyan),
            ],
            general: GeneralTheme::default(),
        }
    }
}

impl Theme {
    /// Load a theme from YAML string, merging over defaults.
    pub fn from_yaml(yaml: &str) -> Result<Self, String> {
        serde_yaml::from_str(yaml).map_err(|e| format!("invalid theme YAML: {e}"))
    }
}

#[cfg(test)]
#[path = "theme_tests.rs"]
mod theme_tests;
