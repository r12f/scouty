//! Color parsing: named colors, hex RGB (#RRGGBB), 256-color index (color(N)).

use ratatui::style::Color;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A wrapper around ratatui `Color` that supports YAML deserialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeColor(pub Color);

impl ThemeColor {
    pub fn parse(s: &str) -> Option<ThemeColor> {
        let s = s.trim();
        // Hex RGB
        if let Some(hex) = s.strip_prefix('#') {
            if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                return Some(ThemeColor(Color::Rgb(r, g, b)));
            }
            return None;
        }
        // 256-color: color(N)
        if let Some(inner) = s.strip_prefix("color(").and_then(|s| s.strip_suffix(')')) {
            let idx: u8 = inner.trim().parse().ok()?;
            return Some(ThemeColor(Color::Indexed(idx)));
        }
        // Named colors
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
            _ => return None,
        };
        Some(ThemeColor(color))
    }
}

impl From<Color> for ThemeColor {
    fn from(c: Color) -> Self {
        ThemeColor(c)
    }
}

impl From<ThemeColor> for Color {
    fn from(tc: ThemeColor) -> Self {
        tc.0
    }
}

impl Default for ThemeColor {
    fn default() -> Self {
        ThemeColor(Color::Reset)
    }
}

impl<'de> Deserialize<'de> for ThemeColor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        ThemeColor::parse(&s).ok_or_else(|| serde::de::Error::custom(format!("invalid color: {s}")))
    }
}

impl Serialize for ThemeColor {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            Color::Rgb(r, g, b) => serializer.serialize_str(&format!("#{r:02X}{g:02X}{b:02X}")),
            Color::Indexed(i) => serializer.serialize_str(&format!("color({i})")),
            Color::Black => serializer.serialize_str("black"),
            Color::Red => serializer.serialize_str("red"),
            Color::Green => serializer.serialize_str("green"),
            Color::Yellow => serializer.serialize_str("yellow"),
            Color::Blue => serializer.serialize_str("blue"),
            Color::Magenta => serializer.serialize_str("magenta"),
            Color::Cyan => serializer.serialize_str("cyan"),
            Color::White => serializer.serialize_str("white"),
            Color::Gray => serializer.serialize_str("gray"),
            Color::DarkGray => serializer.serialize_str("dark_gray"),
            Color::LightRed => serializer.serialize_str("light_red"),
            Color::LightGreen => serializer.serialize_str("light_green"),
            Color::LightYellow => serializer.serialize_str("light_yellow"),
            Color::LightBlue => serializer.serialize_str("light_blue"),
            Color::LightMagenta => serializer.serialize_str("light_magenta"),
            Color::LightCyan => serializer.serialize_str("light_cyan"),
            _ => serializer.serialize_str("reset"),
        }
    }
}

#[cfg(test)]
#[path = "color_tests.rs"]
mod color_tests;
