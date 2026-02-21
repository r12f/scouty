#[cfg(test)]
mod tests {
    use crate::ui::widgets::search_input_widget::SearchInputWidget;
    use crate::ui::{dispatch_key, ComponentResult, UiComponent};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

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

    #[test]
    fn test_dispatch_j_goes_to_on_char_not_navigation() {
        // j/k should NOT be intercepted as navigation since enable_jk_navigation=false
        let mut widget = SearchInputWidget;
        assert_eq!(
            dispatch_key(&mut widget, key(KeyCode::Char('j'))),
            ComponentResult::Consumed
        );
    }

    #[test]
    fn test_dispatch_esc_closes() {
        let mut widget = SearchInputWidget;
        assert_eq!(
            dispatch_key(&mut widget, key(KeyCode::Esc)),
            ComponentResult::Close
        );
    }
}
