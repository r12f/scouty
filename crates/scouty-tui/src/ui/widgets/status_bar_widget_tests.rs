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
        assert_eq!(label, "[500ms/█]");
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
        assert_eq!(label, "[5s/█]");
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
        assert_eq!(label, "[2m/█]");
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
        assert_eq!(label, "[2h/█]");
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
        assert_eq!(label, "[5.5s/█]");
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
        assert_eq!(label, "[2.5m/█]");
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
        assert_eq!(label, "[1.5h/█]");
    }
}
