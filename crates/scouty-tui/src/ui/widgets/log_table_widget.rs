//! Log table widget — main scrollable log table.

#[cfg(test)]
#[path = "log_table_widget_tests.rs"]
mod log_table_widget_tests;

use crate::app::{App, Column};
use crate::config::Theme;
use crate::ui::{ComponentResult, UiComponent};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Cell, Row, Table};
use ratatui::Frame;
use scouty::record::LogLevel;

pub fn level_style(level: Option<LogLevel>, theme: &Theme) -> Style {
    match level {
        Some(LogLevel::Fatal) => theme.log_levels.fatal.to_style(),
        Some(LogLevel::Error) => theme.log_levels.error.to_style(),
        Some(LogLevel::Warn) => theme.log_levels.warn.to_style(),
        Some(LogLevel::Notice) => theme.log_levels.notice.to_style(),
        Some(LogLevel::Info) => theme.log_levels.info.to_style(),
        Some(LogLevel::Debug) => theme.log_levels.debug.to_style(),
        Some(LogLevel::Trace) => theme.log_levels.trace.to_style(),
        None => Style::default(),
    }
}

pub struct LogTableWidget;

impl LogTableWidget {
    pub fn render_with_app(&self, frame: &mut Frame, area: Rect, app: &App) {
        let theme = &app.theme;
        let visible = app.visible_records();
        let cw = &app.col_widths;
        let vis_cols = app.column_config.visible_columns();

        let sep_style = Style::default().fg(Color::DarkGray);

        let widths: Vec<Constraint> = vis_cols
            .iter()
            .enumerate()
            .flat_map(|(i, col)| {
                let w = match col {
                    Column::Time => Constraint::Length(cw[0]),
                    Column::Level => Constraint::Length(cw[1]),
                    Column::Hostname => Constraint::Length(20),
                    Column::Container => Constraint::Length(15),
                    Column::Context => Constraint::Length(25),
                    Column::Function => Constraint::Length(10),
                    Column::ProcessName => Constraint::Length(cw[2]),
                    Column::Pid => Constraint::Length(cw[3]),
                    Column::Tid => Constraint::Length(cw[4]),
                    Column::Component => Constraint::Length(cw[5]),
                    Column::Source => Constraint::Length(15),
                    Column::Log => Constraint::Fill(1),
                };
                if i < vis_cols.len() - 1 {
                    vec![w, Constraint::Length(1)]
                } else {
                    vec![w]
                }
            })
            .collect();

        let header_cells: Vec<Cell> = vis_cols
            .iter()
            .enumerate()
            .flat_map(|(i, col)| {
                let cell =
                    Cell::from(col.label()).style(Style::default().add_modifier(Modifier::BOLD));
                if i < vis_cols.len() - 1 {
                    vec![cell, Cell::from("│").style(sep_style)]
                } else {
                    vec![cell]
                }
            })
            .collect();

        let header = Row::new(header_cells).style(theme.table.header.to_style());

        let rows: Vec<Row> = visible
            .iter()
            .enumerate()
            .map(|(i, record)| {
                let filtered_idx = app.scroll_offset + i;
                let is_selected = filtered_idx == app.selected;
                let is_match = app.is_search_match(filtered_idx);
                let record_idx = app.filtered_indices[filtered_idx];
                let is_bookmarked = app.is_bookmarked(record_idx);
                let row_style = level_style(record.level, theme);

                let cells: Vec<Cell> = vis_cols
                    .iter()
                    .enumerate()
                    .flat_map(|(ci, col)| {
                        let cell = match col {
                            Column::Time => {
                                Cell::from(record.timestamp.format("%Y-%m-%d %H:%M:%S").to_string())
                            }
                            Column::Level => Cell::from(
                                record.level.map(|l| format!("{}", l)).unwrap_or_default(),
                            ),
                            Column::Hostname => {
                                Cell::from(record.hostname.as_deref().unwrap_or("").to_string())
                            }
                            Column::Container => {
                                Cell::from(record.container.as_deref().unwrap_or("").to_string())
                            }
                            Column::Context => {
                                Cell::from(record.context.as_deref().unwrap_or("").to_string())
                            }
                            Column::Function => {
                                Cell::from(record.function.as_deref().unwrap_or("").to_string())
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
                            Column::Component => Cell::from(
                                record.component_name.as_deref().unwrap_or("").to_string(),
                            ),
                            Column::Source => Cell::from(record.source.to_string()),
                            Column::Log => Cell::from(record.message.clone()),
                        };
                        if ci < vis_cols.len() - 1 {
                            vec![cell, Cell::from("│").style(sep_style)]
                        } else {
                            vec![cell]
                        }
                    })
                    .collect();

                // Determine highlight row background: last matching rule wins
                let highlight_bg: Option<Color> = if app.highlight_rules.is_empty() {
                    None
                } else {
                    let mut bg = None;
                    for rule in &app.highlight_rules {
                        if rule.regex.is_match(&record.message) {
                            bg = Some(rule.color);
                        }
                    }
                    bg
                };

                let mut row = Row::new(cells).style(row_style);
                if is_selected && is_match {
                    row = row.style(row_style.bg(theme.table.selected_search.bg_color()));
                } else if is_selected && is_bookmarked {
                    row = row.style(row_style.bg(theme.table.selected_highlight.bg_color()));
                } else if is_selected {
                    row = row.style(row_style.bg(theme.table.selected.bg_color()));
                } else if is_match {
                    row = row.style(row_style.bg(theme.table.search_match.bg_color()));
                } else if is_bookmarked {
                    row = row.style(row_style.bg(theme.table.bookmark.bg_color()));
                } else if let Some(bg) = highlight_bg {
                    row = row.style(Style::default().bg(bg).fg(Color::Black));
                }
                row
            })
            .collect();

        let table = Table::new(rows, widths).header(header).column_spacing(0);
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
