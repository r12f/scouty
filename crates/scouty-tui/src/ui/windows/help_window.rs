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

/// Help overlay showing keyboard shortcuts — scrollable with j/k.
pub struct HelpWindow {
    pub scroll: u16,
}

impl HelpWindow {
    pub fn new() -> Self {
        Self { scroll: 0 }
    }

    fn help_lines() -> Vec<Line<'static>> {
        let section = |title: &str| {
            Line::styled(
                format!(" {title}"),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        };

        vec![
            Line::styled(
                " scouty-tui — TUI Log Viewer ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::styled(
                format!(" Version {}", env!("CARGO_PKG_VERSION")),
                Style::default().fg(Color::DarkGray),
            ),
            Line::styled(
                " https://github.com/r12f/scouty",
                Style::default().fg(Color::Blue),
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
            Line::from("  ] / [            Time jump forward / back"),
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
            section("Bookmarks"),
            Line::from("  m                Toggle bookmark"),
            Line::from("  '                Jump to next bookmark"),
            Line::from("  \"                Jump to prev bookmark"),
            Line::from("  M                Bookmark manager"),
            Line::from(""),
            section("Highlight"),
            Line::from("  h                Quick highlight word"),
            Line::from("  H                Highlight manager"),
            Line::from(""),
            section("Display"),
            Line::from("  Enter            Toggle detail panel"),
            Line::from("  c                Column selector"),
            Line::from("  S                Statistics summary"),
            Line::from(""),
            section("Commands"),
            Line::from("  :                Enter command mode"),
            Line::from("  :w <file>        Export filtered log"),
            Line::from("  :q               Quit"),
            Line::from(""),
            section("General"),
            Line::from("  y                Copy selected line"),
            Line::from("  Y                Copy format dialog"),
            Line::from("  ?                Show this help"),
            Line::from("  Esc              Close dialog / panel"),
            Line::from("  q                Quit"),
            Line::from(""),
            Line::styled(
                " j/k to scroll • Esc/q to close ",
                Style::default().fg(Color::DarkGray),
            ),
        ]
    }

    fn total_lines() -> u16 {
        Self::help_lines().len() as u16
    }
}

impl UiComponent for HelpWindow {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let width = 58u16.min(area.width.saturating_sub(4));
        let height = area.height.saturating_sub(4).max(5);
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        let overlay = Rect::new(x, y, width, height);

        frame.render_widget(Clear, overlay);

        let help_text = Self::help_lines();

        let help = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title(" Help ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .style(Style::default().bg(Color::Black))
            .scroll((self.scroll, 0));
        frame.render_widget(help, overlay);
    }

    fn on_cancel(&mut self) -> ComponentResult {
        ComponentResult::Close
    }

    fn on_up(&mut self) -> ComponentResult {
        self.scroll = self.scroll.saturating_sub(1);
        ComponentResult::Consumed
    }

    fn on_down(&mut self) -> ComponentResult {
        let max_scroll = Self::total_lines().saturating_sub(1);
        if self.scroll < max_scroll {
            self.scroll += 1;
        }
        ComponentResult::Consumed
    }

    fn enable_jk_navigation(&self) -> bool {
        true
    }

    fn on_confirm(&mut self) -> ComponentResult {
        ComponentResult::Close
    }

    fn on_char(&mut self, c: char) -> ComponentResult {
        if c == 'q' {
            ComponentResult::Close
        } else {
            ComponentResult::Consumed
        }
    }
}
