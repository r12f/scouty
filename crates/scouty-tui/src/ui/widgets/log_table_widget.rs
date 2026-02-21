//! Log table widget — main scrollable log table.

#[cfg(test)]
#[path = "log_table_widget_tests.rs"]
mod log_table_widget_tests;

use crate::app::{App, Column};
use crate::ui::{ComponentResult, UiComponent};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Cell, Row, Table};
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

pub struct LogTableWidget;

impl LogTableWidget {
    pub fn render_with_app(&self, frame: &mut Frame, area: Rect, app: &App) {
        let visible = app.visible_records();
        let cw = &app.col_widths;
        let vis_cols = app.column_config.visible_columns();

        let widths: Vec<Constraint> = vis_cols
            .iter()
            .map(|col| match col {
                Column::Time => Constraint::Length(cw[0]),
                Column::Level => Constraint::Length(cw[1]),
                Column::Hostname => Constraint::Length(20),
                Column::Container => Constraint::Length(15),
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
                        Column::Hostname => {
                            Cell::from(record.hostname.as_deref().unwrap_or("").to_string())
                        }
                        Column::Container => {
                            Cell::from(record.container.as_deref().unwrap_or("").to_string())
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
}

impl UiComponent for LogTableWidget {
    fn render(&self, _frame: &mut Frame, _area: Rect) {}

    fn enable_jk_navigation(&self) -> bool {
        true
    }

    fn on_up(&mut self) -> ComponentResult {
        ComponentResult::Consumed
    }

    fn on_down(&mut self) -> ComponentResult {
        ComponentResult::Consumed
    }

    fn on_page_up(&mut self) -> ComponentResult {
        ComponentResult::Consumed
    }

    fn on_page_down(&mut self) -> ComponentResult {
        ComponentResult::Consumed
    }
}
