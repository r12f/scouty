//! Centralized color and style management for the TUI.
//!
//! All UI colors are defined in the [`Theme`] struct, enabling consistent
//! styling and future customization via YAML theme files.

use ratatui::style::{Color, Modifier, Style};
use scouty::record::LogLevel;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Color wrapper (serializable)
// ---------------------------------------------------------------------------

/// A single color value that can be serialized to/from YAML.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ThemeColor(pub Color);

impl ThemeColor {
    pub fn to_color(self) -> Color {
        self.0
    }
}

impl From<Color> for ThemeColor {
    fn from(c: Color) -> Self {
        Self(c)
    }
}

impl From<ThemeColor> for String {
    fn from(tc: ThemeColor) -> String {
        match tc.0 {
            Color::Black => "black".into(),
            Color::Red => "red".into(),
            Color::Green => "green".into(),
            Color::Yellow => "yellow".into(),
            Color::Blue => "blue".into(),
            Color::Magenta => "magenta".into(),
            Color::Cyan => "cyan".into(),
            Color::White => "white".into(),
            Color::Gray => "gray".into(),
            Color::DarkGray => "dark_gray".into(),
            Color::LightRed => "light_red".into(),
            Color::LightGreen => "light_green".into(),
            Color::LightYellow => "light_yellow".into(),
            Color::LightBlue => "light_blue".into(),
            Color::LightMagenta => "light_magenta".into(),
            Color::LightCyan => "light_cyan".into(),
            Color::Rgb(r, g, b) => format!("#{:02X}{:02X}{:02X}", r, g, b),
            Color::Indexed(i) => format!("color({})", i),
            Color::Reset => "reset".into(),
        }
    }
}

impl TryFrom<String> for ThemeColor {
    type Error = String;
    fn try_from(s: String) -> Result<Self, String> {
        let s = s.trim();
        let color = match s.to_lowercase().as_str() {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "white" => Color::White,
            "gray" | "grey" => Color::Gray,
            "dark_gray" | "dark_grey" | "darkgray" | "darkgrey" => Color::DarkGray,
            "light_red" | "lightred" => Color::LightRed,
            "light_green" | "lightgreen" => Color::LightGreen,
            "light_yellow" | "lightyellow" => Color::LightYellow,
            "light_blue" | "lightblue" => Color::LightBlue,
            "light_magenta" | "lightmagenta" => Color::LightMagenta,
            "light_cyan" | "lightcyan" => Color::LightCyan,
            "reset" => Color::Reset,
            _ => {
                if s.starts_with('#') && s.len() == 7 {
                    let r = u8::from_str_radix(&s[1..3], 16)
                        .map_err(|e| format!("bad hex color '{}': {}", s, e))?;
                    let g = u8::from_str_radix(&s[3..5], 16)
                        .map_err(|e| format!("bad hex color '{}': {}", s, e))?;
                    let b = u8::from_str_radix(&s[5..7], 16)
                        .map_err(|e| format!("bad hex color '{}': {}", s, e))?;
                    Color::Rgb(r, g, b)
                } else if s.starts_with("color(") && s.ends_with(')') {
                    let idx: u8 = s[6..s.len() - 1]
                        .trim()
                        .parse()
                        .map_err(|e| format!("bad color index '{}': {}", s, e))?;
                    Color::Indexed(idx)
                } else {
                    return Err(format!("unknown color '{}'", s));
                }
            }
        };
        Ok(Self(color))
    }
}

// ---------------------------------------------------------------------------
// Style entry (fg + optional bg + optional bold)
// ---------------------------------------------------------------------------

/// A serializable style entry with optional foreground, background, and bold.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
    pub const fn fg(color: Color) -> Self {
        Self {
            fg: Some(ThemeColor(color)),
            bg: None,
            bold: None,
        }
    }

    pub const fn fg_bg(fg: Color, bg: Color) -> Self {
        Self {
            fg: Some(ThemeColor(fg)),
            bg: Some(ThemeColor(bg)),
            bold: None,
        }
    }

    pub const fn fg_bold(fg: Color) -> Self {
        Self {
            fg: Some(ThemeColor(fg)),
            bg: None,
            bold: Some(true),
        }
    }

    pub const fn bg(bg: Color) -> Self {
        Self {
            fg: None,
            bg: Some(ThemeColor(bg)),
            bold: None,
        }
    }

    /// Convert to a ratatui [`Style`].
    pub fn to_style(self) -> Style {
        let mut s = Style::default();
        if let Some(fg) = self.fg {
            s = s.fg(fg.0);
        }
        if let Some(bg) = self.bg {
            s = s.bg(bg.0);
        }
        if self.bold == Some(true) {
            s = s.add_modifier(Modifier::BOLD);
        }
        s
    }
}

