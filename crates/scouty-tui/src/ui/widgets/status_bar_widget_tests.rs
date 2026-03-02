#[cfg(test)]
mod tests {
    use crate::ui::widgets::status_bar_widget::StatusBarWidget;
    use crate::ui::{dispatch_key, ComponentResult, UiComponent};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn test_enable_jk_navigation() {
        let widget = StatusBarWidget;
        assert!(!widget.enable_jk_navigation());
    }

    #[test]
    fn test_esc_closes() {
        let mut widget = StatusBarWidget;
        assert_eq!(
            dispatch_key(&mut widget, key(KeyCode::Esc)),
            ComponentResult::Close
        );
    }

    #[test]
    fn test_chars_ignored() {
        let mut widget = StatusBarWidget;
        assert_eq!(
            dispatch_key(&mut widget, key(KeyCode::Char('x'))),
            ComponentResult::Ignored
        );
    }

    #[test]
    fn test_time_per_column_label_milliseconds() {
        let cache = crate::app::DensityCache {
            braille_text: String::new(),
            num_buckets: 100,
            min_ts: chrono::Utc::now(),
            max_ts: chrono::Utc::now() + chrono::Duration::milliseconds(50_000),
            filter_version: 0,
            chart_width: 50,
            density_source: crate::app::DensitySource::All,
        };
        let label = StatusBarWidget::time_per_column_label(&cache).unwrap();
        assert_eq!(label, "[█=500ms]");
    }

    #[test]
    fn test_time_per_column_label_seconds() {
        let cache = crate::app::DensityCache {
            braille_text: String::new(),
            num_buckets: 100,
            min_ts: chrono::Utc::now(),
            max_ts: chrono::Utc::now() + chrono::Duration::seconds(500),
            filter_version: 0,
            chart_width: 50,
            density_source: crate::app::DensitySource::All,
        };
        let label = StatusBarWidget::time_per_column_label(&cache).unwrap();
        assert_eq!(label, "[█=5s]");
    }

    #[test]
    fn test_time_per_column_label_minutes() {
        let cache = crate::app::DensityCache {
            braille_text: String::new(),
            num_buckets: 100,
            min_ts: chrono::Utc::now(),
            max_ts: chrono::Utc::now() + chrono::Duration::minutes(200),
            filter_version: 0,
            chart_width: 50,
            density_source: crate::app::DensitySource::All,
        };
        let label = StatusBarWidget::time_per_column_label(&cache).unwrap();
        assert_eq!(label, "[█=2m]");
    }

    #[test]
    fn test_time_per_column_label_hours() {
        let cache = crate::app::DensityCache {
            braille_text: String::new(),
            num_buckets: 10,
            min_ts: chrono::Utc::now(),
            max_ts: chrono::Utc::now() + chrono::Duration::hours(20),
            filter_version: 0,
            chart_width: 50,
            density_source: crate::app::DensitySource::All,
        };
        let label = StatusBarWidget::time_per_column_label(&cache).unwrap();
        assert_eq!(label, "[█=2h]");
    }

    #[test]
    fn test_time_per_column_label_same_timestamps() {
        let now = chrono::Utc::now();
        let cache = crate::app::DensityCache {
            braille_text: String::new(),
            num_buckets: 100,
            min_ts: now,
            max_ts: now,
            filter_version: 0,
            chart_width: 50,
            density_source: crate::app::DensitySource::All,
        };
        assert!(StatusBarWidget::time_per_column_label(&cache).is_none());
    }

    #[test]
    fn test_time_per_column_label_decimal_seconds() {
        let now = chrono::Utc::now();
        let cache = crate::app::DensityCache {
            braille_text: String::new(),
            num_buckets: 10,
            min_ts: now,
            max_ts: now + chrono::Duration::milliseconds(55_000),
            filter_version: 0,
            chart_width: 50,
            density_source: crate::app::DensitySource::All,
        };
        let label = StatusBarWidget::time_per_column_label(&cache).unwrap();
        assert_eq!(label, "[█=10s]");
    }

    #[test]
    fn test_time_per_column_label_decimal_minutes() {
        let now = chrono::Utc::now();
        let cache = crate::app::DensityCache {
            braille_text: String::new(),
            num_buckets: 10,
            min_ts: now,
            max_ts: now + chrono::Duration::minutes(25),
            filter_version: 0,
            chart_width: 50,
            density_source: crate::app::DensitySource::All,
        };
        let label = StatusBarWidget::time_per_column_label(&cache).unwrap();
        assert_eq!(label, "[█=5m]");
    }

