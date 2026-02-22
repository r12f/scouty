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
}
