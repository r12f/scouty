mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn default_theme_creates_without_panic() {
        let theme = Theme::default();
        assert_eq!(theme.log_levels.fatal.bold, Some(true));
    }

    #[test]
    fn theme_color_roundtrip_named() {
        let tc = ThemeColor(Color::Cyan);
        let s: String = tc.into();
        assert_eq!(s, "cyan");
        let back = ThemeColor::try_from(s).unwrap();
        assert_eq!(back.0, Color::Cyan);
    }

    #[test]
    fn theme_color_roundtrip_hex() {
        let tc = ThemeColor(Color::Rgb(0xFF, 0x66, 0x00));
        let s: String = tc.into();
        assert_eq!(s, "#FF6600");
        let back = ThemeColor::try_from(s).unwrap();
        assert_eq!(back.0, Color::Rgb(0xFF, 0x66, 0x00));
    }

    #[test]
    fn theme_color_256() {
        let tc = ThemeColor::try_from("color(123)".to_string()).unwrap();
        assert_eq!(tc.0, Color::Indexed(123));
    }

    #[test]
    fn style_entry_to_style() {
        let entry = StyleEntry::fg_bg(Color::Red, Color::Blue);
        let style = entry.to_style();
        assert_eq!(style.fg, Some(Color::Red));
        assert_eq!(style.bg, Some(Color::Blue));
    }

    #[test]
    fn theme_yaml_roundtrip() {
        let theme = Theme::default();
        let yaml = serde_yaml::to_string(&theme).unwrap();
        let back: Theme = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(theme, back);
    }

    #[test]
    fn log_level_style_returns_correct_colors() {
        let theme = Theme::default();
        let style = theme.log_level_style(Some(&scouty::record::LogLevel::Fatal));
        assert_eq!(style.fg, Some(Color::Red));
    }

    #[test]
    fn highlight_color_wraps() {
        let theme = Theme::default();
        let len = theme.highlight_palette.colors.len();
        assert_eq!(theme.highlight_color(0), theme.highlight_color(len));
    }
}
