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

    #[test]
    fn landmine_theme_preset() {
        let theme = Theme::landmine();
        assert_eq!(theme.highlight_palette.len(), 6);
        assert_eq!(theme.table.separator.separator_char(), "♡");
    }

    #[test]
    fn builtin_returns_landmine() {
        let theme = Theme::builtin("landmine");
        assert!(theme.is_some());
        assert_eq!(theme.unwrap().table.separator.separator_char(), "♡");
    }

    #[test]
    fn default_separator_char() {
        let theme = Theme::default();
        assert_eq!(theme.table.separator.separator_char(), "│");
    }

    #[test]
    fn separator_char_from_yaml() {
        let yaml = "table:\n  separator:\n    fg: \"#FF0000\"\n    char: \"|\"\n";
        let theme = Theme::from_yaml(yaml).unwrap();
        assert_eq!(theme.table.separator.separator_char(), "|");
    }

    #[test]
    fn all_presets_have_header_unfocused() {
        let themes = vec![
            ("default", Theme::default()),
            ("dark", Theme::dark()),
            ("light", Theme::light()),
            ("solarized", Theme::solarized()),
            ("landmine", Theme::landmine()),
        ];
        for (name, theme) in &themes {
            assert!(
                theme.table.header_unfocused.fg.is_some(),
                "preset '{}' missing header_unfocused fg",
                name
            );
        }
    }

    #[test]
    fn all_presets_header_matches_panel_tab_focused() {
        let themes = vec![
            ("default", Theme::default()),
            ("dark", Theme::dark()),
            ("light", Theme::light()),
            ("solarized", Theme::solarized()),
            ("landmine", Theme::landmine()),
        ];
        for (name, theme) in &themes {
            assert_eq!(
                theme.table.header.fg, theme.panel_tab.focused.fg,
                "preset '{}': table.header.fg should match panel_tab.focused.fg",
                name
            );
            assert_eq!(
                theme.table.header.bg, theme.panel_tab.focused.bg,
                "preset '{}': table.header.bg should match panel_tab.focused.bg",
                name
            );
        }
    }
}
