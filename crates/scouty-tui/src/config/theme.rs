//! Theme: centralized color definitions for the TUI.

use super::color::ThemeColor;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Style entry: foreground, background, bold.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct StyleEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fg: Option<ThemeColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg: Option<ThemeColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
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

    /// Built-in preset: look up by name. Returns None for unknown names.
    pub fn builtin(name: &str) -> Option<Self> {
        match name {
            "default" => Some(Self::default()),
            "dark" => Some(Self::dark()),
            "light" => Some(Self::light()),
            "solarized" => Some(Self::solarized()),
            _ => None,
        }
    }

    /// Muted dark theme — lower contrast, softer colors.
    pub fn dark() -> Self {
        use Color::*;
        Self {
            log_levels: LogLevelTheme {
                fatal: StyleEntry::fg_bold(Red),
                error: StyleEntry::fg(Rgb(205, 92, 92)),
                warn: StyleEntry::fg(Rgb(210, 180, 100)),
                notice: StyleEntry::fg(Rgb(100, 160, 180)),
                info: StyleEntry::fg(Rgb(120, 180, 120)),
                debug: StyleEntry::fg(Rgb(140, 140, 140)),
                trace: StyleEntry::fg(DarkGray),
            },
            table: TableTheme {
                header: StyleEntry::fg_bg(Rgb(180, 180, 180), Rgb(30, 30, 40)),
                selected: StyleEntry::bg(Rgb(40, 40, 55)),
                ..TableTheme::default()
            },
            status_bar: StatusBarTheme {
                line1_bg: StyleEntry::bg(Rgb(30, 30, 40)),
                line2_bg: StyleEntry::bg(Rgb(35, 35, 50)),
                mode_label: StyleEntry::fg_bg(Black, Rgb(100, 160, 180)),
                mode_view: StyleEntry::fg_bg(Black, Rgb(140, 140, 140)),
                mode_follow: StyleEntry::fg_bg(Black, Rgb(120, 180, 120)),
                density_normal: StyleEntry::fg(Rgb(100, 160, 180)),
                density_hot: StyleEntry::fg(Rgb(210, 180, 100)),
                ..StatusBarTheme::default()
            },
            search: SearchTheme {
                match_highlight: StyleEntry::fg_bg(Black, Rgb(210, 180, 100)),
                current_match: StyleEntry::fg_bg(Black, Rgb(200, 120, 50)),
            },
            dialog: DialogTheme {
                border: StyleEntry::fg(Rgb(100, 160, 180)),
                title: StyleEntry::fg_bold(Rgb(180, 180, 180)),
                selected: StyleEntry::fg_bg(White, Rgb(40, 40, 55)),
                text: StyleEntry::fg(Rgb(180, 180, 180)),
                muted: StyleEntry::fg(DarkGray),
                ..DialogTheme::default()
            },
            detail_panel: DetailPanelTheme {
                border: StyleEntry::fg(Rgb(60, 60, 80)),
                field_name: StyleEntry::fg(Rgb(100, 160, 180)),
                field_value: StyleEntry::fg(Rgb(180, 180, 180)),
                section_header: StyleEntry::fg_bold(Rgb(180, 180, 180)),
            },
            input: InputTheme {
                prompt: StyleEntry::fg(Rgb(210, 180, 100)),
                error: StyleEntry::fg(Rgb(205, 92, 92)),
                ..InputTheme::default()
            },
            general: GeneralTheme {
                accent: StyleEntry::fg(Rgb(100, 160, 180)),
                muted: StyleEntry::fg(DarkGray),
                border: StyleEntry::fg(Rgb(60, 60, 80)),
            },
            ..Self::default()
        }
    }

    /// Light theme — light background, dark text.
    pub fn light() -> Self {
        use Color::*;
        Self {
            log_levels: LogLevelTheme {
                fatal: StyleEntry::fg_bold(Rgb(180, 0, 0)),
                error: StyleEntry::fg(Rgb(180, 0, 0)),
                warn: StyleEntry::fg(Rgb(180, 120, 0)),
                notice: StyleEntry::fg(Rgb(0, 120, 150)),
                info: StyleEntry::fg(Rgb(0, 130, 60)),
                debug: StyleEntry::fg(Rgb(100, 100, 100)),
                trace: StyleEntry::fg(Rgb(150, 150, 150)),
            },
            table: TableTheme {
                header: StyleEntry::fg_bg(Rgb(30, 30, 40), Rgb(220, 220, 230)),
                selected: StyleEntry::fg_bg(Black, Rgb(200, 210, 230)),
                ..TableTheme::default()
            },
            status_bar: StatusBarTheme {
                line1_bg: StyleEntry::bg(Rgb(220, 220, 230)),
                line2_bg: StyleEntry::bg(Rgb(210, 210, 220)),
                mode_label: StyleEntry::fg_bg(White, Rgb(0, 120, 150)),
                mode_view: StyleEntry::fg_bg(White, Rgb(100, 100, 100)),
                mode_follow: StyleEntry::fg_bg(White, Rgb(0, 130, 60)),
                density_normal: StyleEntry::fg(Rgb(0, 120, 150)),
                density_hot: StyleEntry::fg(Rgb(180, 120, 0)),
                ..StatusBarTheme::default()
            },
            search: SearchTheme {
                match_highlight: StyleEntry::fg_bg(Black, Rgb(255, 230, 100)),
                current_match: StyleEntry::fg_bg(Black, Rgb(255, 180, 50)),
            },
            dialog: DialogTheme {
                border: StyleEntry::fg(Rgb(0, 120, 150)),
                title: StyleEntry::fg_bold(Rgb(30, 30, 40)),
                selected: StyleEntry::fg_bg(Black, Rgb(200, 210, 230)),
                text: StyleEntry::fg(Rgb(30, 30, 40)),
                muted: StyleEntry::fg(Rgb(150, 150, 150)),
                ..DialogTheme::default()
            },
            detail_panel: DetailPanelTheme {
                border: StyleEntry::fg(Rgb(180, 180, 190)),
                field_name: StyleEntry::fg(Rgb(0, 120, 150)),
                field_value: StyleEntry::fg(Rgb(30, 30, 40)),
                section_header: StyleEntry::fg_bold(Rgb(30, 30, 40)),
            },
            input: InputTheme {
                prompt: StyleEntry::fg(Rgb(180, 120, 0)),
                error: StyleEntry::fg(Rgb(180, 0, 0)),
                ..InputTheme::default()
            },
            general: GeneralTheme {
                accent: StyleEntry::fg(Rgb(0, 120, 150)),
                muted: StyleEntry::fg(Rgb(150, 150, 150)),
                border: StyleEntry::fg(Rgb(180, 180, 190)),
            },
            ..Self::default()
        }
    }

    /// Solarized theme — based on Ethan Schoonover's solarized palette.
    pub fn solarized() -> Self {
        use Color::*;
        let base03 = Rgb(0, 43, 54);
        let base02 = Rgb(7, 54, 66);
        let base01 = Rgb(88, 110, 117);
        let base0 = Rgb(131, 148, 150);
        let base1 = Rgb(147, 161, 161);
        let yellow = Rgb(181, 137, 0);
        let orange = Rgb(203, 75, 22);
        let red = Rgb(220, 50, 47);
        let blue = Rgb(38, 139, 210);
        let cyan = Rgb(42, 161, 152);
        let green = Rgb(133, 153, 0);

        Self {
            log_levels: LogLevelTheme {
                fatal: StyleEntry::fg_bold(red),
                error: StyleEntry::fg(red),
                warn: StyleEntry::fg(yellow),
                notice: StyleEntry::fg(cyan),
                info: StyleEntry::fg(green),
                debug: StyleEntry::fg(base01),
                trace: StyleEntry::fg(base01),
            },
            table: TableTheme {
                header: StyleEntry::fg_bg(base1, base02),
                selected: StyleEntry::fg_bg(base1, base02),
                ..TableTheme::default()
            },
            status_bar: StatusBarTheme {
                line1_bg: StyleEntry::bg(base02),
                line2_bg: StyleEntry::bg(base02),
                mode_label: StyleEntry::fg_bg(base03, cyan),
                mode_view: StyleEntry::fg_bg(base03, base01),
                mode_follow: StyleEntry::fg_bg(base03, green),
                density_normal: StyleEntry::fg(blue),
                density_hot: StyleEntry::fg(orange),
                ..StatusBarTheme::default()
            },
            search: SearchTheme {
                match_highlight: StyleEntry::fg_bg(base03, yellow),
                current_match: StyleEntry::fg_bg(base03, orange),
            },
            dialog: DialogTheme {
                border: StyleEntry::fg(blue),
                title: StyleEntry::fg_bold(base1),
                selected: StyleEntry::fg_bg(base1, base02),
                text: StyleEntry::fg(base0),
                muted: StyleEntry::fg(base01),
                ..DialogTheme::default()
            },
            detail_panel: DetailPanelTheme {
                border: StyleEntry::fg(base01),
                field_name: StyleEntry::fg(cyan),
                field_value: StyleEntry::fg(base0),
                section_header: StyleEntry::fg_bold(base1),
            },
            input: InputTheme {
                prompt: StyleEntry::fg(yellow),
                error: StyleEntry::fg(red),
                ..InputTheme::default()
            },
            general: GeneralTheme {
                accent: StyleEntry::fg(blue),
                muted: StyleEntry::fg(base01),
                border: StyleEntry::fg(base01),
            },
            ..Self::default()
        }
    }
}

#[cfg(test)]
#[path = "theme_tests.rs"]
mod theme_tests;
