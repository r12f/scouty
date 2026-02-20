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
        InputMode::Filter | InputMode::Search | InputMode::TimeJump | InputMode::GotoLine
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
}

fn render_log_table(frame: &mut Frame, app: &App, area: Rect) {
    let visible = app.visible_records();
    let cw = &app.col_widths;

    // Build column constraints: fixed widths for first 6, Fill for Log
    let widths = [
        Constraint::Length(cw[0]),
        Constraint::Length(cw[1]),
        Constraint::Length(cw[2]),
        Constraint::Length(cw[3]),
        Constraint::Length(cw[4]),
        Constraint::Length(cw[5]),
        Constraint::Fill(1), // Log column fills remaining
    ];

    let header = Row::new(vec![
        Cell::from("Time").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Level").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("ProcessName").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Pid").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Tid").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Component").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Log").style(Style::default().add_modifier(Modifier::BOLD)),
    ])
    .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    let rows: Vec<Row> = visible
        .iter()
        .enumerate()
        .map(|(i, record)| {
            let filtered_idx = app.scroll_offset + i;
            let is_selected = filtered_idx == app.selected;
            let is_match = app.is_search_match(filtered_idx);

            let row_style = level_style(record.level);

            let ts = record.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
            let level_str = record.level.map(|l| format!("{}", l)).unwrap_or_default();
            let proc_name = record.process_name.as_deref().unwrap_or("");
            let pid = record.pid.map(|p| p.to_string()).unwrap_or_default();
            let tid = record.tid.map(|t| t.to_string()).unwrap_or_default();
            let component = record.component_name.as_deref().unwrap_or("");

            let cells = vec![
                Cell::from(ts),
                Cell::from(level_str),
                Cell::from(proc_name.to_string()),
                Cell::from(pid),
                Cell::from(tid),
                Cell::from(component.to_string()),
                Cell::from(record.message.clone()),
            ];

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

            // Right side: position + optional status message
            let mut right_text = format!(" {} ", position);
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
                format!(" {} ", position),
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
    let width = 55u16.min(area.width.saturating_sub(4));
    let height = 22u16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let overlay = Rect::new(x, y, width, height);

    frame.render_widget(Clear, overlay);

    let help_text = vec![
        Line::styled(
            " Keyboard Shortcuts ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Line::from(""),
        Line::from(" Navigation"),
        Line::from("  j / k            Move up/down one row"),
        Line::from("  Ctrl+j/k/↑/↓    Page jump (half screen)"),
        Line::from("  g                Jump to first row"),
        Line::from("  G                Jump to last row"),
        Line::from("  Ctrl+G           Go to line number"),
        Line::from(""),
        Line::from(" Actions"),
        Line::from("  Enter            Toggle detail panel"),
        Line::from("  f                Filter expression"),
        Line::from("  /                Search (regex)"),
        Line::from("  n / N            Next / prev match"),
        Line::from("  t                Jump to time"),
        Line::from("  ?                Show this help"),
        Line::from(""),
        Line::from(" General"),
        Line::from("  Esc              Close dialog/panel"),
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
