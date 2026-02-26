#[cfg(test)]
mod tests {
    use crate::panel::{Panel, PanelHeight};
    use crate::ui::widgets::region_panel_widget::{
        format_duration, RegionPanelWidget, RegionSortMode,
    };

    #[test]
    fn test_region_panel_trait_name() {
        let widget = RegionPanelWidget;
        assert_eq!(Panel::name(&widget), "Region");
    }

    #[test]
    fn test_region_panel_trait_shortcut() {
        let widget = RegionPanelWidget;
        assert_eq!(widget.shortcut(), Some('r'));
    }

    #[test]
    fn test_region_panel_trait_default_height() {
        let widget = RegionPanelWidget;
        assert_eq!(widget.default_height(), PanelHeight::Percentage(40));
    }

    #[test]
    fn test_region_panel_trait_is_available() {
        let widget = RegionPanelWidget;
        assert!(widget.is_available());
    }

    #[test]
    fn test_format_duration_ms() {
        assert_eq!(format_duration(0), "0ms");
        assert_eq!(format_duration(12), "12ms");
        assert_eq!(format_duration(999), "999ms");
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(1000), "1.0s");
        assert_eq!(format_duration(1500), "1.5s");
        assert_eq!(format_duration(59999), "60.0s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(60_000), "1m0s");
        assert_eq!(format_duration(90_000), "1m30s");
    }

    #[test]
    fn test_format_duration_negative() {
        assert_eq!(format_duration(-1), "0ms");
    }

    #[test]
    fn test_sort_mode_toggle() {
        assert_eq!(RegionSortMode::StartTime.toggle(), RegionSortMode::Duration);
        assert_eq!(RegionSortMode::Duration.toggle(), RegionSortMode::StartTime);
    }

    #[test]
    fn test_sort_mode_label() {
        assert_eq!(RegionSortMode::StartTime.label(), "start");
        assert_eq!(RegionSortMode::Duration.label(), "duration");
    }
}
