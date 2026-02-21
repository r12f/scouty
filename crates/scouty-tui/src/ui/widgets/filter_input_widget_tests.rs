#[cfg(test)]
mod tests {
    use crate::ui::widgets::filter_input_widget::FilterInputWidget;
    use crate::ui::{ComponentResult, UiComponent};

    #[test]
    fn test_enable_jk_navigation_false() {
        let widget = FilterInputWidget;
        assert!(!widget.enable_jk_navigation());
    }

    #[test]
    fn test_on_char_consumed() {
        let mut widget = FilterInputWidget;
        assert_eq!(widget.on_char('x'), ComponentResult::Consumed);
    }

    #[test]
    fn test_on_cancel_closes() {
        let mut widget = FilterInputWidget;
        assert_eq!(widget.on_cancel(), ComponentResult::Close);
    }

    #[test]
    fn test_on_confirm_closes() {
        let mut widget = FilterInputWidget;
        assert_eq!(widget.on_confirm(), ComponentResult::Close);
    }
}
