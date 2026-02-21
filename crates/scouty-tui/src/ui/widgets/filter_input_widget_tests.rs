#[cfg(test)]
mod tests {
    use crate::ui::widgets::filter_input_widget::FilterInputWidget;
    use crate::ui::{dispatch_key, ComponentResult, UiComponent};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn test_enable_jk_navigation_false() {
        let widget = FilterInputWidget;
        assert!(!widget.enable_jk_navigation());
    }

    #[test]
    fn test_on_char_consumed() {
        let mut widget = FilterInputWidget;
        assert_eq!(widget.on_char('a'), ComponentResult::Consumed);
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

    #[test]
    fn test_dispatch_j_goes_to_on_char_not_navigation() {
        let mut widget = FilterInputWidget;
        assert_eq!(
            dispatch_key(&mut widget, key(KeyCode::Char('j'))),
            ComponentResult::Consumed
        );
    }

    #[test]
    fn test_dispatch_enter_closes() {
        let mut widget = FilterInputWidget;
        assert_eq!(
            dispatch_key(&mut widget, key(KeyCode::Enter)),
            ComponentResult::Close
        );
    }
}
