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
        for name in Theme::builtin_names() {
            let theme = Theme::builtin(name).unwrap();
            assert!(
                theme.table.header_unfocused.fg.is_some(),
                "preset '{}' missing header_unfocused fg",
                name
            );
        }
    }

    #[test]
    fn all_presets_header_matches_panel_tab_focused() {
        for name in Theme::builtin_names() {
            let theme = Theme::builtin(name).unwrap();
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

    #[test]
    fn all_builtins_have_non_empty_description() {
        for (name, desc) in Theme::builtin_catalog() {
            assert!(
                !desc.is_empty(),
                "builtin theme '{}' has empty description",
                name
            );
        }
    }
}

mod no_ansi16_tests {
    use super::super::{StyleEntry, Theme};
    use ratatui::style::Color;

    /// Returns true if the color is not an RGB color (and not Reset).
    /// This catches ANSI 16 named colors, indexed colors, etc.
    fn is_not_rgb(c: Color) -> bool {
        !matches!(c, Color::Rgb(_, _, _) | Color::Reset)
    }

    fn check_style(entry: &StyleEntry, name: &str, violations: &mut Vec<String>) {
        if let Some(fg) = entry.fg {
            if is_not_rgb(fg.0) {
                violations.push(format!("{name}.fg = {fg:?}"));
            }
        }
        if let Some(bg) = entry.bg {
            if is_not_rgb(bg.0) {
                violations.push(format!("{name}.bg = {bg:?}"));
            }
        }
    }

    #[test]
    fn default_theme_has_no_ansi16_colors() {
        let theme = Theme::default();
        let mut v = Vec::new();

        check_style(&theme.log_levels.fatal, "log_levels.fatal", &mut v);
        check_style(&theme.log_levels.error, "log_levels.error", &mut v);
        check_style(&theme.log_levels.warn, "log_levels.warn", &mut v);
        check_style(&theme.log_levels.notice, "log_levels.notice", &mut v);
        check_style(&theme.log_levels.info, "log_levels.info", &mut v);
        check_style(&theme.log_levels.debug, "log_levels.debug", &mut v);
        check_style(&theme.log_levels.trace, "log_levels.trace", &mut v);
        check_style(&theme.table.header, "table.header", &mut v);
        check_style(
            &theme.table.header_unfocused,
            "table.header_unfocused",
            &mut v,
        );
        check_style(&theme.table.selected, "table.selected", &mut v);
        check_style(
            &theme.table.selected_search,
            "table.selected_search",
            &mut v,
        );
        check_style(
            &theme.table.selected_highlight,
            "table.selected_highlight",
            &mut v,
        );
        check_style(&theme.table.search_match, "table.search_match", &mut v);
        check_style(&theme.table.bookmark, "table.bookmark", &mut v);
        check_style(
            &theme.table.separator.to_style_entry(),
            "table.separator",
            &mut v,
        );
        check_style(&theme.status_bar.line1_bg, "status_bar.line1_bg", &mut v);
        check_style(&theme.status_bar.line2_bg, "status_bar.line2_bg", &mut v);
        check_style(
            &theme.status_bar.density_hot,
            "status_bar.density_hot",
            &mut v,
        );
        check_style(
            &theme.status_bar.density_normal,
            "status_bar.density_normal",
            &mut v,
        );
        check_style(&theme.status_bar.position, "status_bar.position", &mut v);
        check_style(
            &theme.status_bar.mode_follow,
            "status_bar.mode_follow",
            &mut v,
        );
        check_style(&theme.status_bar.mode_view, "status_bar.mode_view", &mut v);
        check_style(
            &theme.status_bar.mode_label,
            "status_bar.mode_label",
            &mut v,
        );
        check_style(
            &theme.status_bar.command_mode_label,
            "status_bar.command_mode_label",
            &mut v,
        );
        check_style(
            &theme.status_bar.search_mode_label,
            "status_bar.search_mode_label",
            &mut v,
        );
        check_style(
            &theme.status_bar.shortcut_key,
            "status_bar.shortcut_key",
            &mut v,
        );
        check_style(
            &theme.status_bar.shortcut_sep,
            "status_bar.shortcut_sep",
            &mut v,
        );
        check_style(
            &theme.status_bar.density_label,
            "status_bar.density_label",
            &mut v,
        );
        check_style(
            &theme.status_bar.cursor_marker,
            "status_bar.cursor_marker",
            &mut v,
        );
        check_style(
            &theme.search.match_highlight,
            "search.match_highlight",
            &mut v,
        );
        check_style(&theme.search.current_match, "search.current_match", &mut v);
        check_style(&theme.dialog.border, "dialog.border", &mut v);
        check_style(&theme.dialog.title, "dialog.title", &mut v);
        check_style(&theme.dialog.selected, "dialog.selected", &mut v);
        check_style(&theme.dialog.text, "dialog.text", &mut v);
        check_style(&theme.dialog.muted, "dialog.muted", &mut v);
        check_style(&theme.dialog.background, "dialog.background", &mut v);
        check_style(&theme.dialog.accent, "dialog.accent", &mut v);
        check_style(
            &theme.detail_panel.field_name,
            "detail_panel.field_name",
            &mut v,
        );
        check_style(
            &theme.detail_panel.field_value,
            "detail_panel.field_value",
            &mut v,
        );
        check_style(&theme.detail_panel.border, "detail_panel.border", &mut v);
        check_style(
            &theme.detail_panel.section_header,
            "detail_panel.section_header",
            &mut v,
        );
        check_style(&theme.input.prompt, "input.prompt", &mut v);
        check_style(&theme.input.cursor, "input.cursor", &mut v);
        check_style(&theme.input.text, "input.text", &mut v);
        check_style(&theme.input.error, "input.error", &mut v);
        check_style(&theme.input.background, "input.background", &mut v);
        check_style(&theme.general.border, "general.border", &mut v);
        check_style(&theme.general.accent, "general.accent", &mut v);
        check_style(&theme.general.muted, "general.muted", &mut v);
        check_style(&theme.panel_tab.focused, "panel_tab.focused", &mut v);
        check_style(&theme.panel_tab.unfocused, "panel_tab.unfocused", &mut v);
        check_style(&theme.panel_tab.bar_bg, "panel_tab.bar_bg", &mut v);

        // Also check highlight_palette colors
        for (i, tc) in theme.highlight_palette.iter().enumerate() {
            if is_not_rgb(tc.0) {
                v.push(format!("highlight_palette[{i}] = {tc:?}"));
            }
        }

        assert!(
            v.is_empty(),
            "Default theme still contains non-RGB colors:\n{}",
            v.join("\n")
        );
    }
}
