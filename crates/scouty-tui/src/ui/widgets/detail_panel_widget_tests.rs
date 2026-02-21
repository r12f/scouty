#[cfg(test)]
mod tests {
    use crate::ui::widgets::detail_panel_widget::DetailPanelWidget;
    use crate::ui::UiComponent;

    #[test]
    fn test_enable_jk_navigation() {
        let widget = DetailPanelWidget;
        assert!(!widget.enable_jk_navigation());
    }
}
