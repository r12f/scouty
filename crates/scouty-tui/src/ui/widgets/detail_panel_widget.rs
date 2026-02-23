//! Detail panel widget — expanded view of the selected log record.

#[cfg(test)]
#[path = "detail_panel_widget_tests.rs"]
mod detail_panel_widget_tests;

use crate::app::App;
use crate::config::Theme;
use crate::ui::UiComponent;
use ratatui::layout::{Constraint, Layout, Rect};

use crate::ui::widgets::log_table_widget::level_style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap};
use ratatui::Frame;

/// Count the number of field rows that would be displayed for a record,
/// without allocating the full field pairs vector.
pub(crate) fn field_count(record: &scouty::record::LogRecord) -> usize {
    // Always-present: Timestamp, Level, Source
    let mut count: usize = 3;

    if record.hostname.is_some() {
        count += 1;
    }
    if record.container.is_some() {
        count += 1;
    }
    if record.context.is_some() {
        count += 1;
    }
    if record.function.is_some() {
        count += 1;
    }
    if record.component_name.is_some() {
        count += 1;
    }
    if record.process_name.is_some() {
        count += 1;
    }
    if record.pid.is_some() {
        count += 1;
    }
    if record.tid.is_some() {
        count += 1;
    }

    if let Some(meta) = record.metadata.as_ref() {
        count += meta.len();
    }

    count
}

/// Build field key-value pairs for the right pane.
fn build_field_pairs(record: &scouty::record::LogRecord) -> Vec<(&'static str, String)> {
    let mut pairs = vec![
        (
            "Timestamp",
            record.timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
        ),
        (
            "Level",
            record
                .level
                .map(|l| l.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ),
        ("Source", record.source.to_string()),
    ];

    let optional_fields: Vec<(&str, Option<String>)> = vec![
        ("Hostname", record.hostname.clone()),
        ("Container", record.container.clone()),
        ("Context", record.context.clone()),
        ("Function", record.function.clone()),
        ("Component", record.component_name.clone()),
        ("Process", record.process_name.clone()),
        ("PID", record.pid.map(|p| p.to_string())),
        ("TID", record.tid.map(|t| t.to_string())),
    ];

    for (label, value) in optional_fields {
        if let Some(val) = value {
            pairs.push((label, val));
        }
    }

    if record.metadata.as_ref().is_some_and(|m| !m.is_empty()) {
        for (k, v) in record.metadata.as_ref().unwrap() {
            // Leak is fine since these are short-lived display strings
            // Use a static prefix instead
            pairs.push(("Meta", format!("{} = {}", k, v)));
        }
    }

    pairs
}

/// Build Line spans from field pairs (for single-column fallback).
fn build_field_lines(record: &scouty::record::LogRecord, theme: &Theme) -> Vec<Line<'static>> {
    let label_style = theme.detail_panel.field_name.to_style();
    build_field_pairs(record)
        .into_iter()
        .map(|(key, val)| {
            let padded_key = format!("{:<11}", format!("{}:", key));
            Line::from(vec![Span::styled(padded_key, label_style), Span::raw(val)])
        })
        .collect()
}

pub struct DetailPanelWidget;

/// Minimum total width to show split layout.
const MIN_SPLIT_WIDTH: u16 = 80;

impl DetailPanelWidget {
    pub fn render_with_app(&self, frame: &mut Frame, area: Rect, app: &App) {
        let theme = &app.theme;
        let block = Block::default()
            .title(" Detail ")
            .borders(Borders::TOP)
            .border_style(theme.detail_panel.border.to_style());

        let Some(record) = app.selected_record() else {
            let empty = Paragraph::new("No record selected").block(block);
            frame.render_widget(empty, area);
            return;
        };

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.width < MIN_SPLIT_WIDTH {
            self.render_single_column(frame, inner, record, theme);
        } else {
            self.render_split(frame, inner, record, theme);
        }
    }

    fn render_split(
        &self,
        frame: &mut Frame,
        area: Rect,
        record: &scouty::record::LogRecord,
        theme: &Theme,
    ) {
        let chunks = Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);

        let left_block = Block::default()
            .title(" Log Content ")
            .borders(Borders::RIGHT)
            .border_style(theme.detail_panel.border.to_style());
        let raw_text = Paragraph::new(record.raw.clone())
            .block(left_block)
            .wrap(Wrap { trim: false });
        frame.render_widget(raw_text, chunks[0]);

        let pairs = build_field_pairs(record);
        let label_style = theme.detail_panel.field_name.to_style();
        let rows: Vec<Row> = pairs
            .into_iter()
            .map(|(key, val)| {
                let val_cell = if key == "Level" {
                    Cell::from(Span::styled(val, level_style(record.level, theme)))
                } else {
                    Cell::from(val)
                };
                Row::new(vec![Cell::from(Span::styled(key, label_style)), val_cell])
            })
            .collect();
        let right_block = Block::default()
            .title(" Fields ")
            .border_style(theme.detail_panel.border.to_style());
        let table = Table::new(rows, [Constraint::Length(11), Constraint::Fill(1)])
            .column_spacing(1)
            .block(right_block);
        frame.render_widget(table, chunks[1]);
    }

    fn render_single_column(
        &self,
        frame: &mut Frame,
        area: Rect,
        record: &scouty::record::LogRecord,
        theme: &Theme,
    ) {
        let mut lines = build_field_lines(record, theme);
        lines.push(Line::from(""));
        lines.push(Line::styled(
            "Message:",
            theme.detail_panel.section_header.to_style(),
        ));
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
