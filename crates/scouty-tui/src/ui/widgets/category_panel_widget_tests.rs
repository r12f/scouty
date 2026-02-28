#[cfg(test)]
mod tests {
    use crate::ui::widgets::category_panel_widget::*;

    #[test]
    fn test_format_count_zero() {
        assert_eq!(format_count(0), "0");
    }

    #[test]
    fn test_format_count_small() {
        assert_eq!(format_count(42), "42");
    }

    #[test]
    fn test_format_count_thousands() {
        assert_eq!(format_count(1234), "1,234");
    }

    #[test]
    fn test_format_count_millions() {
        assert_eq!(format_count(1234567), "1,234,567");
    }

    #[test]
    fn test_sparkline_empty() {
        assert_eq!(render_sparkline(&[], 10), "");
    }

    #[test]
    fn test_sparkline_zero_width() {
        assert_eq!(render_sparkline(&[1, 2, 3], 0), "");
    }

    #[test]
    fn test_sparkline_all_zeros() {
        let result = render_sparkline(&[0, 0, 0], 3);
        assert_eq!(result.chars().count(), 3);
        assert!(result.chars().all(|c| c == ' '));
    }

    #[test]
    fn test_sparkline_uniform() {
        let result = render_sparkline(&[5, 5, 5], 3);
        // All equal → all max level
        assert_eq!(result.chars().count(), 3);
        assert!(result.chars().all(|c| c == '█'));
    }

    #[test]
    fn test_sparkline_ascending() {
        let result = render_sparkline(&[0, 50, 100], 3);
        let chars: Vec<char> = result.chars().collect();
        assert_eq!(chars.len(), 3);
        assert_eq!(chars[0], ' '); // 0
        assert_eq!(chars[2], '█'); // max
    }

    #[test]
    fn test_resample_smaller_target() {
        let result = resample(&[10, 20, 30, 40], 2);
        assert_eq!(result.len(), 2);
        // First half: avg(10,20)=15, second half: avg(30,40)=35
        assert_eq!(result[0], 15);
        assert_eq!(result[1], 35);
    }

    #[test]
    fn test_resample_larger_target() {
        let result = resample(&[5, 10], 4);
        assert_eq!(result.len(), 4);
        // Pads with zeros
        assert_eq!(result[0], 5);
        assert_eq!(result[1], 10);
        assert_eq!(result[2], 0);
        assert_eq!(result[3], 0);
    }

    #[test]
    fn test_build_entries_no_processor() {
        let app = crate::app::App::load_stdin(Vec::new()).unwrap();
        let entries = CategoryPanelWidget::build_entries(&app);
        assert!(entries.is_empty());
    }
}
