//! Detail panel widget — expanded view of the selected log record.

#[cfg(test)]
#[path = "detail_panel_widget_tests.rs"]
mod detail_panel_widget_tests;

use crate::app::App;
use crate::ui::UiComponent;
use ratatui::layout::Rect;
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

pub struct DetailPanelWidget;

impl DetailPanelWidget {
    pub fn render_with_app(&self, frame: &mut Frame, area: Rect, app: &App) {
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

            if let Some(ref host) = record.hostname {
                lines.push(Line::from(vec![
                    Span::styled("Hostname:  ", Style::default().fg(Color::Cyan)),
                    Span::raw(host),
                ]));
            }
            if let Some(ref ctr) = record.container {
                lines.push(Line::from(vec![
                    Span::styled("Container: ", Style::default().fg(Color::Cyan)),
                    Span::raw(ctr),
                ]));
            }
            if let Some(ref ctx) = record.context {
                lines.push(Line::from(vec![
                    Span::styled("Context:   ", Style::default().fg(Color::Cyan)),
                    Span::raw(ctx),
                ]));
            }
            if let Some(ref func) = record.function {
                lines.push(Line::from(vec![
                    Span::styled("Function:  ", Style::default().fg(Color::Cyan)),
                    Span::raw(func),
                ]));
            }
            if let Some(ref comp) = record.component_name {
                lines.push(Line::from(vec![
                    Span::styled("Component: ", Style::default().fg(Color::Cyan)),
                    Span::raw(comp),
                ]));
            }
            if let Some(ref proc_name) = record.process_name {
                lines.push(Line::from(vec![
                    Span::styled("Process:   ", Style::default().fg(Color::Cyan)),
                    Span::raw(proc_name),
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
}

impl UiComponent for DetailPanelWidget {
    fn render(&self, _frame: &mut Frame, _area: Rect) {}

    fn enable_jk_navigation(&self) -> bool {
        false
    }
}
