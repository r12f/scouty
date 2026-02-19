//! UI rendering for the TUI.

use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
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

    // Layout: header (1) + log list (remaining) + footer (1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " scouty ",
            Style::default().fg(Color::White).bg(Color::Blue),
        ),
        Span::raw(format!(" {} records loaded", app.total)),
    ]));
    frame.render_widget(header, chunks[0]);

    // Log list
    let log_area = chunks[1];

    let visible = app.visible_records();
    let lines: Vec<Line> = visible
        .iter()
        .map(|record| {
            let text = format_record(record);
            Line::styled(text, level_style(record.level))
        })
        .collect();

    let log_block = Paragraph::new(lines).block(Block::default().borders(Borders::NONE));
    frame.render_widget(log_block, log_area);

    // Footer
    let position = if app.total == 0 {
        "Empty".to_string()
    } else {
        format!(
            "Lines {}-{} of {}",
            app.scroll_offset + 1,
            (app.scroll_offset + app.visible_rows).min(app.total),
            app.total
        )
    };

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            " q:quit j/k:scroll PgUp/PgDn g/G:top/bottom ",
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            format!(" {} ", position),
            Style::default().fg(Color::White).bg(Color::DarkGray),
        ),
    ]))
    .alignment(Alignment::Right);
    frame.render_widget(footer, chunks[2]);
}
