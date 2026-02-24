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
            error: StyleEntry::fg(Color::Rgb(255, 107, 107)), // soft red #FF6B6B
            warn: StyleEntry::fg(Color::Rgb(255, 217, 61)),   // warm yellow #FFD93D
            notice: StyleEntry::fg(Color::Rgb(107, 203, 119)), // soft green #6BCB77
            info: StyleEntry::fg(Color::Rgb(79, 195, 247)),   // light blue #4FC3F7
            debug: StyleEntry::fg(Color::Rgb(139, 139, 139)), // medium gray #8B8B8B
            trace: StyleEntry::fg(Color::Rgb(92, 92, 92)),    // dark gray #5C5C5C
        }
    }
}

/// Separator style: color + character.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SeparatorStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fg: Option<ThemeColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg: Option<ThemeColor>,
    /// Separator character (default: "│").
    #[serde(default = "SeparatorStyle::default_char")]
    pub char: String,
}

impl SeparatorStyle {
    fn default_char() -> String {
        "│".to_string()
    }

    pub fn separator_char(&self) -> &str {
        &self.char
    }

    /// Convert to a StyleEntry for rendering.
    pub fn to_style_entry(&self) -> StyleEntry {
        StyleEntry {
            fg: self.fg,
            bg: self.bg,
            bold: None,
        }
    }
}

