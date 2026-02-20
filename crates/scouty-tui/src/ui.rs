//! UI rendering for the TUI.

use crate::app::{App, InputMode};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Wrap},
};
use scouty::record::LogLevel;

/// Style a log line based on its level.
fn level_style(level: Option<LogLevel>) -> Style {
    match level {
        Some(LogLevel::Fatal) => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        Some(LogLevel::Error) => Style::default().fg(Color::Red),
        Some(LogLevel::Warn) => Style::default().fg(Color::Yellow),
        Some(LogLevel::Info) => Style::default().fg(Color::Green),
        Some(LogLevel::Debug) => Style::default().fg(Color::Gray),
        Some(LogLevel::Trace) => Style::default().fg(Color::DarkGray),
        None => Style::default(),
    }
}

/// Render the full UI.
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let footer_height = if matches!(
        app.input_mode,
        InputMode::Filter
            | InputMode::Search
            | InputMode::TimeJump
            | InputMode::GotoLine
            | InputMode::QuickExclude
            | InputMode::QuickInclude
    ) {
        2
    } else {
        1
    };

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),                // body (table + optional detail)
            Constraint::Length(footer_height), // footer
        ])
        .split(area);

    // Body: log table + optional detail panel
    if app.detail_open {
        let body_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(main_chunks[0]);
        render_log_table(frame, app, body_chunks[0]);
        render_detail_panel(frame, app, body_chunks[1]);
    } else {
        render_log_table(frame, app, main_chunks[0]);
    }

    render_footer(frame, app, main_chunks[1]);

    // Help overlay
    if app.input_mode == InputMode::Help {
        render_help_overlay(frame, area);
    }

    // Field filter overlay
    if app.input_mode == InputMode::FieldFilter {
        render_field_filter_overlay(frame, app, area);
    }

    // Filter manager overlay
    if app.input_mode == InputMode::FilterManager {
        render_filter_manager_overlay(frame, app, area);
    }

    // Column selector overlay
    if app.input_mode == InputMode::ColumnSelector {
        render_column_selector_overlay(frame, app, area);
    }

    if app.input_mode == InputMode::CopyFormat {
        render_copy_format_overlay(frame, area);
    }
}

fn render_log_table(frame: &mut Frame, app: &App, area: Rect) {
    use crate::app::Column;

    let visible = app.visible_records();
    let cw = &app.col_widths;
    let vis_cols = app.column_config.visible_columns();

    // Build column constraints based on visible columns
    let widths: Vec<Constraint> = vis_cols
        .iter()
        .map(|col| match col {
            Column::Time => Constraint::Length(cw[0]),
            Column::Level => Constraint::Length(cw[1]),
            Column::ProcessName => Constraint::Length(cw[2]),
            Column::Pid => Constraint::Length(cw[3]),
            Column::Tid => Constraint::Length(cw[4]),
            Column::Component => Constraint::Length(cw[5]),
            Column::Source => Constraint::Length(15),
            Column::Log => Constraint::Fill(1),
        })
        .collect();

    let header_cells: Vec<Cell> = vis_cols
        .iter()
        .map(|col| Cell::from(col.label()).style(Style::default().add_modifier(Modifier::BOLD)))
        .collect();

    let header =
        Row::new(header_cells).style(Style::default().bg(Color::DarkGray).fg(Color::White));

    let rows: Vec<Row> = visible
        .iter()
        .enumerate()
        .map(|(i, record)| {
            let filtered_idx = app.scroll_offset + i;
            let is_selected = filtered_idx == app.selected;
            let is_match = app.is_search_match(filtered_idx);

            let row_style = level_style(record.level);

            let cells: Vec<Cell> = vis_cols
                .iter()
                .map(|col| match col {
                    Column::Time => {
                        Cell::from(record.timestamp.format("%Y-%m-%d %H:%M:%S").to_string())
                    }
                    Column::Level => {
                        Cell::from(record.level.map(|l| format!("{}", l)).unwrap_or_default())
                    }
                    Column::ProcessName => {
                        Cell::from(record.process_name.as_deref().unwrap_or("").to_string())
                    }
                    Column::Pid => {
                        Cell::from(record.pid.map(|p| p.to_string()).unwrap_or_default())
                    }
                    Column::Tid => {
                        Cell::from(record.tid.map(|t| t.to_string()).unwrap_or_default())
                    }
                    Column::Component => {
                        Cell::from(record.component_name.as_deref().unwrap_or("").to_string())
                    }
                    Column::Source => Cell::from(record.source.to_string()),
                    Column::Log => Cell::from(record.message.clone()),
                })
                .collect();

            let mut row = Row::new(cells).style(row_style);
            if is_selected && is_match {
                row = row.style(row_style.bg(Color::Rgb(120, 120, 0)));
            } else if is_selected {
                row = row.style(row_style.bg(Color::Rgb(40, 40, 60)));
            } else if is_match {
                row = row.style(row_style.bg(Color::Rgb(80, 80, 0)));
            }
            row
        })
        .collect();

    let table = Table::new(rows, widths).header(header).column_spacing(1);

    frame.render_widget(table, area);
}

