#[cfg(test)]
mod tests {
    use super::super::{sanitize_name, FilterPreset, FilterPresetEntry};
    use std::sync::Mutex;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    /// Override HOME for testing, serialized to avoid races.
    fn with_test_home<F: FnOnce()>(test_name: &str, f: F) {
        let _guard = TEST_LOCK.lock().unwrap();
        let dir = std::env::temp_dir().join(format!("scouty_preset_{}", test_name));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let old_home = std::env::var("HOME").ok();
        unsafe { std::env::set_var("HOME", &dir) };
        f();
        if let Some(h) = old_home {
            unsafe { std::env::set_var("HOME", h) };
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("my-preset"), "my-preset");
        assert_eq!(sanitize_name("foo bar"), "foo_bar");
        assert_eq!(sanitize_name("test/bad"), "test_bad");
        assert_eq!(sanitize_name("under_score"), "under_score");
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        with_test_home("roundtrip", || {
            let preset = FilterPreset {
                filters: vec![
                    FilterPresetEntry {
                        expr: "level == \"ERROR\"".to_string(),
                        exclude: false,
                    },
                    FilterPresetEntry {
                        expr: "timeout".to_string(),
                        exclude: true,
                    },
                ],
                level_filter: Some("WARN+".to_string()),
            };

            super::super::save_preset("test-preset", &preset).unwrap();
            let loaded = super::super::load_preset("test-preset").unwrap();
            assert_eq!(loaded.filters.len(), 2);
            assert_eq!(loaded.filters[0].expr, "level == \"ERROR\"");
            assert!(!loaded.filters[0].exclude);
            assert_eq!(loaded.filters[1].expr, "timeout");
            assert!(loaded.filters[1].exclude);
            assert_eq!(loaded.level_filter, Some("WARN+".to_string()));
        });
    }

    #[test]
    fn test_list_presets() {
        with_test_home("list", || {
            let p1 = FilterPreset {
                filters: vec![],
                level_filter: None,
            };
            super::super::save_preset("alpha", &p1).unwrap();
            super::super::save_preset("beta", &p1).unwrap();

            let names = super::super::list_presets();
            assert!(names.contains(&"alpha".to_string()));
            assert!(names.contains(&"beta".to_string()));
        });
    }

    #[test]
    fn test_delete_preset() {
        with_test_home("delete", || {
            let p = FilterPreset {
                filters: vec![],
                level_filter: None,
            };
            super::super::save_preset("to-delete", &p).unwrap();
            assert!(super::super::list_presets().contains(&"to-delete".to_string()));

            super::super::delete_preset("to-delete").unwrap();
            assert!(!super::super::list_presets().contains(&"to-delete".to_string()));
        });
    }

    #[test]
    fn test_load_nonexistent() {
        with_test_home("nonexistent", || {
            let result = super::super::load_preset("nonexistent");
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_empty_preset() {
        with_test_home("empty", || {
            let preset = FilterPreset {
                filters: vec![],
                level_filter: None,
            };
            super::super::save_preset("empty", &preset).unwrap();
            let loaded = super::super::load_preset("empty").unwrap();
            assert!(loaded.filters.is_empty());
            assert!(loaded.level_filter.is_none());
        });
    }

    #[test]
    fn test_round_trip_with_expressions() {
        with_test_home("roundtrip_expr", || {
            let preset = FilterPreset {
                filters: vec![
                    FilterPresetEntry {
                        expr: "message contains \"timeout\"".to_string(),
                        exclude: true,
                    },
                    FilterPresetEntry {
                        expr: "message contains \"error\"".to_string(),
                        exclude: false,
                    },
                ],
                level_filter: Some("WARN+".to_string()),
            };
            super::super::save_preset("roundtrip_expr", &preset).unwrap();
            let loaded = super::super::load_preset("roundtrip_expr").unwrap();
            assert_eq!(loaded.filters.len(), 2);
            assert_eq!(loaded.filters[0].expr, "message contains \"timeout\"");
            assert!(loaded.filters[0].exclude);
            assert_eq!(loaded.filters[1].expr, "message contains \"error\"");
            assert!(!loaded.filters[1].exclude);
            assert_eq!(loaded.level_filter, Some("WARN+".to_string()));
        });
    }
}