impl Default for SeparatorStyle {
    fn default() -> Self {
        Self {
            fg: Some(ThemeColor(Color::Rgb(59, 66, 82))), // #3B4252
            bg: None,
            char: "│".to_string(),
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
    /// Column separator style (color + character).
    pub separator: SeparatorStyle,
}

impl Default for TableTheme {
    fn default() -> Self {
        Self {
            header: StyleEntry {
                fg: Some(ThemeColor(Color::Rgb(184, 196, 206))), // light steel #B8C4CE
                bg: Some(ThemeColor(Color::Rgb(30, 42, 56))),    // dark slate #1E2A38
                bold: Some(true),
            },
            selected: StyleEntry::bg(Color::Rgb(42, 63, 85)), // steel blue #2A3F55
            selected_search: StyleEntry::bg(Color::Rgb(120, 120, 0)),
            selected_highlight: StyleEntry::bg(Color::Rgb(40, 60, 80)),
            search_match: StyleEntry::bg(Color::Rgb(80, 80, 0)),
            bookmark: StyleEntry::bg(Color::Rgb(20, 40, 60)),
            separator: SeparatorStyle::default(),
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
    /// Density chart time label (e.g. "[█=5s]") — dimmer than chart itself.
    pub density_label: StyleEntry,
    /// Cursor marker in density chart.
    pub cursor_marker: StyleEntry,
}

impl Default for StatusBarTheme {
    fn default() -> Self {
        Self {
            line1_bg: StyleEntry::fg_bg(Color::Rgb(212, 212, 212), Color::Rgb(27, 40, 56)), // #D4D4D4 on #1B2838
            line2_bg: StyleEntry::fg_bg(Color::Rgb(160, 160, 160), Color::Rgb(13, 17, 23)), // #A0A0A0 on #0D1117
            density_hot: StyleEntry::fg_bg(Color::Yellow, Color::Rgb(27, 40, 56)),
            density_normal: StyleEntry::fg(Color::Rgb(79, 195, 247)), // light blue #4FC3F7
            position: StyleEntry::fg_bg(Color::White, Color::DarkGray),
            mode_follow: StyleEntry::fg_bg(Color::Black, Color::Green),
            mode_view: StyleEntry::fg_bg(Color::Black, Color::Cyan),
            mode_label: StyleEntry {
                fg: Some(ThemeColor(Color::Rgb(27, 40, 56))), // #1B2838
                bg: Some(ThemeColor(Color::Rgb(79, 195, 247))), // #4FC3F7
                bold: Some(true),
            },
            command_mode_label: StyleEntry::fg_bg(Color::Black, Color::Magenta),
            search_mode_label: StyleEntry::fg_bg(Color::Black, Color::Magenta),
            shortcut_key: StyleEntry::fg(Color::Yellow),
            shortcut_sep: StyleEntry::fg(Color::DarkGray),
            density_label: StyleEntry::fg(Color::Rgb(107, 123, 141)), // dimmer #6B7B8D
            cursor_marker: StyleEntry::fg(Color::Rgb(255, 217, 61)),  // yellow #FFD93D
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
            border: StyleEntry::fg(Color::Rgb(59, 66, 82)), // #3B4252
            accent: StyleEntry::fg(Color::Rgb(79, 195, 247)), // light blue #4FC3F7
            muted: StyleEntry::fg(Color::Rgb(107, 123, 141)), // #6B7B8D
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
                ThemeColor(Color::Rgb(255, 107, 107)), // soft red
                ThemeColor(Color::Rgb(107, 203, 119)), // soft green
                ThemeColor(Color::Rgb(79, 195, 247)),  // light blue
                ThemeColor(Color::Rgb(255, 217, 61)),  // yellow
                ThemeColor(Color::Rgb(186, 147, 230)), // lavender
                ThemeColor(Color::Rgb(77, 208, 225)),  // teal
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
            "landmine" => Some(Self::landmine()),
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

    /// Landmine theme — Jirai Kei (地雷系) black and pink aesthetic.
    pub fn landmine() -> Self {
        use Color::*;
        let deep_black = Rgb(13, 6, 11); // #0D060B
        let dark_wine = Rgb(26, 10, 20); // #1A0A14
        let rose_pink = Rgb(232, 87, 126); // #E8577E
        let bright_pink = Rgb(255, 51, 102); // #FF3366
        let soft_pink = Rgb(245, 160, 192); // #F5A0C0
        let dusty_rose = Rgb(212, 160, 185); // #D4A0B9
        let pale_pink = Rgb(245, 208, 224); // #F5D0E0
        let muted_plum = Rgb(138, 106, 126); // #8A6A7E
        let dark_plum = Rgb(107, 74, 94); // #6B4A5E
        let dark_mauve = Rgb(107, 91, 107); // #6B5B6B
        let deep_mauve = Rgb(74, 58, 74); // #4A3A4A
        let selected_bg = Rgb(45, 16, 40); // #2D1028
        let separator_fg = Rgb(74, 32, 64); // #4A2040
        let border_fg = Rgb(61, 26, 48); // #3D1A30
        let light_text = Rgb(200, 200, 200); // #C8C8C8

        Self {
            log_levels: LogLevelTheme {
                fatal: StyleEntry::fg_bold(bright_pink),
                error: StyleEntry::fg(rose_pink),
                warn: StyleEntry::fg(soft_pink),
                notice: StyleEntry::fg(dusty_rose),
                info: StyleEntry::fg(light_text),
                debug: StyleEntry::fg(dark_mauve),
                trace: StyleEntry::fg(deep_mauve),
            },
            table: TableTheme {
                header: StyleEntry {
                    fg: Some(ThemeColor(soft_pink)),
                    bg: Some(ThemeColor(dark_wine)),
                    bold: Some(true),
                },
                selected: StyleEntry::bg(selected_bg),
                separator: SeparatorStyle {
                    fg: Some(ThemeColor(separator_fg)),
                    bg: None,
                    char: "♡".to_string(),
                },
                ..TableTheme::default()
            },
            status_bar: StatusBarTheme {
                line1_bg: StyleEntry::fg_bg(dusty_rose, dark_wine),
                line2_bg: StyleEntry::fg_bg(muted_plum, deep_black),
                mode_label: StyleEntry {
                    fg: Some(ThemeColor(deep_black)),
                    bg: Some(ThemeColor(rose_pink)),
                    bold: Some(true),
                },
                mode_view: StyleEntry::fg_bg(deep_black, muted_plum),
                mode_follow: StyleEntry::fg_bg(deep_black, soft_pink),
                density_normal: StyleEntry::fg(rose_pink),
                density_hot: StyleEntry::fg(bright_pink),
                density_label: StyleEntry::fg(dark_plum),
                position: StyleEntry::fg(pale_pink),
                cursor_marker: StyleEntry::fg(bright_pink),
                ..StatusBarTheme::default()
            },
            search: SearchTheme {
                match_highlight: StyleEntry::fg_bg(Black, soft_pink),
                current_match: StyleEntry::fg_bg(Black, bright_pink),
            },
            dialog: DialogTheme {
                border: StyleEntry::fg(rose_pink),
                title: StyleEntry::fg_bold(soft_pink),
                selected: StyleEntry::fg_bg(White, selected_bg),
                text: StyleEntry::fg(dusty_rose),
                muted: StyleEntry::fg(dark_plum),
                ..DialogTheme::default()
            },
            detail_panel: DetailPanelTheme {
                border: StyleEntry::fg(separator_fg),
                field_name: StyleEntry::fg(rose_pink),
                field_value: StyleEntry::fg(dusty_rose),
                section_header: StyleEntry::fg_bold(soft_pink),
            },
            input: InputTheme {
                prompt: StyleEntry::fg(soft_pink),
                error: StyleEntry::fg(bright_pink),
                ..InputTheme::default()
            },
            highlight_palette: vec![
                ThemeColor(bright_pink), // #FF3366
                ThemeColor(soft_pink),   // #F5A0C0
                ThemeColor(dusty_rose),  // #D4A0B9
                ThemeColor(rose_pink),   // #E8577E
                ThemeColor(pale_pink),   // #F5D0E0
                ThemeColor(muted_plum),  // #8A6A7E
            ],
            general: GeneralTheme {
                accent: StyleEntry::fg(rose_pink),
                muted: StyleEntry::fg(dark_plum),
                border: StyleEntry::fg(border_fg),
            },
        }
    }
}

#[cfg(test)]
#[path = "theme_tests.rs"]
mod theme_tests;
