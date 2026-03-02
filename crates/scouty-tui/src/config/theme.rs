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
            fatal: StyleEntry::fg_bold(Color::Rgb(255, 107, 107)),
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
    pub header_unfocused: StyleEntry,
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
                fg: Some(ThemeColor(Color::Rgb(27, 40, 56))), // #1B2838 (matches panel_tab.focused)
                bg: Some(ThemeColor(Color::Rgb(79, 195, 247))), // #4FC3F7 accent
                bold: Some(true),
            },
            header_unfocused: StyleEntry::fg_bg(
                Color::Rgb(107, 123, 141), // #6B7B8D
                Color::Rgb(30, 42, 56),
            ),
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
    /// Tick marks between density chart braille groups.
    pub density_tick: StyleEntry,
}

impl Default for StatusBarTheme {
    fn default() -> Self {
        Self {
            line1_bg: StyleEntry::fg_bg(Color::Rgb(212, 212, 212), Color::Rgb(27, 40, 56)), // #D4D4D4 on #1B2838
            line2_bg: StyleEntry::fg_bg(Color::Rgb(160, 160, 160), Color::Rgb(13, 17, 23)), // #A0A0A0 on #0D1117
            density_hot: StyleEntry::fg_bg(Color::Rgb(255, 217, 61), Color::Rgb(27, 40, 56)),
            density_normal: StyleEntry::fg(Color::Rgb(79, 195, 247)), // light blue #4FC3F7
            position: StyleEntry::fg_bg(Color::Rgb(212, 212, 212), Color::Rgb(92, 92, 92)),
            mode_follow: StyleEntry::fg_bg(Color::Rgb(13, 17, 23), Color::Rgb(107, 203, 119)),
            mode_view: StyleEntry::fg_bg(Color::Rgb(13, 17, 23), Color::Rgb(77, 208, 225)),
            mode_label: StyleEntry {
                fg: Some(ThemeColor(Color::Rgb(27, 40, 56))), // #1B2838
                bg: Some(ThemeColor(Color::Rgb(79, 195, 247))), // #4FC3F7
                bold: Some(true),
            },
            command_mode_label: StyleEntry::fg_bg(
                Color::Rgb(13, 17, 23),
                Color::Rgb(206, 147, 216),
            ),
            search_mode_label: StyleEntry::fg_bg(Color::Rgb(13, 17, 23), Color::Rgb(206, 147, 216)),
            shortcut_key: StyleEntry::fg(Color::Rgb(255, 217, 61)),
            shortcut_sep: StyleEntry::fg(Color::Rgb(92, 92, 92)),
            density_label: StyleEntry::fg(Color::Rgb(107, 123, 141)), // dimmer #6B7B8D
            cursor_marker: StyleEntry::fg(Color::Rgb(255, 217, 61)),  // yellow #FFD93D
            density_tick: StyleEntry::fg(Color::Rgb(59, 66, 82)),     // dim #3B4252
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
            match_highlight: StyleEntry::fg_bg(Color::Rgb(13, 17, 23), Color::Rgb(255, 217, 61)),
            current_match: StyleEntry::fg_bg(Color::Rgb(13, 17, 23), Color::Rgb(255, 102, 0)),
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
            border: StyleEntry::fg(Color::Rgb(77, 208, 225)),
            title: StyleEntry::fg(Color::Rgb(77, 208, 225)),
            selected: StyleEntry::fg_bg(Color::Rgb(212, 212, 212), Color::Rgb(92, 92, 92)),
            text: StyleEntry::fg(Color::Rgb(212, 212, 212)),
            muted: StyleEntry::fg(Color::Rgb(92, 92, 92)),
            background: StyleEntry::bg(Color::Rgb(13, 17, 23)),
            accent: StyleEntry::fg(Color::Rgb(255, 217, 61)),
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
            field_name: StyleEntry::fg(Color::Rgb(77, 208, 225)),
            field_value: StyleEntry::fg(Color::Rgb(212, 212, 212)),
            border: StyleEntry::fg(Color::Rgb(92, 92, 92)),
            section_header: StyleEntry::fg(Color::Rgb(77, 208, 225)),
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
            prompt: StyleEntry::fg(Color::Rgb(255, 217, 61)),
            cursor: StyleEntry::fg(Color::Rgb(212, 212, 212)),
            text: StyleEntry::fg(Color::Rgb(212, 212, 212)),
            error: StyleEntry::fg(Color::Rgb(255, 107, 107)),
            background: StyleEntry::bg(Color::Rgb(92, 92, 92)),
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

/// Panel tab bar styling.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelTabTheme {
    /// Active tab when panel has keyboard focus.
    pub focused: StyleEntry,
    /// Active tab when panel does NOT have focus (gray/muted).
    pub unfocused: StyleEntry,
    /// Tab bar background.
    pub bar_bg: StyleEntry,
}

impl Default for PanelTabTheme {
    fn default() -> Self {
        Self {
            focused: StyleEntry {
                fg: Some(ThemeColor(Color::Rgb(27, 40, 56))), // #1B2838
                bg: Some(ThemeColor(Color::Rgb(79, 195, 247))), // #4FC3F7 accent
                bold: Some(true),
            },
            unfocused: StyleEntry {
                fg: Some(ThemeColor(Color::Rgb(107, 123, 141))), // #6B7B8D gray
                bg: Some(ThemeColor(Color::Rgb(27, 40, 56))),    // #1B2838
                bold: None,
            },
            bar_bg: StyleEntry::bg(Color::Rgb(13, 17, 23)), // #0D1117
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
    pub panel_tab: PanelTabTheme,
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
            panel_tab: PanelTabTheme::default(),
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
            "mizuiro" => Some(Self::mizuiro()),
            "amai" => Some(Self::amai()),
            "maid" => Some(Self::maid()),
            "gyaru" => Some(Self::gyaru()),
            _ => None,
        }
    }

    /// List all built-in theme names.
    pub fn builtin_names() -> &'static [&'static str] {
        &[
            "default",
            "dark",
            "light",
            "solarized",
            "landmine",
            "mizuiro",
            "amai",
            "maid",
            "gyaru",
        ]
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
                header: StyleEntry {
                    fg: Some(ThemeColor(Black)),
                    bg: Some(ThemeColor(Rgb(100, 160, 180))),
                    bold: None,
                },
                header_unfocused: StyleEntry::fg_bg(Rgb(85, 85, 85), Rgb(30, 30, 40)),
                selected: StyleEntry::bg(Rgb(40, 40, 55)),
                selected_search: StyleEntry::bg(Rgb(80, 80, 30)),
                selected_highlight: StyleEntry::bg(Rgb(35, 45, 60)),
                search_match: StyleEntry::bg(Rgb(60, 60, 20)),
                bookmark: StyleEntry::bg(Rgb(25, 35, 50)),
                separator: SeparatorStyle::default(),
            },
            status_bar: StatusBarTheme {
                line1_bg: StyleEntry::bg(Rgb(30, 30, 40)),
                line2_bg: StyleEntry::bg(Rgb(35, 35, 50)),
                mode_label: StyleEntry::fg_bg(Black, Rgb(100, 160, 180)),
                mode_view: StyleEntry::fg_bg(Black, Rgb(140, 140, 140)),
                mode_follow: StyleEntry::fg_bg(Black, Rgb(120, 180, 120)),
                density_normal: StyleEntry::fg(Rgb(100, 160, 180)),
                density_hot: StyleEntry::fg(Rgb(210, 180, 100)),
                density_tick: StyleEntry::fg(Rgb(42, 74, 90)), // Ocean #2A4A5A
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
            panel_tab: PanelTabTheme {
                focused: StyleEntry {
                    fg: Some(ThemeColor(Black)),
                    bg: Some(ThemeColor(Rgb(100, 160, 180))),
                    bold: Some(true),
                },
                unfocused: StyleEntry {
                    fg: Some(ThemeColor(Rgb(85, 85, 85))),
                    bg: Some(ThemeColor(Rgb(30, 30, 40))),
                    bold: None,
                },
                bar_bg: StyleEntry::bg(Rgb(20, 20, 30)),
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
                header: StyleEntry {
                    fg: Some(ThemeColor(White)),
                    bg: Some(ThemeColor(Rgb(0, 120, 150))),
                    bold: None,
                },
                header_unfocused: StyleEntry::fg_bg(Rgb(153, 153, 153), Rgb(220, 220, 230)),
                selected: StyleEntry::fg_bg(Black, Rgb(200, 210, 230)),
                selected_search: StyleEntry::fg_bg(Black, Rgb(220, 220, 160)),
                selected_highlight: StyleEntry::fg_bg(Black, Rgb(210, 218, 235)),
                search_match: StyleEntry::bg(Rgb(240, 240, 180)),
                bookmark: StyleEntry::bg(Rgb(200, 220, 240)),
                separator: SeparatorStyle::default(),
            },
            status_bar: StatusBarTheme {
                line1_bg: StyleEntry::bg(Rgb(220, 220, 230)),
                line2_bg: StyleEntry::bg(Rgb(210, 210, 220)),
                mode_label: StyleEntry::fg_bg(White, Rgb(0, 120, 150)),
                mode_view: StyleEntry::fg_bg(White, Rgb(100, 100, 100)),
                mode_follow: StyleEntry::fg_bg(White, Rgb(0, 130, 60)),
                density_normal: StyleEntry::fg(Rgb(0, 120, 150)),
                density_hot: StyleEntry::fg(Rgb(180, 120, 0)),
                density_tick: StyleEntry::fg(Rgb(42, 74, 42)), // Forest #2A4A2A
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
            panel_tab: PanelTabTheme {
                focused: StyleEntry {
                    fg: Some(ThemeColor(White)),
                    bg: Some(ThemeColor(Rgb(0, 120, 150))),
                    bold: Some(true),
                },
                unfocused: StyleEntry {
                    fg: Some(ThemeColor(Rgb(153, 153, 153))),
                    bg: Some(ThemeColor(Rgb(230, 230, 235))),
                    bold: None,
                },
                bar_bg: StyleEntry::bg(Rgb(240, 240, 245)),
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
                header: StyleEntry {
                    fg: Some(ThemeColor(base03)),
                    bg: Some(ThemeColor(blue)),
                    bold: None,
                },
                header_unfocused: StyleEntry::fg_bg(Rgb(101, 123, 131), base02), // #657B83
                selected: StyleEntry::fg_bg(base1, base02),
                selected_search: StyleEntry::bg(Rgb(80, 80, 20)),
                selected_highlight: StyleEntry::bg(Rgb(10, 50, 60)),
                search_match: StyleEntry::bg(Rgb(60, 60, 10)),
                bookmark: StyleEntry::bg(Rgb(0, 50, 70)),
                separator: SeparatorStyle::default(),
            },
            status_bar: StatusBarTheme {
                line1_bg: StyleEntry::bg(base02),
                line2_bg: StyleEntry::bg(base02),
                mode_label: StyleEntry::fg_bg(base03, cyan),
                mode_view: StyleEntry::fg_bg(base03, base01),
                mode_follow: StyleEntry::fg_bg(base03, green),
                density_normal: StyleEntry::fg(blue),
                density_hot: StyleEntry::fg(orange),
                density_tick: StyleEntry::fg(Rgb(7, 54, 66)), // Solarized #073642
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
            panel_tab: PanelTabTheme {
                focused: StyleEntry {
                    fg: Some(ThemeColor(base03)),
                    bg: Some(ThemeColor(blue)),
                    bold: Some(true),
                },
                unfocused: StyleEntry {
                    fg: Some(ThemeColor(Rgb(101, 123, 131))),
                    bg: Some(ThemeColor(base02)),
                    bold: None,
                }, // #657B83
                bar_bg: StyleEntry::bg(base03),
            },
            ..Self::default()
        }
    }

    /// Landmine theme — Jirai Kei black and pink aesthetic.
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
                    fg: Some(ThemeColor(deep_black)),
                    bg: Some(ThemeColor(rose_pink)),
                    bold: Some(true),
                },
                header_unfocused: StyleEntry::fg_bg(Rgb(107, 74, 94), dark_wine), // #6B4A5E
                selected: StyleEntry::bg(selected_bg),
                selected_search: StyleEntry::bg(Rgb(90, 60, 30)),
                selected_highlight: StyleEntry::bg(Rgb(50, 25, 40)),
                search_match: StyleEntry::bg(Rgb(70, 50, 20)),
                bookmark: StyleEntry::bg(Rgb(45, 20, 45)),
                separator: SeparatorStyle {
                    fg: Some(ThemeColor(separator_fg)),
                    bg: None,
                    char: "♡".to_string(),
                },
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
                density_tick: StyleEntry::fg(separator_fg), // Sakura #4A2040
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
            panel_tab: PanelTabTheme {
                focused: StyleEntry {
                    fg: Some(ThemeColor(deep_black)),
                    bg: Some(ThemeColor(rose_pink)),
                    bold: Some(true),
                },
                unfocused: StyleEntry {
                    fg: Some(ThemeColor(Rgb(107, 74, 94))),
                    bg: Some(ThemeColor(dark_wine)),
                    bold: None,
                }, // #6B4A5E
                bar_bg: StyleEntry::bg(deep_black),
            },
        }
    }

    /// Mizuiro — clear, transparent, aqua theme.
    pub fn mizuiro() -> Self {
        use Color::*;
        let deep_navy = Rgb(10, 22, 40); // #0A1628
        let dark_ocean = Rgb(15, 32, 56); // #0F2038
        let water_blue = Rgb(91, 164, 207); // #5BA4CF
        let sky_blue = Rgb(123, 200, 246); // #7BC8F6
        let ice_blue = Rgb(168, 216, 234); // #A8D8EA
        let pale_aqua = Rgb(212, 238, 246); // #D4EEF6
        let deep_blue = Rgb(46, 107, 158); // #2E6B9E
        let steel_blue = Rgb(74, 123, 157); // #4A7B9D
        let muted_blue = Rgb(59, 90, 122); // #3B5A7A
        let dark_slate = Rgb(30, 58, 80); // #1E3A50
        let silver_mist = Rgb(139, 164, 184); // #8BA4B8
        let light_text = Rgb(200, 214, 229); // #C8D6E5
        let selected_bg = Rgb(22, 46, 72); // #162E48
        let coral_accent = Rgb(232, 116, 97); // #E87461
        let amber_warn = Rgb(232, 167, 78); // #E8A74E

        Self {
            log_levels: LogLevelTheme {
                fatal: StyleEntry::fg_bold(coral_accent),
                error: StyleEntry::fg(Rgb(207, 107, 94)), // #CF6B5E
                warn: StyleEntry::fg(amber_warn),
                notice: StyleEntry::fg(sky_blue),
                info: StyleEntry::fg(light_text),
                debug: StyleEntry::fg(steel_blue),
                trace: StyleEntry::fg(muted_blue),
            },
            table: TableTheme {
                header: StyleEntry {
                    fg: Some(ThemeColor(deep_navy)),
                    bg: Some(ThemeColor(water_blue)),
                    bold: Some(true),
                },
                header_unfocused: StyleEntry::fg_bg(steel_blue, dark_ocean),
                selected: StyleEntry::bg(selected_bg),
                separator: SeparatorStyle {
                    fg: Some(ThemeColor(dark_slate)),
                    bg: None,
                    char: "│".to_string(),
                },
                selected_search: StyleEntry::bg(Rgb(20, 50, 80)), // #143250 deep ocean search selection
                selected_highlight: StyleEntry::bg(Rgb(18, 40, 68)), // #122844 dark blue highlight
                search_match: StyleEntry::bg(Rgb(40, 70, 100)),   // #284664 steel blue search match
                bookmark: StyleEntry::bg(Rgb(15, 45, 75)),        // #0F2D4B navy bookmark
            },
            status_bar: StatusBarTheme {
                line1_bg: StyleEntry::fg_bg(silver_mist, dark_ocean),
                line2_bg: StyleEntry::fg_bg(steel_blue, deep_navy),
                mode_label: StyleEntry {
                    fg: Some(ThemeColor(deep_navy)),
                    bg: Some(ThemeColor(water_blue)),
                    bold: Some(true),
                },
                mode_view: StyleEntry::fg_bg(deep_navy, steel_blue),
                mode_follow: StyleEntry::fg_bg(deep_navy, sky_blue),
                density_normal: StyleEntry::fg(water_blue),
                density_hot: StyleEntry::fg(sky_blue),
                density_label: StyleEntry::fg(muted_blue),
                density_tick: StyleEntry::fg(dark_slate),
                position: StyleEntry::fg(ice_blue),
                cursor_marker: StyleEntry::fg(sky_blue),
                ..StatusBarTheme::default()
            },
            search: SearchTheme {
                match_highlight: StyleEntry::fg_bg(deep_navy, ice_blue),
                current_match: StyleEntry::fg_bg(deep_navy, sky_blue),
            },
            dialog: DialogTheme {
                border: StyleEntry::fg(water_blue),
                title: StyleEntry::fg_bold(sky_blue),
                selected: StyleEntry::fg_bg(Rgb(232, 240, 246), selected_bg), // #E8F0F6
                text: StyleEntry::fg(silver_mist),
                muted: StyleEntry::fg(muted_blue),
                ..DialogTheme::default()
            },
            detail_panel: DetailPanelTheme {
                border: StyleEntry::fg(dark_slate),
                field_name: StyleEntry::fg(water_blue),
                field_value: StyleEntry::fg(silver_mist),
                section_header: StyleEntry::fg_bold(sky_blue),
            },
            input: InputTheme {
                prompt: StyleEntry::fg(sky_blue),
                error: StyleEntry::fg(coral_accent),
                ..InputTheme::default()
            },
            highlight_palette: vec![
                ThemeColor(sky_blue),   // #7BC8F6
                ThemeColor(water_blue), // #5BA4CF
                ThemeColor(ice_blue),   // #A8D8EA
                ThemeColor(deep_blue),  // #2E6B9E
                ThemeColor(pale_aqua),  // #D4EEF6
                ThemeColor(steel_blue), // #4A7B9D
            ],
            general: GeneralTheme {
                accent: StyleEntry::fg(water_blue),
                muted: StyleEntry::fg(muted_blue),
                border: StyleEntry::fg(dark_slate),
            },
            panel_tab: PanelTabTheme {
                focused: StyleEntry {
                    fg: Some(ThemeColor(deep_navy)),
                    bg: Some(ThemeColor(water_blue)),
                    bold: Some(true),
                },
                unfocused: StyleEntry {
                    fg: Some(ThemeColor(steel_blue)),
                    bg: Some(ThemeColor(dark_ocean)),
                    bold: None,
                },
                bar_bg: StyleEntry::bg(deep_navy),
            },
        }
    }

    /// Amai — Sweet Lolita, dreamy pastel pink theme.
    pub fn amai() -> Self {
        use Color::*;
        let deep_rose = Rgb(20, 10, 16); // #140A10 (darker, sweeter bg)
        let dark_berry = Rgb(61, 37, 64); // #3D2540 (richer berry midtone)
        let baby_pink = Rgb(255, 200, 214); // #FFC8D6 (brighter candy pink)
        let hot_pink = Rgb(255, 107, 138); // #FF6B8A
        let lavender = Rgb(200, 162, 200); // #C8A2C8
        let mint = Rgb(152, 216, 200); // #98D8C8
        let dusty_pink = Rgb(232, 184, 200); // #E8B8C8 (warmer pink text)
        let soft_mauve = Rgb(176, 138, 158); // #B08A9E (brighter unfocused)
        let muted_mauve = Rgb(138, 106, 126); // #8A6A7E (debug level)
        let muted_plum = Rgb(122, 90, 110); // #7A5A6E (muted/density label)
        let deep_plum = Rgb(90, 58, 78); // #5A3A4E
        let dark_mauve = Rgb(74, 46, 64); // #4A2E40
        let selected_bg = Rgb(58, 24, 48); // #3A1830
        let light_text = Rgb(232, 208, 218); // #E8D0DA
        let pale_yellow = Rgb(255, 232, 160); // #FFE8A0
        let position_fg = Rgb(212, 178, 212); // #D4B2D4

        Self {
            log_levels: LogLevelTheme {
                fatal: StyleEntry::fg_bold(hot_pink),
                error: StyleEntry::fg(Rgb(232, 90, 122)), // #E85A7A
                warn: StyleEntry::fg(pale_yellow),
                notice: StyleEntry::fg(lavender),
                info: StyleEntry::fg(light_text),
                debug: StyleEntry::fg(muted_mauve),
                trace: StyleEntry::fg(deep_plum),
            },
            table: TableTheme {
                header: StyleEntry {
                    fg: Some(ThemeColor(deep_rose)),
                    bg: Some(ThemeColor(baby_pink)),
                    bold: Some(true),
                },
                header_unfocused: StyleEntry::fg_bg(soft_mauve, dark_berry),
                selected: StyleEntry::bg(selected_bg),
                separator: SeparatorStyle {
                    fg: Some(ThemeColor(dark_mauve)),
                    bg: None,
                    char: "♡".to_string(),
                },
                selected_search: StyleEntry::bg(Rgb(70, 20, 50)), // #461432 deep rose search selection
                selected_highlight: StyleEntry::bg(Rgb(55, 22, 44)), // #37162C dark berry highlight
                search_match: StyleEntry::bg(Rgb(90, 30, 60)),    // #5A1E3C berry search match
                bookmark: StyleEntry::bg(Rgb(60, 18, 48)),        // #3C1230 plum bookmark
            },
            status_bar: StatusBarTheme {
                line1_bg: StyleEntry::fg_bg(dusty_pink, dark_berry),
                line2_bg: StyleEntry::fg_bg(soft_mauve, deep_rose),
                mode_label: StyleEntry {
                    fg: Some(ThemeColor(deep_rose)),
                    bg: Some(ThemeColor(baby_pink)),
                    bold: Some(true),
                },
                mode_view: StyleEntry::fg_bg(deep_rose, soft_mauve),
                mode_follow: StyleEntry::fg_bg(deep_rose, mint),
                density_normal: StyleEntry::fg(baby_pink),
                density_hot: StyleEntry::fg(hot_pink),
                density_label: StyleEntry::fg(muted_plum),
                density_tick: StyleEntry::fg(dark_mauve),
                position: StyleEntry::fg(position_fg),
                cursor_marker: StyleEntry::fg(hot_pink),
                ..StatusBarTheme::default()
            },
            search: SearchTheme {
                match_highlight: StyleEntry::fg_bg(deep_rose, lavender),
                current_match: StyleEntry::fg_bg(deep_rose, baby_pink),
            },
            dialog: DialogTheme {
                border: StyleEntry::fg(baby_pink),
                title: StyleEntry::fg_bold(hot_pink),
                selected: StyleEntry::fg_bg(Rgb(255, 240, 232), selected_bg), // #FFF0E8
                text: StyleEntry::fg(dusty_pink),
                muted: StyleEntry::fg(muted_plum),
                ..DialogTheme::default()
            },
            detail_panel: DetailPanelTheme {
                border: StyleEntry::fg(dark_mauve),
                field_name: StyleEntry::fg(baby_pink),
                field_value: StyleEntry::fg(dusty_pink),
                section_header: StyleEntry::fg_bold(mint),
            },
            input: InputTheme {
                prompt: StyleEntry::fg(lavender),
                error: StyleEntry::fg(hot_pink),
                ..InputTheme::default()
            },
            highlight_palette: vec![
                ThemeColor(hot_pink),           // #FF6B8A
                ThemeColor(baby_pink),          // #FFC8D6
                ThemeColor(lavender),           // #C8A2C8
                ThemeColor(mint),               // #98D8C8
                ThemeColor(Rgb(107, 142, 194)), // #6B8EC2 sax blue
                ThemeColor(pale_yellow),        // #FFE8A0
            ],
            general: GeneralTheme {
                accent: StyleEntry::fg(baby_pink),
                muted: StyleEntry::fg(muted_plum),
                border: StyleEntry::fg(dark_mauve),
            },
            panel_tab: PanelTabTheme {
                focused: StyleEntry {
                    fg: Some(ThemeColor(deep_rose)),
                    bg: Some(ThemeColor(baby_pink)),
                    bold: Some(true),
                },
                unfocused: StyleEntry {
                    fg: Some(ThemeColor(soft_mauve)),
                    bg: Some(ThemeColor(dark_berry)),
                    bold: None,
                },
                bar_bg: StyleEntry::bg(deep_rose),
            },
        }
    }


    /// Maid — classic maid, black & white high contrast with wine red accents.
    pub fn maid() -> Self {
        use Color::*;
        let black_dress = Rgb(13, 13, 26); // #0D0D1A
        let dark_fabric = Rgb(26, 26, 46); // #1A1A2E
        let lace_white = Rgb(240, 237, 232); // #F0EDE8
        let lace_shadow = Rgb(176, 168, 185); // #B0A8B9
        let wine_red = Rgb(139, 34, 82); // #8B2252
        let bright_red = Rgb(196, 48, 96); // #C43060
        let steel_gray = Rgb(107, 107, 128); // #6B6B80
        let dark_gray = Rgb(58, 58, 78); // #3A3A4E
        let deep_gray = Rgb(42, 42, 62); // #2A2A3E
        let selected_bg = Rgb(30, 30, 56); // #1E1E38
        let amber_warn = Rgb(212, 160, 80); // #D4A050

        Self {
            log_levels: LogLevelTheme {
                fatal: StyleEntry::fg_bold(bright_red),
                error: StyleEntry::fg(wine_red),
                warn: StyleEntry::fg(amber_warn),
                notice: StyleEntry::fg(lace_shadow),
                info: StyleEntry::fg(lace_white),
                debug: StyleEntry::fg(steel_gray),
                trace: StyleEntry::fg(dark_gray),
            },
            table: TableTheme {
                header: StyleEntry {
                    fg: Some(ThemeColor(black_dress)),
                    bg: Some(ThemeColor(lace_white)),
                    bold: Some(true),
                },
                header_unfocused: StyleEntry::fg_bg(steel_gray, dark_fabric),
                selected: StyleEntry::bg(selected_bg),
                separator: SeparatorStyle {
                    fg: Some(ThemeColor(deep_gray)),
                    bg: None,
                    char: "│".to_string(),
                },
                selected_search: StyleEntry::bg(Rgb(50, 20, 42)), // #32142A wine-tinted search selection
                selected_highlight: StyleEntry::bg(Rgb(35, 30, 55)), // #231E37 muted purple highlight
                search_match: StyleEntry::bg(Rgb(80, 25, 50)), // #501932 wine-red search match
                bookmark: StyleEntry::bg(Rgb(40, 20, 45)),     // #28142D deep plum bookmark
            },
            status_bar: StatusBarTheme {
                line1_bg: StyleEntry::fg_bg(lace_shadow, dark_fabric),
                line2_bg: StyleEntry::fg_bg(steel_gray, black_dress),
                mode_label: StyleEntry {
                    fg: Some(ThemeColor(black_dress)),
                    bg: Some(ThemeColor(lace_white)),
                    bold: Some(true),
                },
                mode_view: StyleEntry::fg_bg(black_dress, steel_gray),
                mode_follow: StyleEntry::fg_bg(black_dress, lace_shadow),
                density_normal: StyleEntry::fg(lace_shadow),
                density_hot: StyleEntry::fg(lace_white),
                density_label: StyleEntry::fg(dark_gray),
                density_tick: StyleEntry::fg(deep_gray),
                position: StyleEntry::fg(lace_white),
                cursor_marker: StyleEntry::fg(bright_red),
                ..StatusBarTheme::default()
            },
            search: SearchTheme {
                match_highlight: StyleEntry::fg_bg(black_dress, lace_shadow),
                current_match: StyleEntry::fg_bg(black_dress, lace_white),
            },
            dialog: DialogTheme {
                border: StyleEntry::fg(steel_gray),
                title: StyleEntry::fg_bold(lace_white),
                selected: StyleEntry::fg_bg(Rgb(250, 250, 245), selected_bg), // #FAFAF5
                text: StyleEntry::fg(lace_shadow),
                muted: StyleEntry::fg(dark_gray),
                ..DialogTheme::default()
            },
            detail_panel: DetailPanelTheme {
                border: StyleEntry::fg(deep_gray),
                field_name: StyleEntry::fg(lace_shadow),
                field_value: StyleEntry::fg(steel_gray),
                section_header: StyleEntry::fg_bold(lace_white),
            },
            input: InputTheme {
                prompt: StyleEntry::fg(lace_white),
                error: StyleEntry::fg(bright_red),
                ..InputTheme::default()
            },
            highlight_palette: vec![
                ThemeColor(lace_white),         // #F0EDE8
                ThemeColor(lace_shadow),        // #B0A8B9
                ThemeColor(bright_red),         // #C43060
                ThemeColor(wine_red),           // #8B2252
                ThemeColor(Rgb(104, 128, 160)), // #6880A0 cold blue
                ThemeColor(steel_gray),         // #6B6B80
            ],
            general: GeneralTheme {
                accent: StyleEntry::fg(wine_red),
                muted: StyleEntry::fg(dark_gray),
                border: StyleEntry::fg(deep_gray),
            },
            panel_tab: PanelTabTheme {
                focused: StyleEntry {
                    fg: Some(ThemeColor(black_dress)),
                    bg: Some(ThemeColor(lace_white)),
                    bold: Some(true),
                },
                unfocused: StyleEntry {
                    fg: Some(ThemeColor(steel_gray)),
                    bg: Some(ThemeColor(dark_fabric)),
                    bold: None,
                },
                bar_bg: StyleEntry::bg(black_dress),
            },
        }
    }

    /// Gyaru — Shibuya bold, gold and hot pink glamour.
    pub fn gyaru() -> Self {
        use Color::*;
        let dark_bronze = Rgb(26, 18, 8); // #1A1208
        let warm_brown = Rgb(42, 31, 20); // #2A1F14
        let hot_pink = Rgb(255, 36, 153); // #FF2499
        let gold = Rgb(255, 215, 0); // #FFD700
        let tan = Rgb(198, 134, 66); // #C68642
        let cream_white = Rgb(255, 240, 212); // #FFF0D4
        let leopard_dark = Rgb(139, 105, 20); // #8B6914
        let bronze = Rgb(166, 124, 82); // #A67C52
        let dark_gold = Rgb(107, 90, 40); // #6B5A28
        let warm_gray = Rgb(90, 72, 48); // #5A4830
        let deep_brown = Rgb(58, 42, 24); // #3A2A18
        let selected_bg = Rgb(58, 40, 24); // #3A2818

        Self {
            log_levels: LogLevelTheme {
                fatal: StyleEntry::fg_bold(hot_pink),
                error: StyleEntry::fg(Rgb(255, 105, 180)), // #FF69B4
                warn: StyleEntry::fg(Rgb(255, 224, 64)),   // #FFE040
                notice: StyleEntry::fg(gold),
                info: StyleEntry::fg(cream_white),
                debug: StyleEntry::fg(bronze),
                trace: StyleEntry::fg(warm_gray),
            },
            table: TableTheme {
                header: StyleEntry {
                    fg: Some(ThemeColor(dark_bronze)),
                    bg: Some(ThemeColor(gold)),
                    bold: Some(true),
                },
                header_unfocused: StyleEntry::fg_bg(bronze, warm_brown),
                selected: StyleEntry::bg(selected_bg),
                separator: SeparatorStyle {
                    fg: Some(ThemeColor(deep_brown)),
                    bg: None,
                    char: "│".to_string(),
                },
                selected_search: StyleEntry::bg(Rgb(70, 50, 20)), // #463214 warm bronze search selection
                selected_highlight: StyleEntry::bg(Rgb(55, 38, 18)), // #372612 dark bronze highlight
                search_match: StyleEntry::bg(Rgb(90, 65, 25)),       // #5A4119 golden search match
                bookmark: StyleEntry::bg(Rgb(60, 45, 15)),           // #3C2D0F deep gold bookmark
            },
            status_bar: StatusBarTheme {
                line1_bg: StyleEntry::fg_bg(tan, warm_brown),
                line2_bg: StyleEntry::fg_bg(bronze, dark_bronze),
                mode_label: StyleEntry {
                    fg: Some(ThemeColor(dark_bronze)),
                    bg: Some(ThemeColor(gold)),
                    bold: Some(true),
                },
                mode_view: StyleEntry::fg_bg(dark_bronze, bronze),
                mode_follow: StyleEntry::fg_bg(dark_bronze, hot_pink),
                density_normal: StyleEntry::fg(tan),
                density_hot: StyleEntry::fg(gold),
                density_label: StyleEntry::fg(dark_gold),
                density_tick: StyleEntry::fg(deep_brown),
                position: StyleEntry::fg(gold),
                cursor_marker: StyleEntry::fg(hot_pink),
                ..StatusBarTheme::default()
            },
            search: SearchTheme {
                match_highlight: StyleEntry::fg_bg(dark_bronze, tan),
                current_match: StyleEntry::fg_bg(dark_bronze, gold),
            },
            dialog: DialogTheme {
                border: StyleEntry::fg(tan),
                title: StyleEntry::fg_bold(gold),
                selected: StyleEntry::fg_bg(cream_white, selected_bg),
                text: StyleEntry::fg(bronze),
                muted: StyleEntry::fg(warm_gray),
                ..DialogTheme::default()
            },
            detail_panel: DetailPanelTheme {
                border: StyleEntry::fg(deep_brown),
                field_name: StyleEntry::fg(gold),
                field_value: StyleEntry::fg(tan),
                section_header: StyleEntry::fg_bold(hot_pink),
            },
            input: InputTheme {
                prompt: StyleEntry::fg(gold),
                error: StyleEntry::fg(hot_pink),
                ..InputTheme::default()
            },
            highlight_palette: vec![
                ThemeColor(hot_pink),           // #FF2499
                ThemeColor(gold),               // #FFD700
                ThemeColor(tan),                // #C68642
                ThemeColor(Rgb(255, 105, 180)), // #FF69B4
                ThemeColor(Rgb(255, 224, 64)),  // #FFE040
                ThemeColor(leopard_dark),       // #8B6914
            ],
            general: GeneralTheme {
                accent: StyleEntry::fg(gold),
                muted: StyleEntry::fg(warm_gray),
                border: StyleEntry::fg(deep_brown),
            },
            panel_tab: PanelTabTheme {
                focused: StyleEntry {
                    fg: Some(ThemeColor(dark_bronze)),
                    bg: Some(ThemeColor(gold)),
                    bold: Some(true),
                },
                unfocused: StyleEntry {
                    fg: Some(ThemeColor(bronze)),
                    bg: Some(ThemeColor(warm_brown)),
                    bold: None,
                },
                bar_bg: StyleEntry::bg(dark_bronze),
            },
        }
    }
}

#[cfg(test)]
#[path = "theme_tests.rs"]
mod theme_tests;
