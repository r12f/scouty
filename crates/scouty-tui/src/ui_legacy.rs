//! UI rendering for the TUI.

use crate::app::{App, InputMode};
use ratatui::{prelude::*, widgets::Paragraph};

/// Layout heights for the main UI areas.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutHeights {
    pub log_table: u16,
    pub tab_bar: u16,
    pub panel_content: u16,
    pub footer: u16,
}

/// Compute the panel content height for the current state.
/// Extracted for testability.
pub fn compute_panel_content_height(app: &App, area_height: u16) -> LayoutHeights {
    let footer_height: u16 = 2;
    let tab_bar_height: u16 = 1;

    let panel_expanded = app.panel_state.expanded;
    let panel_maximized = app.panel_state.maximized;

    let panel_content_height = if panel_expanded {
        use crate::panel::PanelHeight;
        let body_height = area_height.saturating_sub(footer_height + tab_bar_height);
        match app.panel_state.active.default_height() {
            PanelHeight::FitContent => {
                if let Some(record) = app.selected_record() {
                    use crate::ui::widgets::detail_panel_widget::field_count;
                    let fc = field_count(record);
                    let left_min: u16 = if record.expanded.is_some() || !record.raw.is_empty() {
                        8
                    } else {
                        4
                    };
                    let raw_height = (fc.min(u16::MAX as usize) as u16)
                        .saturating_add(1)
                        .max(left_min);
                    let max_detail = (body_height as f64 * app.detail_panel_ratio) as u16;
                    if panel_maximized {
                        body_height
                    } else {
                        raw_height.min(max_detail).max(left_min)
                    }
                } else if panel_maximized {
                    body_height
                } else {
                    4
                }
            }
            PanelHeight::Percentage(pct) => {
                if panel_maximized {
                    body_height
                } else {
                    (body_height as u32 * pct as u32 / 100) as u16
                }
            }
        }
    } else {
        0
    };

    let log_table = if panel_maximized && panel_expanded {
        0
    } else {
        area_height.saturating_sub(footer_height + tab_bar_height + panel_content_height)
    };

    LayoutHeights {
        log_table,
        tab_bar: tab_bar_height,
        panel_content: panel_content_height,
        footer: footer_height,
    }
}

