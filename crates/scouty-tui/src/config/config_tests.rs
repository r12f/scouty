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

    #[test]
    fn test_deep_merge_scalars() {
        let base: serde_yaml::Value = serde_yaml::from_str("theme: default").unwrap();
        let overlay: serde_yaml::Value = serde_yaml::from_str("theme: dark").unwrap();
        let merged = super::super::deep_merge(base, overlay);
        let cfg: Config = serde_yaml::from_value(merged).unwrap();
        assert_eq!(cfg.theme, "dark");
    }

    #[test]
    fn test_deep_merge_maps() {
        let base: serde_yaml::Value =
            serde_yaml::from_str("general:\n  follow_on_pipe: true\n  detail_panel_ratio: 0.3")
                .unwrap();
        let overlay: serde_yaml::Value =
            serde_yaml::from_str("general:\n  detail_panel_ratio: 0.5").unwrap();
        let merged = super::super::deep_merge(base, overlay);
        let cfg: Config = serde_yaml::from_value(merged).unwrap();
        // Deep merge: follow_on_pipe preserved, detail_panel_ratio overridden
        assert!(cfg.general.follow_on_pipe);
        assert!((cfg.general.detail_panel_ratio - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deep_merge_list_replaces() {
        let base: serde_yaml::Value =
            serde_yaml::from_str("default_paths:\n  - /var/log/syslog").unwrap();
        let overlay: serde_yaml::Value =
            serde_yaml::from_str("default_paths:\n  - /tmp/a.log\n  - /tmp/b.log").unwrap();
        let merged = super::super::deep_merge(base, overlay);
        let cfg: Config = serde_yaml::from_value(merged).unwrap();
        // List is fully replaced, not appended
        assert_eq!(cfg.default_paths, vec!["/tmp/a.log", "/tmp/b.log"]);
    }

    #[test]
    fn test_deep_merge_null_resets() {
        let base: serde_yaml::Value = serde_yaml::from_str("theme: dark").unwrap();
        let overlay: serde_yaml::Value = serde_yaml::from_str("theme: null").unwrap();
        let merged = super::super::deep_merge(base, overlay);
        // theme key removed → deserialization uses default
        let cfg: Config = serde_yaml::from_value(merged).unwrap();
        assert_eq!(cfg.theme, "default");
    }

    #[test]
    fn test_local_config_path_exists() {
        // Verify local_config_path returns ./scouty.yaml
        let path = super::super::local_config_path();
        assert_eq!(path, std::path::PathBuf::from("./scouty.yaml"));
    }

    #[test]
    fn test_local_config_deep_merge_priority() {
        // Simulate local config overriding user config via deep_merge
        let user_yaml: serde_yaml::Value =
            serde_yaml::from_str("theme: dark\ndefault_paths:\n  - /var/log/syslog").unwrap();
        let local_yaml: serde_yaml::Value =
            serde_yaml::from_str("theme: solarized\ndefault_paths:\n  - ./logs/*.log").unwrap();
        let merged = super::super::deep_merge(user_yaml, local_yaml);
        let cfg: Config = serde_yaml::from_value(merged).unwrap();
        assert_eq!(cfg.theme, "solarized"); // local overrides user
        assert_eq!(cfg.default_paths, vec!["./logs/*.log"]); // list replaced
    }

    #[test]
    fn test_local_config_overridden_by_cli_merge() {
        // CLI config layer should override local config layer
        let local_yaml: serde_yaml::Value =
            serde_yaml::from_str("theme: solarized").unwrap();
        let cli_yaml: serde_yaml::Value =
            serde_yaml::from_str("theme: dark").unwrap();
        let merged = super::super::deep_merge(local_yaml, cli_yaml);
        let cfg: Config = serde_yaml::from_value(merged).unwrap();
        assert_eq!(cfg.theme, "dark"); // CLI wins
    }

    #[test]
    fn test_local_config_partial_deep_merge() {
        // Local config only overrides what it specifies
        let base = serde_yaml::to_value(Config::default()).unwrap();
        let local_yaml: serde_yaml::Value =
            serde_yaml::from_str("general:\n  detail_panel_ratio: 0.5").unwrap();
        let merged = super::super::deep_merge(base, local_yaml);
        let cfg: Config = serde_yaml::from_value(merged).unwrap();
        assert!((cfg.general.detail_panel_ratio - 0.5).abs() < f64::EPSILON);
        assert!(cfg.general.follow_on_pipe); // default preserved
        assert_eq!(cfg.theme, "default"); // default preserved
    }

    #[test]
    fn test_load_config_layered_with_file() {
        let dir = std::env::temp_dir().join("scouty_test_layered");
        let _ = std::fs::create_dir_all(&dir);
        let config_path = dir.join("override.yaml");
        std::fs::write(&config_path, "theme: solarized\n").unwrap();
        let cfg = super::super::load_config_layered(Some(config_path.to_str().unwrap()));
        assert_eq!(cfg.theme, "solarized");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
