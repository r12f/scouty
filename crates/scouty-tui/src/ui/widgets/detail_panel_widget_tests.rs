#[cfg(test)]
mod tests {
    use crate::ui::widgets::detail_panel_widget::DetailPanelWidget;
    use crate::ui::{dispatch_key, ComponentResult, UiComponent};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn test_enable_jk_navigation() {
        let widget = DetailPanelWidget;
        assert!(!widget.enable_jk_navigation());
    }

    #[test]
    fn test_esc_closes() {
        let mut widget = DetailPanelWidget;
        assert_eq!(
            dispatch_key(&mut widget, key(KeyCode::Esc)),
            ComponentResult::Close
        );
    }

    #[test]
    fn test_navigation_ignored() {
        let mut widget = DetailPanelWidget;
        assert_eq!(
            dispatch_key(&mut widget, key(KeyCode::Up)),
            ComponentResult::Ignored
        );
        assert_eq!(
            dispatch_key(&mut widget, key(KeyCode::Down)),
            ComponentResult::Ignored
        );
    }
}