/// Render the full UI.
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let layout = compute_panel_content_height(app, area.height);
    let panel_maximized = app.panel_state.maximized;
    let panel_content_height = layout.panel_content;

    // Layout: [log table] [tab bar] [panel content?] [footer]
    let main_chunks = if panel_content_height > 0 {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(if panel_maximized { 0 } else { 3 }),
                Constraint::Length(layout.tab_bar),
                Constraint::Length(panel_content_height),
                Constraint::Length(layout.footer),
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Length(layout.tab_bar),
                Constraint::Length(0),
                Constraint::Length(layout.footer),
            ])
            .split(area)
    };

    let table_area = main_chunks[0];
    let tab_bar_area = main_chunks[1];
    let panel_area = main_chunks[2];
    let footer_area = main_chunks[3];

    // Render log table (skip if maximized)
    if !panel_maximized {
        render_log_table(frame, app, table_area);
    }

    // Compute visible_rows
    app.visible_rows = if panel_maximized {
        0
    } else {
        table_area.height.saturating_sub(1).max(1) as usize
    };

    // Render tab bar
    render_panel_tab_bar(frame, app, tab_bar_area);

    // Render panel content
    if panel_content_height > 0 {
        use crate::panel::PanelId;
        match app.panel_state.active {
            PanelId::Detail => {
                // Sync legacy state
                app.detail_open = true;
                render_detail_panel(frame, app, panel_area);
            }
            PanelId::Region => {
                render_region_panel(frame, app, panel_area);
            }
        }
    } else {
        app.detail_open = false;
    }

    render_footer(frame, app, footer_area);

    // Help overlay
    if app.input_mode == InputMode::Help {
        use crate::ui::windows::help_window::HelpWindow;
        use crate::ui::UiComponent;
        let mut window = HelpWindow::new(&app.theme);
        window.scroll = app.help_scroll;
        window.render(frame, area);
    }

    // Statistics overlay
    if app.input_mode == InputMode::Statistics {
        use crate::ui::windows::stats_window::StatsWindow;
        use crate::ui::UiComponent;
        if let Some(ref stats) = app.cached_stats {
            let window = StatsWindow {
                stats,
                theme: &app.theme,
            };
            window.render(frame, area);
        }
    }

    // Field filter overlay
    if app.input_mode == InputMode::FieldFilter {
        use crate::ui::windows::field_filter_window::FieldFilterWindow;
        use crate::ui::UiComponent;
        if let Some(window) = FieldFilterWindow::from_app(app) {
            window.render(frame, area);
        }
    }

    // Filter manager overlay
    if app.input_mode == InputMode::FilterManager {
        use crate::ui::windows::filter_manager_window::FilterManagerWindow;
        let window = FilterManagerWindow::from_app(app);
        window.render_with_app(frame, app, area);
    }

    // Column selector overlay
    if app.input_mode == InputMode::ColumnSelector {
        use crate::ui::windows::column_selector_window::ColumnSelectorWindow;
        use crate::ui::UiComponent;
        let window = ColumnSelectorWindow::from_app(app);
        window.render(frame, area);
    }

    if app.input_mode == InputMode::CopyFormat {
        use crate::ui::windows::copy_format_window::CopyFormatWindow;
        use crate::ui::UiComponent;
        let window = CopyFormatWindow::from_app(app);
        window.render(frame, area);
    }

    if app.input_mode == InputMode::SaveDialog {
        use crate::ui::windows::save_dialog_window::SaveDialogWindow;
        use crate::ui::UiComponent;
        let window = SaveDialogWindow::from_app(app);
        window.render(frame, area);
    }

    // Highlight manager overlay
    if app.input_mode == InputMode::HighlightManager {
        use crate::ui::windows::highlight_manager_window::HighlightManagerWindow;
        let window = HighlightManagerWindow::from_app(app);
        window.render_with_app(frame, app, area);
    }
    if app.input_mode == InputMode::BookmarkManager {
        use crate::ui::windows::bookmark_manager_window::BookmarkManagerWindow;
        let window = BookmarkManagerWindow::from_app(app);
        window.render_with_app(frame, app, area);
    }

    if app.input_mode == InputMode::LevelFilter {
        use crate::ui::windows::level_filter_window::LevelFilterWindow;
        use crate::ui::UiComponent;
        let window = LevelFilterWindow::from_app(app);
        UiComponent::render(&window, frame, area);
    }

    if app.input_mode == InputMode::SavePreset {
        use crate::ui::windows::save_preset_window::SavePresetWindow;
        use crate::ui::UiComponent;
        let mut window = SavePresetWindow::new();
        window.input = app.preset_name_input.clone();
        UiComponent::render(&window, frame, area);
    }

    if app.input_mode == InputMode::LoadPreset {
        use crate::ui::windows::load_preset_window::LoadPresetWindow;
        use crate::ui::UiComponent;
        let window = LoadPresetWindow::new(app.preset_list.clone());
        UiComponent::render(&window, frame, area);
    }

    if app.input_mode == InputMode::DensitySelector {
        use crate::ui::windows::density_selector_window::DensitySelectorWindow;
        use crate::ui::UiComponent;
        let options = app.density_source_options();
        let window = DensitySelectorWindow::new(options, app.density_selector_cursor);
        UiComponent::render(&window, frame, area);
    }

    if app.input_mode == InputMode::RegionManager {
        use crate::ui::windows::region_manager_window::RegionManagerWindow;
        let window = RegionManagerWindow::from_app(app);
        window.render_with_app(frame, area, app);
    }
}

fn render_panel_tab_bar(frame: &mut Frame, app: &App, area: Rect) {
    use crate::panel::PanelId;

    let indicator = if app.panel_state.expanded {
        "▾"
    } else {
        "▸"
    };

    let mut spans: Vec<Span> = vec![Span::raw(format!(" {} ", indicator))];

    for panel_id in PanelId::all() {
        let is_active = *panel_id == app.panel_state.active;
        let name = panel_id.name();

        if is_active {
            spans.push(Span::styled(
                format!(" {} ", name),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                format!(" {} ", name),
                Style::default().fg(Color::DarkGray),
            ));
        }
        spans.push(Span::raw(" │"));
    }

    let line = Paragraph::new(Line::from(spans)).style(
        Style::default().bg(app
            .theme
            .status_bar
            .line1_bg
            .to_style()
            .bg
            .unwrap_or(Color::Reset)),
    );
    frame.render_widget(line, area);
}

