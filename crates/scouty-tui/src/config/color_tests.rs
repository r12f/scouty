#[cfg(test)]
mod tests {
    use super::super::ThemeColor;
    use ratatui::style::Color;

    #[test]
    fn parse_named_colors() {
        assert_eq!(ThemeColor::parse("red").unwrap().0, Color::Red);
        assert_eq!(ThemeColor::parse("dark_gray").unwrap().0, Color::DarkGray);
        assert_eq!(ThemeColor::parse("DarkGray").unwrap().0, Color::DarkGray);
    }

    #[test]
    fn parse_hex_rgb() {
        assert_eq!(
            ThemeColor::parse("#FF6600").unwrap().0,
            Color::Rgb(255, 102, 0)
        );
        assert_eq!(ThemeColor::parse("#000000").unwrap().0, Color::Rgb(0, 0, 0));
    }

    #[test]
    fn parse_256_color() {
        assert_eq!(
            ThemeColor::parse("color(123)").unwrap().0,
            Color::Indexed(123)
        );
    }

    #[test]
    fn parse_invalid() {
        assert!(ThemeColor::parse("notacolor").is_none());
        assert!(ThemeColor::parse("#GG0000").is_none());
        assert!(ThemeColor::parse("color(999)").is_none());
    }
}
