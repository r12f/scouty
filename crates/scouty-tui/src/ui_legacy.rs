//! UI rendering for the TUI.

use crate::app::{App, InputMode};
use ratatui::{prelude::*, widgets::Paragraph};

/// Render the full UI.
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Footer is always 2 lines: line 1 = density/position, line 2 = mode/shortcuts or input
    let footer_height = 2;

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(footer_height)])
        .split(area);

    // Body: log table + optional detail panel
    if app.detail_open {
        let detail_height = if let Some(record) = app.selected_record() {
            use crate::ui::widgets::detail_panel_widget::field_count;
            let fc = field_count(record);
            // +1 for top border, min 4
            let raw_height = (fc.min(u16::MAX as usize) as u16).saturating_add(1).max(4);
            // Cap at half the available body height so the log table stays usable
            let max_detail = main_chunks[0].height / 2;
            raw_height.min(max_detail).max(4)
        } else {
            4 // "No record selected" + border
        };
        let body_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(detail_height)])
            .split(main_chunks[0]);
        render_log_table(frame, app, body_chunks[0]);
        render_detail_panel(frame, app, body_chunks[1]);
    } else {
        render_log_table(frame, app, main_chunks[0]);
    }

    render_footer(frame, app, main_chunks[1]);

    // Help overlay
    if app.input_mode == InputMode::Help {
        use crate::ui::windows::help_window::HelpWindow;
        use crate::ui::UiComponent;
        let window = HelpWindow;
        window.render(frame, area);
    }

    // Statistics overlay
    if app.input_mode == InputMode::Statistics {
        use crate::ui::windows::stats_window::StatsWindow;
        use crate::ui::UiComponent;
        let window = StatsWindow::new(app);
        window.render(frame, area);
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

    // Highlight manager overlay
    if app.input_mode == InputMode::HighlightManager {
        use crate::ui::windows::highlight_manager_window::HighlightManagerWindow;
        let window = HighlightManagerWindow::from_app(app);
        window.render_with_app(frame, app, area);
    }
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
            );
        }
        InputMode::Search => {
            render_input_line2(frame, line2_area, "[SEARCH]", &app.search_input, None);
        }
        InputMode::TimeJump => {
            render_input_line2(frame, line2_area, "[GOTO]", &app.time_input, None);
        }
        InputMode::GotoLine => {
            render_input_line2(frame, line2_area, "[GOTO]", &app.goto_input, None);
        }
        InputMode::QuickExclude => {
            render_input_line2(
                frame,
                line2_area,
                "[EXCLUDE]",
                &app.quick_filter_input,
                None,
            );
        }
        InputMode::QuickInclude => {
            render_input_line2(
                frame,
                line2_area,
                "[INCLUDE]",
                &app.quick_filter_input,
                None,
            );
        }
        InputMode::Highlight => {
            render_input_line2(frame, line2_area, "[HIGHLIGHT]", &app.highlight_input, None);
        }
        _ => {
            use crate::ui::widgets::status_bar_widget::StatusBarWidget;
            let widget = StatusBarWidget;
            widget.render_line2(frame, line2_area, app);
        }
    }
}

fn render_input_line2(frame: &mut Frame, area: Rect, mode: &str, input: &str, error: Option<&str>) {
    let mut spans = vec![
        Span::styled(
            format!(" {} ", mode),
            Style::default().fg(Color::Black).bg(Color::Yellow),
        ),
        Span::raw(" "),
        Span::raw(input),
        Span::styled("█", Style::default().fg(Color::White)),
    ];

    if let Some(err) = error {
        spans.push(Span::styled(
            format!("  {}", err),
            Style::default().fg(Color::Red),
        ));
    }

    let input_line = Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::DarkGray));
    frame.render_widget(input_line, area);
}
