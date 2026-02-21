#[cfg(test)]
mod tests {
    use crate::ui::widgets::log_table_widget::LogTableWidget;
    use crate::ui::{ComponentResult, UiComponent};

    #[test]
    fn test_enable_jk_navigation_true() {
        let widget = LogTableWidget;
        assert!(widget.enable_jk_navigation());
    }

    #[test]
    fn test_on_up_consumed() {
        let mut widget = LogTableWidget;
        assert_eq!(widget.on_up(), ComponentResult::Consumed);
    }

    #[test]
    fn test_on_down_consumed() {
        let mut widget = LogTableWidget;
        assert_eq!(widget.on_down(), ComponentResult::Consumed);
    }

    #[test]
    fn test_on_page_up_consumed() {
        let mut widget = LogTableWidget;
        assert_eq!(widget.on_page_up(), ComponentResult::Consumed);
    }

    #[test]
    fn test_on_page_down_consumed() {
        let mut widget = LogTableWidget;
        assert_eq!(widget.on_page_down(), ComponentResult::Consumed);
    }
}