fn render_detail_panel(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Detail ")
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));

    if let Some(record) = app.selected_record() {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Timestamp: ", Style::default().fg(Color::Cyan)),
                Span::raw(record.timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string()),
            ]),
            Line::from(vec![
                Span::styled("Level:     ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    record
                        .level
                        .map(|l| l.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    level_style(record.level),
                ),
            ]),
            Line::from(vec![
                Span::styled("Source:    ", Style::default().fg(Color::Cyan)),
                Span::raw(record.source.as_ref()),
            ]),
        ];

        if let Some(ref comp) = record.component_name {
            lines.push(Line::from(vec![
                Span::styled("Component: ", Style::default().fg(Color::Cyan)),
                Span::raw(comp),
            ]));
        }
        if let Some(ref proc) = record.process_name {
            lines.push(Line::from(vec![
                Span::styled("Process:   ", Style::default().fg(Color::Cyan)),
                Span::raw(proc),
            ]));
        }
        if let Some(pid) = record.pid {
            lines.push(Line::from(vec![
                Span::styled("PID:       ", Style::default().fg(Color::Cyan)),
                Span::raw(pid.to_string()),
            ]));
        }
        if let Some(tid) = record.tid {
            lines.push(Line::from(vec![
                Span::styled("TID:       ", Style::default().fg(Color::Cyan)),
                Span::raw(tid.to_string()),
            ]));
        }

        if record.metadata.as_ref().is_some_and(|m| !m.is_empty()) {
            lines.push(Line::from(""));
            lines.push(Line::styled("Metadata:", Style::default().fg(Color::Cyan)));
            for (k, v) in record.metadata.as_ref().unwrap() {
                lines.push(Line::from(format!("  {} = {}", k, v)));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::styled("Message:", Style::default().fg(Color::Cyan)));
        lines.push(Line::from(record.message.clone()));

        let detail = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });
        frame.render_widget(detail, area);
    } else {
        let empty = Paragraph::new("No record selected").block(block);
        frame.render_widget(empty, area);
    }
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    match app.input_mode {
        InputMode::Filter => {
            render_input_footer(
                frame,
                area,
                "Filter: ",
                &app.filter_input,
                app.filter_error.as_deref(),
            );
        }
        InputMode::Search => {
            render_input_footer(frame, area, "/", &app.search_input, None);
        }
        InputMode::TimeJump => {
            render_input_footer(frame, area, "Jump to time: ", &app.time_input, None);
        }
        InputMode::GotoLine => {
            render_input_footer(frame, area, "Go to line: ", &app.goto_input, None);
        }
        InputMode::QuickExclude => {
            render_input_footer(frame, area, "Exclude text: ", &app.quick_filter_input, None);
        }
        InputMode::QuickInclude => {
            render_input_footer(frame, area, "Include text: ", &app.quick_filter_input, None);
        }
        _ => {
            // Status bar: density chart │ position info
            let position = if app.total() == 0 {
                format!("0/0 (Total: {})", app.total_records)
            } else {
                let current = app.selected + 1;
                let filtered = app.total();
                let total = app.total_records;
                if filtered == total {
                    format!("{}/{}", current, total)
                } else {
                    format!("{}/{} (Total: {})", current, filtered, total)
                }
            };

            // Right side: position + optional follow indicator + status message
            let follow_indicator = if app.follow_mode { " [FOLLOW]" } else { "" };
            let mut right_text = format!(" {}{} ", position, follow_indicator);
            if let Some(ref msg) = app.status_message {
                right_text = format!(" {} │{}", msg, right_text);
            }
            let right_width = right_text.len() as u16 + 1; // +1 for separator

            // Left side: density chart (adaptive width)
            let chart_width = area.width.saturating_sub(right_width + 3) as usize; // 3 for " │ "

            let mut spans: Vec<Span> = Vec::new();

            if chart_width >= 4 && app.total() > 0 {
                // Collect filtered timestamps
                let timestamps: Vec<chrono::DateTime<chrono::Utc>> = app
                    .filtered_indices
                    .iter()
                    .map(|&i| app.records[i].timestamp)
                    .collect();

                // 2 data points per braille char
                let num_buckets = (chart_width * 2).min(200);
                let buckets = crate::density::compute_density(&timestamps, num_buckets);

                let cursor_ts = app.selected_record().map(|r| r.timestamp);
                let cursor_bucket = cursor_ts
                    .and_then(|ts| crate::density::cursor_bucket(ts, &timestamps, num_buckets));

                let (braille_text, cursor_char_idx) =
                    crate::density::render_braille(&buckets, cursor_bucket);

                // Render braille with cursor highlight
                for (i, ch) in braille_text.chars().enumerate() {
                    let style = if Some(i) == cursor_char_idx {
                        Style::default()
                            .fg(Color::Yellow)
                            .bg(Color::Rgb(40, 40, 60))
                    } else {
                        Style::default().fg(Color::Cyan)
                    };
                    spans.push(Span::styled(ch.to_string(), style));
                }

                spans.push(Span::styled(" │", Style::default().fg(Color::DarkGray)));
            }

            // Right side
            if let Some(ref msg) = app.status_message {
                spans.push(Span::styled(
                    format!(" {} │", msg),
                    Style::default().fg(Color::Yellow),
                ));
            }
            spans.push(Span::styled(
                format!(" {}{} ", position, follow_indicator),
                Style::default().fg(Color::White).bg(Color::DarkGray),
            ));

            let footer = Paragraph::new(Line::from(spans));
            frame.render_widget(footer, area);
        }
    }
}

