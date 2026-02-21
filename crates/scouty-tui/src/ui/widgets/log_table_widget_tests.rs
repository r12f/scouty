#[cfg(test)]
mod tests {
    use crate::ui::widgets::log_table_widget::LogTableWidget;
    use crate::ui::{dispatch_key, ComponentResult, UiComponent};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

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

    #[test]
    fn test_dispatch_j_calls_on_down() {
        let mut widget = LogTableWidget;
        assert_eq!(
            dispatch_key(&mut widget, key(KeyCode::Char('j'))),
            ComponentResult::Consumed
        );
    }

    #[test]
    fn test_dispatch_k_calls_on_up() {
        let mut widget = LogTableWidget;
        assert_eq!(
            dispatch_key(&mut widget, key(KeyCode::Char('k'))),
            ComponentResult::Consumed
        );
    }

    #[test]
    fn test_dispatch_arrow_keys() {
        let mut widget = LogTableWidget;
        assert_eq!(
            dispatch_key(&mut widget, key(KeyCode::Up)),
            ComponentResult::Consumed
        );
        assert_eq!(
            dispatch_key(&mut widget, key(KeyCode::Down)),
            ComponentResult::Consumed
        );
    }
}