fn render_region_panel(frame: &mut Frame, app: &App, area: Rect) {
    use crate::ui::widgets::region_panel_widget::RegionPanelWidget;
    let widget = RegionPanelWidget;
    widget.render_with_app(frame, area, app);
}

fn render_log_table(frame: &mut Frame, app: &App, area: Rect) {
    use crate::ui::widgets::log_table_widget::LogTableWidget;
    let widget = LogTableWidget;
    widget.render_with_app(frame, area, app);
}

fn render_detail_panel(frame: &mut Frame, app: &App, area: Rect) {
    use crate::ui::widgets::detail_panel_widget::DetailPanelWidget;
    let widget = DetailPanelWidget;
    widget.render_with_app(frame, area, app);
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    // Line 1: density chart + position info (always shown)
    let line1_area = Rect::new(area.x, area.y, area.width, 1);
    {
        use crate::ui::widgets::status_bar_widget::StatusBarWidget;
        let widget = StatusBarWidget;
        widget.render_line1(frame, line1_area, app);
    }

    // Line 2: mode/shortcuts or input
    if area.height < 2 {
        return;
    }
    let line2_area = Rect::new(area.x, area.y + 1, area.width, 1);
    match app.input_mode {
        InputMode::Filter => {
            render_input_line2(
                frame,
                line2_area,
                "[FILTER]",
                &app.filter_input,
                app.filter_error.as_deref(),
                app,
            );
        }
        InputMode::Search => {
            render_input_line2(frame, line2_area, "[SEARCH]", &app.search_input, None, app);
        }
        InputMode::GotoLine => {
            render_input_line2(frame, line2_area, "[GOTO]", &app.goto_input, None, app);
        }
        InputMode::QuickExclude => {
            render_input_line2(
                frame,
                line2_area,
                "[EXCLUDE]",
                &app.quick_filter_input,
                None,
                app,
            );
        }
        InputMode::QuickInclude => {
            render_input_line2(
                frame,
                line2_area,
                "[INCLUDE]",
                &app.quick_filter_input,
                None,
                app,
            );
        }
        InputMode::Highlight => {
            render_input_line2(
                frame,
                line2_area,
                "[HIGHLIGHT]",
                &app.highlight_input,
                None,
                app,
            );
        }
        _ => {
            use crate::ui::widgets::status_bar_widget::StatusBarWidget;
            let widget = StatusBarWidget;
            widget.render_line2(frame, line2_area, app);
        }
    }
}

fn render_input_line2(
    frame: &mut Frame,
    area: Rect,
    mode: &str,
    text_input: &crate::text_input::TextInput,
    error: Option<&str>,
    app: &App,
) {
    let theme = &app.theme;
    let (before, cursor_ch, after) = text_input.render_parts();
    let mut spans = vec![
        Span::styled(
            format!(" {} ", mode),
            theme.status_bar.search_mode_label.to_style(),
        ),
        Span::raw(" "),
        Span::raw(before),
        Span::styled(cursor_ch, theme.input.cursor.to_style()),
        Span::raw(after),
    ];

    if let Some(err) = error {
        spans.push(Span::styled(
            format!("  {}", err),
            theme.input.error.to_style(),
        ));
    }

    let input_line = Paragraph::new(Line::from(spans)).style(theme.input.background.to_style());
    frame.render_widget(input_line, area);
}

#[cfg(test)]
mod ui_legacy_tests {
    use super::*;
    use crate::app::{ColumnConfig, DensitySource, InputMode};
    use crate::config::Theme;
    use crate::panel::PanelId;
    use crate::text_input::TextInput;