fn render_input_footer(
    frame: &mut Frame,
    area: Rect,
    prompt: &str,
    input: &str,
    error: Option<&str>,
) {
    let input_line = Paragraph::new(Line::from(vec![
        Span::styled(prompt, Style::default().fg(Color::Yellow)),
        Span::raw(input),
        Span::styled("█", Style::default().fg(Color::White)),
    ]));
    frame.render_widget(input_line, area);

    if let Some(err) = error {
        if area.height > 1 {
            let err_area = Rect::new(area.x, area.y + 1, area.width, 1);
            let err_line = Paragraph::new(Span::styled(err, Style::default().fg(Color::Red)));
            frame.render_widget(err_line, err_area);
        }
    }
}

fn render_help_overlay(frame: &mut Frame, area: Rect) {
    let width = 58u16.min(area.width.saturating_sub(4));
    let height = 36u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let overlay = Rect::new(x, y, width, height);

    frame.render_widget(Clear, overlay);

    let section = |title: &str| {
        Line::styled(
            format!(" {title}"),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    };

    let help_text = vec![
        Line::styled(
            " Keyboard Shortcuts ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Line::from(""),
        section("Navigation"),
        Line::from("  j / ↓            Move down one row"),
        Line::from("  k / ↑            Move up one row"),
        Line::from("  Ctrl+j/k / PgDn  Page down / up"),
        Line::from("  g / Home         Jump to first row"),
        Line::from("  G / End          Jump to last row"),
        Line::from("  Ctrl+G           Go to line number"),
        Line::from("  t                Jump to timestamp"),
        Line::from("  Ctrl+]           Toggle follow mode"),
        Line::from(""),
        section("Search & Filter"),
        Line::from("  /                Search (regex)"),
        Line::from("  n / N            Next / prev search match"),
        Line::from("  f                Filter expression"),
        Line::from("  - / +            Quick exclude / include"),
        Line::from("  Ctrl+F           Filter manager"),
        Line::from(""),
        section("Display"),
        Line::from("  Enter            Toggle detail panel"),
        Line::from("  Ctrl+C           Column selector"),
        Line::from(""),
        section("General"),
        Line::from("  ?                Show this help"),
        Line::from("  Esc              Close dialog / panel"),
        Line::from("  q                Quit"),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().bg(Color::Black));
    frame.render_widget(help, overlay);
}

fn render_field_filter_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let ff = match &app.field_filter {
        Some(ff) => ff,
        None => return,
    };

    let width = 60u16.min(area.width.saturating_sub(4));
    let height = (ff.fields.len() as u16 + 8).min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let overlay = Rect::new(x, y, width, height);

    frame.render_widget(Clear, overlay);

    let action = if ff.exclude { "Exclude" } else { "Include" };
    let logic = if ff.logic_or { "OR" } else { "AND" };
    let mut lines = vec![
        Line::from(vec![
            Span::raw(" Action: "),
            Span::styled(
                format!("[{}]", action),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" (Tab)", Style::default().fg(Color::DarkGray)),
            Span::raw("  Logic: "),
            Span::styled(
                format!("[{}]", logic),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" (o)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
    ];

    // Calculate visible window for scrolling
    let max_visible = (height as usize).saturating_sub(6); // header + footer lines
    let scroll_offset = if ff.cursor >= max_visible {
        ff.cursor - max_visible + 1
    } else {
        0
    };
    let visible_end = (scroll_offset + max_visible).min(ff.fields.len());

    for i in scroll_offset..visible_end {
        let (name, val, checked) = &ff.fields[i];
        let checkbox = if *checked { "[x]" } else { "[ ]" };
        let is_cursor = i == ff.cursor;
        let style = if is_cursor {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default()
        };
        // Truncate value to fit
        let max_val = (width as usize).saturating_sub(22);
        let display_val = if val.len() > max_val {
            format!("{}…", &val[..max_val.saturating_sub(1)])
        } else {
            val.clone()
        };
        lines.push(Line::styled(
            format!(" {} {:<14} = {}", checkbox, name, display_val),
            style,
        ));
    }

    if ff.fields.len() > max_visible {
        lines.push(Line::styled(
            format!(" ({}/{})", ff.cursor + 1, ff.fields.len()),
            Style::default().fg(Color::DarkGray),
        ));
    }

    lines.push(Line::from(""));
    lines.push(Line::styled(
        " Enter: Apply  Esc: Cancel  Space: Toggle",
        Style::default().fg(Color::DarkGray),
    ));

    let title = if ff.exclude {
        " Exclude Fields (Ctrl+-) "
    } else {
        " Include Fields (Ctrl++) "
    };

    let dialog = Paragraph::new(lines)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().bg(Color::Black));
    frame.render_widget(dialog, overlay);
}

fn render_filter_manager_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let width = 55u16.min(area.width.saturating_sub(4));
    let height = (app.filters.len() as u16 + 7)
        .min(area.height.saturating_sub(4))
        .max(8);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let overlay = Rect::new(x, y, width, height);

    frame.render_widget(Clear, overlay);

    let mut lines = vec![
        Line::styled(
            format!(" Active Filters ({})", app.filters.len()),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Line::from(""),
    ];

    if app.filters.is_empty() {
        lines.push(Line::styled(
            " (no active filters)",
            Style::default().fg(Color::DarkGray),
        ));
    } else {
        for (i, filter) in app.filters.iter().enumerate() {
            let is_cursor = i == app.filter_manager_cursor;
            let prefix = if filter.exclude { "−" } else { "+" };
            let style = if is_cursor {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };
            lines.push(Line::styled(format!(" {} {}", prefix, filter.label), style));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::styled(
        " d: Delete  c: Clear all  Esc: Close",
        Style::default().fg(Color::DarkGray),
    ));

    let dialog = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Filter Manager (F) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().bg(Color::Black));
    frame.render_widget(dialog, overlay);
}

fn render_column_selector_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let cols = &app.column_config.columns;
    let width = 35u16.min(area.width.saturating_sub(4));
    let height = (cols.len() as u16 + 5).min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let overlay = Rect::new(x, y, width, height);

    frame.render_widget(Clear, overlay);

    let mut lines = vec![
        Line::styled(
            " Toggle columns (Space/Enter)",
            Style::default().fg(Color::DarkGray),
        ),
        Line::from(""),
    ];

    for (i, (col, visible)) in cols.iter().enumerate() {
        let checkbox = if *visible { "[x]" } else { "[ ]" };
        let is_cursor = i == app.column_config.cursor;
        let suffix = if *col == crate::app::Column::Log {
            " (always on)"
        } else {
            ""
        };
        let style = if is_cursor {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default()
        };
        lines.push(Line::styled(
            format!(" {} {:<12}{}", checkbox, col.label(), suffix),
            style,
        ));
    }

    lines.push(Line::from(""));
    lines.push(Line::styled(
        " Esc: Close",
        Style::default().fg(Color::DarkGray),
    ));

    let dialog = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Columns (c) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().bg(Color::Black));
    frame.render_widget(dialog, overlay);
}

fn render_copy_format_overlay(frame: &mut Frame, area: Rect) {
    let width = 40u16.min(area.width.saturating_sub(4));
    let height = 9u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let overlay = Rect::new(x, y, width, height);

    frame.render_widget(Clear, overlay);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " [r] ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Raw text"),
        ]),
        Line::from(vec![
            Span::styled(
                " [j] ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("JSON"),
        ]),
        Line::from(vec![
            Span::styled(
                " [y] ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("YAML"),
        ]),
        Line::from(""),
        Line::styled(" Esc: Cancel", Style::default().fg(Color::DarkGray)),
    ];

    let dialog = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Copy As (Ctrl+Shift+C) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().bg(Color::Black));
    frame.render_widget(dialog, overlay);
}
