//! Column selector dialog (c key).

#[cfg(test)]
#[path = "column_selector_window_tests.rs"]
mod column_selector_window_tests;

use crate::app::{App, Column};
use crate::ui::{ComponentResult, UiComponent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

/// Column selector overlay.
#[allow(dead_code)]
pub struct ColumnSelectorWindow {
    pub cursor: usize,
    pub columns: Vec<(Column, bool)>,
}

#[allow(dead_code)]
impl ColumnSelectorWindow {
    pub fn from_app(app: &App) -> Self {
        Self {
            cursor: app.column_config.cursor,
            columns: app.column_config.columns.clone(),
        }
    }

    pub fn sync_to_app(&self, app: &mut App) {
        app.column_config.cursor = self.cursor;
        app.column_config.columns = self.columns.clone();
    }

    fn toggle_current(&mut self) {
        let cur = self.cursor;
        if cur < self.columns.len() && self.columns[cur].0 != Column::Log {
            self.columns[cur].1 = !self.columns[cur].1;
        }
    }
}

#[allow(dead_code)]
impl UiComponent for ColumnSelectorWindow {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let width = 35u16.min(area.width.saturating_sub(4));
        let height = (self.columns.len() as u16 + 5).min(area.height.saturating_sub(4));
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

        for (i, (col, visible)) in self.columns.iter().enumerate() {
            let checkbox = if *visible { "[x]" } else { "[ ]" };
            let is_cursor = i == self.cursor;
            let suffix = if *col == Column::Log {
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

    fn enable_jk_navigation(&self) -> bool {
        true
    }

    fn on_up(&mut self) -> ComponentResult {
        self.cursor = self.cursor.saturating_sub(1);
        ComponentResult::Consumed
    }

    fn on_down(&mut self) -> ComponentResult {
        if self.cursor + 1 < self.columns.len() {
            self.cursor += 1;
        }
        ComponentResult::Consumed
    }

    fn on_toggle(&mut self) -> ComponentResult {
        self.toggle_current();
        ComponentResult::Consumed
    }

    fn on_confirm(&mut self) -> ComponentResult {
        self.toggle_current();
        ComponentResult::Consumed
    }

    fn on_cancel(&mut self) -> ComponentResult {
        ComponentResult::Close
    }
}
