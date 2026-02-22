//! Help overlay window (? key).

#[cfg(test)]
#[path = "help_window_tests.rs"]
mod help_window_tests;

use crate::ui::{ComponentResult, UiComponent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

/// Help overlay showing keyboard shortcuts.
#[allow(dead_code)]
pub struct HelpWindow;

#[allow(dead_code)]
impl UiComponent for HelpWindow {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let width = 58u16.min(area.width.saturating_sub(4));
        let height = 36u16.min(area.height.saturating_sub(4));
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        let overlay = Rect::new(x, y, width, height);

        frame.render_widget(Clear, overlay);

        let section = |title: &str| {
            Line::styled(
                format!(" {title}"),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        };

        let help_text = vec![
            Line::styled(
                " Keyboard Shortcuts ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from(""),
            section("Navigation"),
            Line::from("  j / ↓            Move down one row"),
            Line::from("  k / ↑            Move up one row"),
            Line::from("  Ctrl+j/k / PgDn  Page down / up"),
            Line::from("  g / Home         Jump to first row"),
            Line::from("  G / End          Jump to last row"),
            Line::from("  Ctrl+G           Go to line number"),
            Line::from("  t                Jump to timestamp"),
            Line::from("  Ctrl+]           Toggle follow mode"),
            Line::from(""),
            section("Search & Filter"),
            Line::from("  /                Search (regex)"),
            Line::from("  n / N            Next / prev search match"),
            Line::from("  f                Filter expression"),
            Line::from("  - / =            Quick exclude / include"),
            Line::from("  _ / +            Exclude / include field filter"),
            Line::from("  Ctrl+F           Filter manager"),
            Line::from(""),
            section("Display"),
            Line::from("  Enter            Toggle detail panel"),
            Line::from("  Ctrl+C           Column selector"),
            Line::from(""),
            section("General"),
            Line::from("  ?                Show this help"),
            Line::from("  Esc              Close dialog / panel"),
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

    fn on_cancel(&mut self) -> ComponentResult {
        ComponentResult::Close
    }

    fn on_key(&mut self, _key: crossterm::event::KeyEvent) -> ComponentResult {
        ComponentResult::Close
    }

    fn on_char(&mut self, _c: char) -> ComponentResult {
        ComponentResult::Close
    }

    fn on_confirm(&mut self) -> ComponentResult {
        ComponentResult::Close
    }

    fn on_toggle(&mut self) -> ComponentResult {
        ComponentResult::Close
    }
}
