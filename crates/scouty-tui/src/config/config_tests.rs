#[cfg(test)]
mod tests {
    use super::super::Config;

    #[test]
    fn default_config() {
        let cfg = Config::default();
        assert_eq!(cfg.theme, "default");
    }

    #[test]
    fn config_from_yaml_partial() {
        let yaml = r#"theme: "solarized""#;
        let cfg: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.theme, "solarized");
    }

    #[test]
    fn config_from_empty_yaml() {
        let cfg: Config = serde_yaml::from_str("{}").unwrap();
        assert_eq!(cfg.theme, "default");
    }

    #[test]
    fn resolve_default_theme() {
        let cfg = Config::default();
        let theme = super::super::resolve_theme(&cfg, None);
        assert_eq!(theme.highlight_palette.len(), 6);
    }

    #[test]
    fn test_expand_default_paths_empty() {
        let result = super::super::expand_default_paths(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_expand_default_paths_no_match() {
        let result =
            super::super::expand_default_paths(&["/nonexistent/path/*.xyz123".to_string()]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_expand_default_paths_glob() {
        let dir = std::env::temp_dir().join("scouty_test_glob");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("test.log"), "hello").unwrap();
        let pattern = format!("{}/*.log", dir.display());
        let result = super::super::expand_default_paths(&[pattern]);
        assert!(
            result.iter().any(|p| p.contains("test.log")),
            "expected test.log in results: {:?}",
            result
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_default_paths_in_config() {
        let cfg = Config::default();
        assert!(cfg.default_paths.is_empty());
    }
}
