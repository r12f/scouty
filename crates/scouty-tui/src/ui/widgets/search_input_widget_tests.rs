#[cfg(test)]
mod tests {
    use crate::ui::widgets::search_input_widget::SearchInputWidget;
    use crate::ui::{ComponentResult, UiComponent};

    #[test]
    fn test_enable_jk_navigation_false() {
        let widget = SearchInputWidget;
        assert!(!widget.enable_jk_navigation());
    }

    #[test]
    fn test_on_char_consumed() {
        let mut widget = SearchInputWidget;
        assert_eq!(widget.on_char('a'), ComponentResult::Consumed);
    }

    #[test]
    fn test_on_cancel_closes() {
        let mut widget = SearchInputWidget;
        assert_eq!(widget.on_cancel(), ComponentResult::Close);
    }

    #[test]
    fn test_on_confirm_closes() {
        let mut widget = SearchInputWidget;
        assert_eq!(widget.on_confirm(), ComponentResult::Close);
    }
}
