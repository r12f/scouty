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
        };
        let label = StatusBarWidget::time_per_column_label(&cache).unwrap();
        assert_eq!(label, "[█=5m]");
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
        };
        let label = StatusBarWidget::time_per_column_label(&cache).unwrap();
        assert_eq!(label, "[█=15s]");
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
        };
        let label = StatusBarWidget::time_per_column_label(&cache).unwrap();
        assert_eq!(label, "[█=2h]");
    }

    #[test]
    fn test_snap_to_standard_intervals() {
        // Below 5s: no snap
        assert_eq!(StatusBarWidget::snap_to_standard(500.0), 500.0);
        assert_eq!(StatusBarWidget::snap_to_standard(3000.0), 3000.0);
        // Seconds
        assert_eq!(StatusBarWidget::snap_to_standard(5_000.0), 5_000.0);
        assert_eq!(StatusBarWidget::snap_to_standard(8_000.0), 15_000.0);
        assert_eq!(StatusBarWidget::snap_to_standard(20_000.0), 30_000.0);
        // Minutes
        assert_eq!(StatusBarWidget::snap_to_standard(40_000.0), 300_000.0); // 40s → 5m
        assert_eq!(StatusBarWidget::snap_to_standard(480_000.0), 900_000.0); // 8m → 15m
        assert_eq!(StatusBarWidget::snap_to_standard(2_700_000.0), 3_600_000.0); // 45m → 1h
                                                                                 // Hours
        assert_eq!(StatusBarWidget::snap_to_standard(3_600_000.0), 3_600_000.0); // 1h
        assert_eq!(StatusBarWidget::snap_to_standard(5_000_000.0), 7_200_000.0); // → 2h
        assert_eq!(
            StatusBarWidget::snap_to_standard(10_000_000.0),
            21_600_000.0
        ); // → 6h
    }
}