// ---------------------------------------------------------------------------
// Theme sub-structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LogLevelColors {
    pub fatal: StyleEntry,
    pub error: StyleEntry,
    pub warn: StyleEntry,
    pub notice: StyleEntry,
    pub info: StyleEntry,
    pub debug: StyleEntry,
    pub trace: StyleEntry,
}

impl Default for LogLevelColors {
    fn default() -> Self {
        Self {
            fatal: StyleEntry::fg_bold(Color::Red),
            error: StyleEntry::fg(Color::Red),
            warn: StyleEntry::fg(Color::Rgb(0xFF, 0xD7, 0x00)),
            notice: StyleEntry::fg(Color::Cyan),
            info: StyleEntry::fg(Color::Rgb(0x00, 0xCC, 0x66)),
            debug: StyleEntry::fg(Color::Gray),
            trace: StyleEntry::fg(Color::DarkGray),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TableColors {
    pub header: StyleEntry,
    pub selected: StyleEntry,
    pub alternating: StyleEntry,
}

impl Default for TableColors {
    fn default() -> Self {
        Self {
            header: StyleEntry::fg_bg(Color::White, Color::Rgb(0x1A, 0x1A, 0x2E)),
            selected: StyleEntry::bg(Color::Rgb(0x16, 0x21, 0x3E)),
            alternating: StyleEntry::bg(Color::Rgb(0x0F, 0x0F, 0x1A)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StatusBarColors {
    pub line1_bg: StyleEntry,
    pub line2_bg: StyleEntry,
    pub mode_label: StyleEntry,
    pub follow_mode: StyleEntry,
    pub view_mode: StyleEntry,
    pub density_chart: StyleEntry,
    pub density_label: StyleEntry,
    pub position: StyleEntry,
    pub filter_active: StyleEntry,
    pub entry_count: StyleEntry,
}

impl Default for StatusBarColors {
    fn default() -> Self {
        Self {
            line1_bg: StyleEntry::bg(Color::Rgb(0x14, 0x14, 0x28)),
            line2_bg: StyleEntry::bg(Color::Rgb(0x1E, 0x1E, 0x1E)),
            mode_label: StyleEntry::fg_bg(Color::Black, Color::Magenta),
            follow_mode: StyleEntry::fg_bg(Color::Black, Color::Green),
            view_mode: StyleEntry::fg_bg(Color::Black, Color::Cyan),
            density_chart: StyleEntry::fg(Color::Cyan),
            density_label: StyleEntry::fg_bg(Color::Yellow, Color::Rgb(0x28, 0x28, 0x3C)),
            position: StyleEntry::fg_bg(Color::White, Color::DarkGray),
            filter_active: StyleEntry::fg(Color::Yellow),
            entry_count: StyleEntry::fg(Color::DarkGray),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchColors {
    pub match_highlight: StyleEntry,
    pub current_match: StyleEntry,
}

impl Default for SearchColors {
    fn default() -> Self {
        Self {
            match_highlight: StyleEntry::fg_bg(Color::Black, Color::Yellow),
            current_match: StyleEntry::fg_bg(Color::Black, Color::Rgb(0xFF, 0x66, 0x00)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FilterColors {
    pub active_indicator: StyleEntry,
    pub error_text: StyleEntry,
}

impl Default for FilterColors {
    fn default() -> Self {
        Self {
            active_indicator: StyleEntry::fg(Color::Yellow),
            error_text: StyleEntry::fg(Color::Red),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DialogColors {
    pub border: StyleEntry,
    pub title: StyleEntry,
    pub selected: StyleEntry,
    pub unselected: StyleEntry,
    pub text: StyleEntry,
    pub muted: StyleEntry,
    pub background: StyleEntry,
}

impl Default for DialogColors {
    fn default() -> Self {
        Self {
            border: StyleEntry::fg(Color::Cyan),
            title: StyleEntry::fg_bold(Color::White),
            selected: StyleEntry::fg_bg(Color::White, Color::DarkGray),
            unselected: StyleEntry::fg(Color::DarkGray),
            text: StyleEntry::fg(Color::White),
            muted: StyleEntry::fg(Color::DarkGray),
            background: StyleEntry::bg(Color::Black),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DetailPanelColors {
    pub field_name: StyleEntry,
    pub field_value: StyleEntry,
    pub separator: StyleEntry,
}

impl Default for DetailPanelColors {
    fn default() -> Self {
        Self {
            field_name: StyleEntry::fg(Color::Cyan),
            field_value: StyleEntry::fg(Color::White),
            separator: StyleEntry::fg(Color::DarkGray),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InputColors {
    pub prompt: StyleEntry,
    pub text: StyleEntry,
    pub cursor: StyleEntry,
    pub error: StyleEntry,
    pub background: StyleEntry,
    pub mode_label: StyleEntry,
}

impl Default for InputColors {
    fn default() -> Self {
        Self {
            prompt: StyleEntry::fg(Color::Yellow),
            text: StyleEntry::fg(Color::White),
            cursor: StyleEntry::fg(Color::White),
            error: StyleEntry::fg(Color::Red),
            background: StyleEntry::bg(Color::DarkGray),
            mode_label: StyleEntry::fg_bg(Color::Black, Color::Yellow),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralColors {
    pub border: StyleEntry,
    pub accent: StyleEntry,
    pub muted: StyleEntry,
}

impl Default for GeneralColors {
    fn default() -> Self {
        Self {
            border: StyleEntry::fg(Color::Rgb(0x33, 0x33, 0x66)),
            accent: StyleEntry::fg(Color::Cyan),
            muted: StyleEntry::fg(Color::DarkGray),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct HighlightPalette {
    pub colors: Vec<ThemeColor>,
}

impl Default for HighlightPalette {
    fn default() -> Self {
        Self {
            colors: vec![
                ThemeColor(Color::Red),
                ThemeColor(Color::Rgb(0x00, 0xCC, 0x66)),
                ThemeColor(Color::Rgb(0x33, 0x99, 0xFF)),
                ThemeColor(Color::Yellow),
                ThemeColor(Color::Magenta),
                ThemeColor(Color::Cyan),
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// Row highlight colors (search match, bookmark, etc.)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RowHighlightColors {
    pub search_match: StyleEntry,
    pub current_search_match: StyleEntry,
    pub bookmark: StyleEntry,
    pub selected: StyleEntry,
    pub selected_search_match: StyleEntry,
}

impl Default for RowHighlightColors {
    fn default() -> Self {
        Self {
            search_match: StyleEntry::bg(Color::Rgb(40, 60, 80)),
            current_search_match: StyleEntry::bg(Color::Rgb(120, 120, 0)),
            bookmark: StyleEntry::bg(Color::Rgb(40, 40, 60)),
            selected: StyleEntry::bg(Color::Rgb(80, 80, 0)),
            selected_search_match: StyleEntry::bg(Color::Rgb(20, 40, 60)),
        }
    }
}

// ---------------------------------------------------------------------------
// Main Theme struct
// ---------------------------------------------------------------------------

/// Centralized theme holding all colors and styles for the TUI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Theme {
    pub log_levels: LogLevelColors,
    pub table: TableColors,
    pub status_bar: StatusBarColors,
    pub search: SearchColors,
    pub filter: FilterColors,
    pub dialog: DialogColors,
    pub detail_panel: DetailPanelColors,
    pub input: InputColors,
    pub highlight_palette: HighlightPalette,
    pub row_highlight: RowHighlightColors,
    pub general: GeneralColors,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            log_levels: LogLevelColors::default(),
            table: TableColors::default(),
            status_bar: StatusBarColors::default(),
            search: SearchColors::default(),
            filter: FilterColors::default(),
            dialog: DialogColors::default(),
            detail_panel: DetailPanelColors::default(),
            input: InputColors::default(),
            highlight_palette: HighlightPalette::default(),
            row_highlight: RowHighlightColors::default(),
            general: GeneralColors::default(),
        }
    }
}

impl Theme {
    /// Get the style for a log level.
    pub fn log_level_style(&self, level: Option<&LogLevel>) -> Style {
        match level {
            Some(LogLevel::Fatal) => self.log_levels.fatal.to_style(),
            Some(LogLevel::Error) => self.log_levels.error.to_style(),
            Some(LogLevel::Warn) => self.log_levels.warn.to_style(),
            Some(LogLevel::Notice) => self.log_levels.notice.to_style(),
            Some(LogLevel::Info) => self.log_levels.info.to_style(),
            Some(LogLevel::Debug) => self.log_levels.debug.to_style(),
            Some(LogLevel::Trace) => self.log_levels.trace.to_style(),
            None => Style::default(),
        }
    }

    /// Get the highlight color at the given index (wraps around).
    pub fn highlight_color(&self, index: usize) -> Color {
        if self.highlight_palette.colors.is_empty() {
            Color::Red
        } else {
            self.highlight_palette.colors[index % self.highlight_palette.colors.len()].0
        }
    }
}

#[cfg(test)]
#[path = "theme_tests.rs"]
mod theme_tests;
