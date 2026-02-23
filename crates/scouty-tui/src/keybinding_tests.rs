#[cfg(test)]
mod tests {
    use crate::keybinding::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_parse_key_simple_char() {
        let key = parse_key("j").unwrap();
        assert_eq!(key.code, KeyCode::Char('j'));
        assert_eq!(key.modifiers, KeyModifiers::NONE);
    }

    #[test]
    fn test_parse_key_uppercase() {
        let key = parse_key("G").unwrap();
        assert_eq!(key.code, KeyCode::Char('G'));
        assert_eq!(key.modifiers, KeyModifiers::NONE);
    }

    #[test]
    fn test_parse_key_ctrl_modifier() {
        let key = parse_key("ctrl+g").unwrap();
        assert_eq!(key.code, KeyCode::Char('g'));
        assert!(key.modifiers.contains(KeyModifiers::CONTROL));
    }

    #[test]
    fn test_parse_key_special_keys() {
        assert_eq!(parse_key("enter").unwrap().code, KeyCode::Enter);
        assert_eq!(parse_key("esc").unwrap().code, KeyCode::Esc);
        assert_eq!(parse_key("pagedown").unwrap().code, KeyCode::PageDown);
        assert_eq!(parse_key("home").unwrap().code, KeyCode::Home);
        assert_eq!(parse_key("tab").unwrap().code, KeyCode::Tab);
        assert_eq!(parse_key("space").unwrap().code, KeyCode::Char(' '));
    }

    #[test]
    fn test_parse_key_invalid() {
        assert!(parse_key("nonexistent").is_none());
    }

    #[test]
    fn test_default_keymap_has_quit() {
        let keymap = Keymap::default_keymap();
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(keymap.action(&key), Some(Action::Quit));
    }

    #[test]
    fn test_default_keymap_move_down() {
        let keymap = Keymap::default_keymap();
        let j = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let down = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        assert_eq!(keymap.action(&j), Some(Action::MoveDown));
        assert_eq!(keymap.action(&down), Some(Action::MoveDown));
    }

    #[test]
    fn test_default_keymap_ctrl_bindings() {
        let keymap = Keymap::default_keymap();
        let ctrl_g = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL);
        assert_eq!(keymap.action(&ctrl_g), Some(Action::GotoLine));
    }

    #[test]
    fn test_custom_config_override() {
        let mut config = KeybindingConfig::default();
        config.quit = Some(KeyOrKeys::Single("ctrl+q".to_string()));
        let keymap = Keymap::from_config(&config);

        // Custom: ctrl+q should quit
        let ctrl_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        assert_eq!(keymap.action(&ctrl_q), Some(Action::Quit));

        // Original 'q' should NOT quit (overridden)
        let q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(keymap.action(&q), None);
    }

    #[test]
    fn test_custom_config_multiple_keys() {
        let mut config = KeybindingConfig::default();
        config.quit = Some(KeyOrKeys::Multiple(vec![
            "q".to_string(),
            "ctrl+q".to_string(),
        ]));
        let keymap = Keymap::from_config(&config);

        let q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let ctrl_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        assert_eq!(keymap.action(&q), Some(Action::Quit));
        assert_eq!(keymap.action(&ctrl_q), Some(Action::Quit));
    }

    #[test]
    fn test_unknown_key_returns_none() {
        let keymap = Keymap::default_keymap();
        let key = KeyEvent::new(KeyCode::F(12), KeyModifiers::NONE);
        assert_eq!(keymap.action(&key), None);
    }

    #[test]
    fn test_partial_config_preserves_defaults() {
        let mut config = KeybindingConfig::default();
        // Only override quit, everything else should use defaults
        config.quit = Some(KeyOrKeys::Single("ctrl+q".to_string()));
        let keymap = Keymap::from_config(&config);

        // move_down should still work with defaults
        let j = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        assert_eq!(keymap.action(&j), Some(Action::MoveDown));
    }

    #[test]
    fn test_deserialization() {
        let yaml = r#"
quit: "ctrl+q"
move_down: ["j", "down"]
"#;
        let config: KeybindingConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.quit.is_some());
        assert!(config.move_down.is_some());
        assert!(config.filter.is_none()); // not specified — will use default
    }

    #[test]
    fn test_parse_key_plus() {
        let key = parse_key("+").unwrap();
        assert_eq!(key.code, KeyCode::Char('+'));
        assert_eq!(key.modifiers, KeyModifiers::NONE);

        let key2 = parse_key("plus").unwrap();
        assert_eq!(key2.code, KeyCode::Char('+'));
    }

    #[test]
    fn test_shift_normalized_for_char() {
        let keymap = Keymap::default_keymap();
        // Uppercase 'G' with SHIFT modifier should still match ScrollToBottom
        let g_shifted = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT);
        assert_eq!(keymap.action(&g_shifted), Some(Action::ScrollToBottom));
    }

    #[test]
    fn test_field_include_plus_key() {
        let keymap = Keymap::default_keymap();
        let plus = KeyEvent::new(KeyCode::Char('+'), KeyModifiers::NONE);
        assert_eq!(keymap.action(&plus), Some(Action::FieldInclude));
    }

    #[test]
    fn test_ctrl_s_export() {
        let keymap = Keymap::default_keymap();
        let ctrl_s = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
        assert_eq!(keymap.action(&ctrl_s), Some(Action::Export));
    }
}
