#[cfg(test)]
mod tests {
    use crate::ui::widgets::status_bar_widget::StatusBarWidget;
    use crate::ui::UiComponent;

    #[test]
    fn test_enable_jk_navigation() {
        let widget = StatusBarWidget;
        assert!(!widget.enable_jk_navigation());
    }
}