    #[test]
    fn test_time_per_column_label_decimal_hours() {
        let now = chrono::Utc::now();
        let cache = crate::app::DensityCache {
            braille_text: String::new(),
            num_buckets: 10,
            min_ts: now,
            max_ts: now + chrono::Duration::hours(15),
            filter_version: 0,
            chart_width: 50,
            density_source: crate::app::DensitySource::All,
        };
        let label = StatusBarWidget::time_per_column_label(&cache).unwrap();
        assert_eq!(label, "[█=2h]");
    }

    #[test]
    fn test_snap_to_standard_sub_second() {
        // Sub-second snapping to human-friendly intervals
        assert_eq!(StatusBarWidget::snap_to_standard(0.5), 1.0); // 0.5ms -> 1ms
        assert_eq!(StatusBarWidget::snap_to_standard(1.0), 1.0); // 1ms exact
        assert_eq!(StatusBarWidget::snap_to_standard(1.5), 2.0); // 1.5ms -> 2ms
        assert_eq!(StatusBarWidget::snap_to_standard(3.0), 5.0); // 3ms -> 5ms
        assert_eq!(StatusBarWidget::snap_to_standard(7.0), 10.0); // 7ms -> 10ms
        assert_eq!(StatusBarWidget::snap_to_standard(15.0), 20.0); // 15ms -> 20ms
        assert_eq!(StatusBarWidget::snap_to_standard(30.0), 50.0); // 30ms -> 50ms
        assert_eq!(StatusBarWidget::snap_to_standard(80.0), 100.0); // 80ms -> 100ms
        assert_eq!(StatusBarWidget::snap_to_standard(150.0), 200.0); // 150ms -> 200ms
        assert_eq!(StatusBarWidget::snap_to_standard(327.0), 500.0); // 327ms -> 500ms (the reported bug case)
        assert_eq!(StatusBarWidget::snap_to_standard(500.0), 500.0); // 500ms exact
        assert_eq!(StatusBarWidget::snap_to_standard(800.0), 1_000.0); // 800ms -> 1s
    }

    #[test]
    fn test_snap_to_standard_seconds() {
        assert_eq!(StatusBarWidget::snap_to_standard(1_000.0), 1_000.0); // 1s exact
        assert_eq!(StatusBarWidget::snap_to_standard(1_500.0), 2_000.0); // 1.5s -> 2s
        assert_eq!(StatusBarWidget::snap_to_standard(3_000.0), 5_000.0); // 3s -> 5s
        assert_eq!(StatusBarWidget::snap_to_standard(5_000.0), 5_000.0); // 5s exact
        assert_eq!(StatusBarWidget::snap_to_standard(8_000.0), 10_000.0); // 8s -> 10s
        assert_eq!(StatusBarWidget::snap_to_standard(12_000.0), 15_000.0); // 12s -> 15s
        assert_eq!(StatusBarWidget::snap_to_standard(20_000.0), 30_000.0); // 20s -> 30s
    }

    #[test]
    fn test_snap_to_standard_minutes() {
        assert_eq!(StatusBarWidget::snap_to_standard(40_000.0), 60_000.0); // 40s -> 1m
        assert_eq!(StatusBarWidget::snap_to_standard(60_000.0), 60_000.0); // 1m exact
        assert_eq!(StatusBarWidget::snap_to_standard(90_000.0), 120_000.0); // 1.5m -> 2m
        assert_eq!(StatusBarWidget::snap_to_standard(180_000.0), 300_000.0); // 3m -> 5m
        assert_eq!(StatusBarWidget::snap_to_standard(480_000.0), 600_000.0); // 8m -> 10m
        assert_eq!(StatusBarWidget::snap_to_standard(700_000.0), 900_000.0); // ~11.7m -> 15m
        assert_eq!(StatusBarWidget::snap_to_standard(2_700_000.0), 3_600_000.0);
        // 45m -> 1h
    }

    #[test]
    fn test_snap_to_standard_hours() {
        assert_eq!(StatusBarWidget::snap_to_standard(3_600_000.0), 3_600_000.0); // 1h exact
        assert_eq!(StatusBarWidget::snap_to_standard(5_000_000.0), 7_200_000.0); // -> 2h
        assert_eq!(
            StatusBarWidget::snap_to_standard(10_000_000.0),
            21_600_000.0
        ); // -> 6h
    }

