//! UI rendering for the TUI.

use crate::app::{App, InputMode};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use scouty::record::LogLevel;

/// Style a log line based on its level.
fn level_style(level: Option<LogLevel>) -> Style {
    match level {
        Some(LogLevel::Fatal) => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        Some(LogLevel::Error) => Style::default().fg(Color::Red),
        Some(LogLevel::Warn) => Style::default().fg(Color::Yellow),
        Some(LogLevel::Info) => Style::default().fg(Color::Green),
        Some(LogLevel::Debug) => Style::default().fg(Color::Cyan),
        Some(LogLevel::Trace) => Style::default().fg(Color::DarkGray),
        None => Style::default(),
    }
}

/// Format a single log record as a line.
fn format_record(record: &scouty::record::LogRecord) -> String {
    let ts = record.timestamp.format("%Y-%m-%d %H:%M:%S");
    let level = record
        .level
        .map(|l| format!("{:5}", l))
        .unwrap_or_else(|| "     ".to_string());
    format!("{} {} {}", ts, level, record.message)
}

/// Render the full UI.
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Main layout: header + body + input/footer
    let footer_height = if matches!(
        app.input_mode,
        InputMode::Filter | InputMode::Search | InputMode::TimeJump
    ) {
        2
    } else {
        1
    };

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(footer_height),
        ])
        .split(area);

    render_header(frame, app, main_chunks[0]);

    // Body: log list + optional detail panel
    if app.detail_open {
        let body_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(main_chunks[1]);
        render_log_list(frame, app, body_chunks[0]);
        render_detail_panel(frame, app, body_chunks[1]);
    } else {
        render_log_list(frame, app, main_chunks[1]);
    }

    render_footer(frame, app, main_chunks[2]);

    // Help overlay
    if app.input_mode == InputMode::Help {
        render_help_overlay(frame, area);
    }
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans = vec![
        Span::styled(
            " scouty ",
            Style::default().fg(Color::White).bg(Color::Blue),
        ),
        Span::raw(format!(" {} records", app.total())),
    ];

    if app.records.len() != app.total() {
        spans.push(Span::styled(
            format!(" (filtered from {})", app.records.len()),
            Style::default().fg(Color::DarkGray),
        ));
    }

    if let Some(ref msg) = app.status_message {
        spans.push(Span::raw(" | "));
        spans.push(Span::styled(msg, Style::default().fg(Color::Yellow)));
    }

    let header = Paragraph::new(Line::from(spans));
    frame.render_widget(header, area);
}

fn render_log_list(frame: &mut Frame, app: &App, area: Rect) {
    let visible = app.visible_records();
    let lines: Vec<Line> = visible
        .iter()
        .enumerate()
        .map(|(i, record)| {
            let filtered_idx = app.scroll_offset + i;
            let is_selected = filtered_idx == app.selected;
            let is_match = app.is_search_match(filtered_idx);
            let text = format_record(record);

            let mut style = level_style(record.level);
            if is_selected {
                style = style.bg(Color::DarkGray);
            }
            if is_match {
                style = style.bg(Color::Rgb(60, 60, 0));
            }
            if is_selected && is_match {
                style = style.bg(Color::Rgb(80, 80, 0));
            }

            Line::styled(text, style)
        })
        .collect();

    let log_block = Paragraph::new(lines).block(Block::default().borders(Borders::NONE));
    frame.render_widget(log_block, area);
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
                Span::raw(&record.source),
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

        if !record.metadata.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::styled("Metadata:", Style::default().fg(Color::Cyan)));
            for (k, v) in &record.metadata {
                lines.push(Line::from(format!("  {} = {}", k, v)));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::styled("Message:", Style::default().fg(Color::Cyan)));
        lines.push(Line::from(record.message.clone()));

        if record.raw != record.message {
            lines.push(Line::from(""));
            lines.push(Line::styled("Raw:", Style::default().fg(Color::Cyan)));
            lines.push(Line::from(record.raw.clone()));
        }

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
            let input = Paragraph::new(Line::from(vec![
                Span::styled("Filter: ", Style::default().fg(Color::Yellow)),
                Span::raw(&app.filter_input),
                Span::styled("█", Style::default().fg(Color::White)),
            ]));
            frame.render_widget(input, area);
            if let Some(ref err) = app.filter_error {
                if area.height > 1 {
                    let err_area = Rect::new(area.x, area.y + 1, area.width, 1);
                    let err_line =
                        Paragraph::new(Span::styled(err.as_str(), Style::default().fg(Color::Red)));
                    frame.render_widget(err_line, err_area);
                }
            }
        }
        InputMode::Search => {
            let input = Paragraph::new(Line::from(vec![
                Span::styled("/", Style::default().fg(Color::Yellow)),
                Span::raw(&app.search_input),
                Span::styled("█", Style::default().fg(Color::White)),
            ]));
            frame.render_widget(input, area);
        }
        InputMode::TimeJump => {
            let input = Paragraph::new(Line::from(vec![
                Span::styled("Jump to time: ", Style::default().fg(Color::Yellow)),
                Span::raw(&app.time_input),
                Span::styled("█", Style::default().fg(Color::White)),
            ]));
            frame.render_widget(input, area);
        }
        _ => {
            let position = if app.total() == 0 {
                "Empty".to_string()
            } else {
                format!("Line {} of {}", app.selected + 1, app.total())
            };

            let footer = Paragraph::new(Line::from(vec![
                Span::styled(
                    " ?:help q:quit j/k:↑↓ Enter:detail /:search f:filter t:time ",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!(" {} ", position),
                    Style::default().fg(Color::White).bg(Color::DarkGray),
                ),
            ]))
            .alignment(Alignment::Right);
            frame.render_widget(footer, area);
        }
    }
}

fn render_help_overlay(frame: &mut Frame, area: Rect) {
    // Center overlay
    let width = 50u16.min(area.width.saturating_sub(4));
    let height = 20u16.min(area.height.saturating_sub(4));
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
        Line::from("  j / ↓        Scroll down"),
        Line::from("  k / ↑        Scroll up"),
        Line::from("  PgDn         Page down"),
        Line::from("  PgUp         Page up"),
        Line::from("  g / Home     Go to top"),
        Line::from("  G / End      Go to bottom"),
        Line::from(""),
        Line::from(" Actions"),
        Line::from("  Enter        Toggle detail panel"),
        Line::from("  f            Filter records"),
        Line::from("  /            Search text"),
        Line::from("  n / N        Next / prev search match"),
        Line::from("  t            Jump to time"),
        Line::from("  ? / h        Show this help"),
        Line::from("  q / Esc      Quit (or close panel)"),
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