    fn make_test_app() -> App {
        App {
            records: vec![],
            total_records: 0,
            filtered_indices: vec![],
            scroll_offset: 0,
            selected: 0,
            visible_rows: 10,
            detail_open: false,
            detail_panel_ratio: 0.3,
            detail_tree_cursor: 0,
            detail_tree_collapsed: std::collections::HashSet::new(),
            detail_tree_focus: false,
            panel_state: crate::panel::PanelState::default(),
            input_mode: InputMode::Normal,
            filter_input: TextInput::new(),
            filter_error: None,
            filters: Vec::new(),
            quick_filter_input: TextInput::new(),
            field_filter: None,
            filter_manager_cursor: 0,
            search_input: TextInput::new(),
            search_matches: vec![],
            search_match_idx: None,
            time_input: TextInput::new(),
            goto_input: TextInput::new(),
            status_message: None,
            status_message_at: None,
            col_widths: [19, 5, 11, 3, 3, 9],
            column_config: ColumnConfig::default(),
            follow_mode: false,
            should_quit: false,
            copy_format_cursor: 0,
            save_path_input: TextInput::with_text("./scouty-export.log"),
            save_format_cursor: 0,
            save_dialog_focus: crate::ui::windows::save_dialog_window::Focus::Path,
            help_scroll: 0,
            command_input: TextInput::new(),
            filter_version: 0,
            density_cache: None,
            highlight_rules: Vec::new(),
            highlight_input: TextInput::new(),
            highlight_manager_cursor: 0,
            cached_stats: None,
            bookmarks: std::collections::HashSet::new(),
            bookmark_manager_cursor: 0,
            theme: Theme::default(),
            level_filter: None,
            level_filter_cursor: 0,
            preset_name_input: TextInput::new(),
            preset_list: Vec::new(),
            preset_list_cursor: 0,
            density_source: DensitySource::All,
            density_selector_cursor: 0,
            regions: scouty::region::store::RegionStore::default(),
            region_manager_cursor: 0,
            region_panel_sort: crate::ui::widgets::region_panel_widget::RegionSortMode::StartTime,
            region_panel_type_filter: None,
        }
    }

    #[test]
    fn test_layout_collapsed_panel() {
        let app = make_test_app();
        assert!(!app.panel_state.expanded);
        let layout = compute_panel_content_height(&app, 40);
        assert_eq!(layout.panel_content, 0);
        assert_eq!(layout.footer, 2);
        assert_eq!(layout.tab_bar, 1);
    }

    #[test]
    fn test_layout_expanded_detail_no_record() {
        let mut app = make_test_app();
        app.panel_state.open(PanelId::Detail);
        let layout = compute_panel_content_height(&app, 40);
        assert_eq!(layout.panel_content, 4);
        assert_eq!(layout.log_table, 40 - 2 - 1 - 4);
    }

    #[test]
    fn test_layout_maximized_detail_no_record() {
        let mut app = make_test_app();
        app.panel_state.open(PanelId::Detail);
        app.panel_state.toggle_maximize();
        assert!(app.panel_state.maximized);
        let layout = compute_panel_content_height(&app, 40);
        let body_height: u16 = 40 - 2 - 1;
        assert_eq!(layout.panel_content, body_height);
        assert_eq!(layout.log_table, 0);
    }

    #[test]
    fn test_layout_maximized_region() {
        let mut app = make_test_app();
        app.panel_state.open(PanelId::Region);
        app.panel_state.toggle_maximize();
        assert!(app.panel_state.maximized);
        let layout = compute_panel_content_height(&app, 50);
        let body_height: u16 = 50 - 2 - 1;
        assert_eq!(layout.panel_content, body_height);
        assert_eq!(layout.log_table, 0);
    }

    #[test]
    fn test_layout_restore_after_maximize() {
        let mut app = make_test_app();
        app.panel_state.open(PanelId::Region);
        let normal_layout = compute_panel_content_height(&app, 50);
        assert!(normal_layout.panel_content > 0);
        assert!(normal_layout.log_table > 0);

        app.panel_state.toggle_maximize();
        let max_layout = compute_panel_content_height(&app, 50);
        assert_eq!(max_layout.log_table, 0);
        assert_eq!(max_layout.panel_content, 50 - 3);

        app.panel_state.toggle_maximize();
        let restored_layout = compute_panel_content_height(&app, 50);
        assert_eq!(restored_layout, normal_layout);
    }
}