    #[test]
    fn test_snap_to_standard_beyond_24h() {
        let big = 100_000_000.0; // ~27.8h
        assert_eq!(StatusBarWidget::snap_to_standard(big), big); // returned as-is
    }

    /// Helper: return the shortcut hints via the production method.
    #[test]
    fn test_view_mode_hints_simplified() {
        let hints = StatusBarWidget::shortcut_hints(false, crate::panel::PanelId::Detail);
        // j/k merged, -/= merged, no separate Exclude/Include/ExclField/InclField
        assert_eq!(hints[0], ("j/k", "↑↓"));
        assert_eq!(hints[3], ("-/=", "Exclude/Include"));
        assert_eq!(hints.len(), 6);
    }

    #[test]
    fn test_detail_panel_hints_simplified() {
        let hints = StatusBarWidget::shortcut_hints(true, crate::panel::PanelId::Detail);
        assert_eq!(hints[0], ("←/→", "Fold"));
        assert_eq!(hints[1], ("H/L", "All"));
        assert_eq!(hints[2], ("Tab/S-Tab", "Switch"));
        assert_eq!(hints[3], ("z", "Max"));
        assert_eq!(hints[4], ("Esc", "Close"));
        assert_eq!(hints.len(), 5);
    }

    #[test]
    fn test_region_panel_hints_simplified() {
        let hints = StatusBarWidget::shortcut_hints(true, crate::panel::PanelId::Region);
        assert_eq!(hints[0], ("j/k", "↑↓"));
        assert_eq!(hints[1], ("Tab/S-Tab", "Switch"));
        assert_eq!(hints.len(), 4);
    }

    #[test]
    fn test_stats_panel_hints_simplified() {
        let hints = StatusBarWidget::shortcut_hints(true, crate::panel::PanelId::Stats);
        assert_eq!(hints.len(), 3);
        assert_eq!(hints[0], ("Tab/S-Tab", "Switch"));
    }
}

#[cfg(test)]
mod bookmark_display_tests {
    use crate::ui::widgets::status_bar_widget::StatusBarWidget;
    use unicode_width::UnicodeWidthStr;

    #[test]
    fn test_build_right_text_no_bookmarks() {
        let result = StatusBarWidget::build_right_text("1/100", 0);
        assert_eq!(result, " 1/100 ");
    }

    #[test]
    fn test_build_right_text_with_bookmarks() {
        let result = StatusBarWidget::build_right_text("1/100", 3);
        assert_eq!(result, " ★3 | 1/100 ");
    }

    #[test]
    fn test_build_right_text_with_bookmarks_single() {
        let result = StatusBarWidget::build_right_text("42/500", 1);
        assert_eq!(result, " ★1 | 42/500 ");
    }

    #[test]
    fn test_build_right_text_trims_position() {
        // position passed with surrounding spaces should be trimmed in bookmark branch
        let result = StatusBarWidget::build_right_text(" 1/100 ", 2);
        assert_eq!(result, " ★2 | 1/100 ");
    }

    #[test]
    fn test_build_right_text_unicode_width_correct() {
        // ★ is a fullwidth-ish character; verify display width is computed correctly
        let result = StatusBarWidget::build_right_text("1/10", 5);
        let display_width = UnicodeWidthStr::width(result.as_str());
        // " ★5 | 1/10 " — each char's display width should sum correctly
        // ' '=1, '★'=1, '5'=1, ' '=1, '|'=1, ' '=1, '1'=1, '/'=1, '1'=1, '0'=1, ' '=1 = 12
        // but ★ may be width 1 or 2 depending on unicode-width version
        assert!(display_width > 0);
        // Crucially, display_width should differ from byte length since ★ is multi-byte
        assert_ne!(
            display_width,
            result.len(),
            "display width should differ from byte length due to ★"
        );
    }

    #[test]
    fn test_build_right_text_pipe_separator_present() {
        let result = StatusBarWidget::build_right_text("1/100", 2);
        assert!(
            result.contains(" | "),
            "should contain pipe separator when bookmarks present"
        );
    }

    #[test]
    fn test_build_right_text_pipe_separator_absent() {
        let result = StatusBarWidget::build_right_text("1/100", 0);
        assert!(
            !result.contains("|"),
            "should not contain pipe when no bookmarks"
        );
    }

    #[test]
    fn test_build_right_text_star_format() {
        let result = StatusBarWidget::build_right_text("1/100", 42);
        assert!(result.contains("★42"), "should contain ★N format");
    }
}
