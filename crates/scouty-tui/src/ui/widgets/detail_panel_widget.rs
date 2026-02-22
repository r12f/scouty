//! Detail panel widget — expanded view of the selected log record.

#[cfg(test)]
#[path = "detail_panel_widget_tests.rs"]
mod detail_panel_widget_tests;

use crate::app::App;
use crate::ui::UiComponent;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use scouty::record::LogLevel;

fn level_style(level: Option<LogLevel>) -> Style {
    match level {
        Some(LogLevel::Fatal) => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        Some(LogLevel::Error) => Style::default().fg(Color::Red),
        Some(LogLevel::Warn) => Style::default().fg(Color::Yellow),
        Some(LogLevel::Notice) => Style::default().fg(Color::Cyan),
        Some(LogLevel::Info) => Style::default().fg(Color::Green),
        Some(LogLevel::Debug) => Style::default().fg(Color::Gray),
        Some(LogLevel::Trace) => Style::default().fg(Color::DarkGray),
        None => Style::default(),
    }
}

/// Build field lines for the right pane.
fn build_field_lines(record: &scouty::record::LogRecord) -> Vec<Line<'static>> {
    let label_style = Style::default().fg(Color::Cyan);
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Timestamp: ", label_style),
            Span::raw(record.timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string()),
        ]),
        Line::from(vec![
            Span::styled("Level:     ", label_style),
            Span::styled(
                record
                    .level
                    .map(|l| l.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                level_style(record.level),
            ),
        ]),
        Line::from(vec![
            Span::styled("Source:    ", label_style),
            Span::raw(record.source.to_string()),
        ]),
    ];

    let optional_fields: Vec<(&str, Option<String>)> = vec![
        ("Hostname:  ", record.hostname.clone()),
        ("Container: ", record.container.clone()),
        ("Context:   ", record.context.clone()),
        ("Function:  ", record.function.clone()),
        ("Component: ", record.component_name.clone()),
        ("Process:   ", record.process_name.clone()),
        ("PID:       ", record.pid.map(|p| p.to_string())),
        ("TID:       ", record.tid.map(|t| t.to_string())),
    ];

    for (label, value) in optional_fields {
        if let Some(val) = value {
            lines.push(Line::from(vec![
                Span::styled(label, label_style),
                Span::raw(val),
            ]));
        }
    }

    if record.metadata.as_ref().is_some_and(|m| !m.is_empty()) {
        lines.push(Line::from(""));
        lines.push(Line::styled("Metadata:", label_style));
        for (k, v) in record.metadata.as_ref().unwrap() {
            lines.push(Line::from(format!("  {} = {}", k, v)));
        }
    }

    lines
}

pub struct DetailPanelWidget;

/// Minimum total width to show split layout.
const MIN_SPLIT_WIDTH: u16 = 80;

impl DetailPanelWidget {
    pub fn render_with_app(&self, frame: &mut Frame, area: Rect, app: &App) {
        let block = Block::default()
            .title(" Detail ")
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray));

        let Some(record) = app.selected_record() else {
            let empty = Paragraph::new("No record selected").block(block);
            frame.render_widget(empty, area);
            return;
        };

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.width < MIN_SPLIT_WIDTH {
            // Narrow: single-column fallback (fields then raw)
            self.render_single_column(frame, inner, record);
        } else {
            self.render_split(frame, inner, record);
        }
    }

    fn render_split(&self, frame: &mut Frame, area: Rect, record: &scouty::record::LogRecord) {
        let chunks = Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);

        // Left pane: raw log content with wrap
        let left_block = Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(Color::DarkGray));
        let raw_text = Paragraph::new(record.raw.clone())
            .block(left_block)
            .wrap(Wrap { trim: false });
        frame.render_widget(raw_text, chunks[0]);

        // Right pane: field list
        let field_lines = build_field_lines(record);
        let fields = Paragraph::new(field_lines).wrap(Wrap { trim: false });
        frame.render_widget(fields, chunks[1]);
    }

    fn render_single_column(
        &self,
        frame: &mut Frame,
        area: Rect,
        record: &scouty::record::LogRecord,
    ) {
        // Same as old layout: fields then message then raw
        let mut lines = build_field_lines(record);
        lines.push(Line::from(""));
        lines.push(Line::styled("Message:", Style::default().fg(Color::Cyan)));
        lines.push(Line::from(record.message.clone()));

        let detail = Paragraph::new(lines).wrap(Wrap { trim: false });
        frame.render_widget(detail, area);
    }
}

impl UiComponent for DetailPanelWidget {
    fn render(&self, _frame: &mut Frame, _area: Rect) {}

    fn enable_jk_navigation(&self) -> bool {
        false
    }
}
