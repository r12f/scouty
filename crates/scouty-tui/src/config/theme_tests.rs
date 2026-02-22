#[cfg(test)]
mod tests {
    use super::super::Theme;

    #[test]
    fn default_theme_is_valid() {
        let theme = Theme::default();
        // Check highlight palette has 6 colors
        assert_eq!(theme.highlight_palette.len(), 6);
    }

    #[test]
    fn partial_yaml_override() {
        let yaml = "log_levels:\n  fatal:\n    fg: \"#FF0000\"\n    bold: true\n";
        let theme = Theme::from_yaml(yaml).unwrap();
        // Overridden field
        assert!(theme.log_levels.fatal.bold == Some(true));
        // Non-overridden fields keep defaults
        assert_eq!(theme.highlight_palette.len(), 6);
    }

    #[test]
    fn empty_yaml_gives_defaults() {
        let theme = Theme::from_yaml("{}").unwrap();
        assert_eq!(theme.highlight_palette.len(), 6);
    }
}
