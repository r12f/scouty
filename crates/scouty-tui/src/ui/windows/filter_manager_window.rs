//! Filter manager panel (F / Ctrl+F).

#[cfg(test)]
#[path = "filter_manager_window_tests.rs"]
mod filter_manager_window_tests;

use crate::app::App;
use crate::ui::{ComponentResult, UiComponent};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

/// Filter manager overlay — shows active filters, allows deletion.
#[allow(dead_code)]
pub struct FilterManagerWindow {
    pub cursor: usize,
    pub filter_count: usize,
    pub action: Option<&'static str>,
}

#[allow(dead_code)]
impl FilterManagerWindow {
    pub fn from_app(app: &App) -> Self {
        Self {
            cursor: app.filter_manager_cursor,
            filter_count: app.filters.len(),
            action: None,
        }
    }

    pub fn render_with_app(&self, frame: &mut Frame, app: &App, area: Rect) {
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
                let is_cursor = i == self.cursor;
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

    pub fn apply_to_app(&self, app: &mut App) {
        app.filter_manager_cursor = self.cursor;
        match self.action {
            Some("delete") => {
                if !app.filters.is_empty() {
                    let idx = self.cursor;
                    app.remove_filter(idx);
                    if app.filter_manager_cursor > 0
                        && app.filter_manager_cursor >= app.filters.len()
                    {
                        app.filter_manager_cursor = app.filters.len().saturating_sub(1);
                    }
                }
            }
            Some("clear") => {
                app.clear_filters();
                app.filter_manager_cursor = 0;
            }
            _ => {}
        }
    }
}

#[allow(dead_code)]
impl UiComponent for FilterManagerWindow {
    fn render(&self, _frame: &mut Frame, _area: Rect) {
        // Use render_with_app instead
    }

    fn enable_jk_navigation(&self) -> bool {
        true
    }

    fn on_up(&mut self) -> ComponentResult {
        self.cursor = self.cursor.saturating_sub(1);
        ComponentResult::Consumed
    }

    fn on_down(&mut self) -> ComponentResult {
        if self.filter_count > 0 && self.cursor + 1 < self.filter_count {
            self.cursor += 1;
        }
        ComponentResult::Consumed
    }

    fn on_page_up(&mut self) -> ComponentResult {
        self.cursor = self.cursor.saturating_sub(10);
        ComponentResult::Consumed
    }

    fn on_page_down(&mut self) -> ComponentResult {
        if self.filter_count > 0 {
            self.cursor = (self.cursor + 10).min(self.filter_count.saturating_sub(1));
        }
        ComponentResult::Consumed
    }

    fn on_confirm(&mut self) -> ComponentResult {
        ComponentResult::Close
    }

    fn on_cancel(&mut self) -> ComponentResult {
        ComponentResult::Close
    }

    fn on_char(&mut self, c: char) -> ComponentResult {
        match c {
            'd' => {
                self.action = Some("delete");
                ComponentResult::Consumed
            }
            'c' => {
                self.action = Some("clear");
                ComponentResult::Consumed
            }
            _ => ComponentResult::Ignored,
        }
    }

    fn on_key(&mut self, key: KeyEvent) -> ComponentResult {
        match key.code {
            KeyCode::Delete => {
                self.action = Some("delete");
                ComponentResult::Consumed
            }
            _ => ComponentResult::Ignored,
        }
    }
}
