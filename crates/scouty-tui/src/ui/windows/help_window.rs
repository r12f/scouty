//! Help overlay window (? key).

#[cfg(test)]
#[path = "help_window_tests.rs"]
mod help_window_tests;

use crate::config::Theme;
use crate::ui::{ComponentResult, UiComponent};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

/// Help overlay showing keyboard shortcuts — scrollable with j/k.
pub struct HelpWindow<'a> {
    pub scroll: u16,
    pub visible_height: u16,
    pub theme: &'a Theme,
}

impl<'a> HelpWindow<'a> {
    pub fn new(theme: &'a Theme) -> Self {
        Self {
            scroll: 0,
            visible_height: 20,
            theme,
        }
    }

    fn help_lines(&self) -> Vec<Line<'static>> {
        let t = self.theme;
        let section = |title: &str| {
            Line::styled(
                format!(" {title}"),
                t.dialog.title.to_style().add_modifier(Modifier::BOLD),
            )
        };

        vec![
            Line::styled(
                " scouty-tui — TUI Log Viewer ",
                t.dialog.text.to_style().add_modifier(Modifier::BOLD),
            ),
            Line::styled(
                format!(" Version {}", env!("CARGO_PKG_VERSION")),
                t.dialog.muted.to_style(),
            ),
            Line::styled(
                " https://github.com/r12f/scouty",
                Style::default().fg(t.general.accent.fg_color()),
            ),
            Line::from(""),
            section("Navigation"),
            Line::from("  j / ↓            Move down one row"),
            Line::from("  k / ↑            Move up one row"),
            Line::from("  PgDn / PgUp       Page down / up"),
            Line::from("  g / Home         Jump to first row"),
            Line::from("  G / End          Jump to last row"),
            Line::from("  Ctrl+G           Go to line number"),
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
            Line::from("  Ctrl+S           Export filtered log to file"),
            Line::from(""),
            section("Commands"),
            Line::from("  :                Enter command mode"),
            Line::from("  :w <file>        Export filtered log to file"),
            Line::from("  :q               Quit"),
            Line::from(""),
            section("General"),
            Line::from("  y                Copy selected line"),
            Line::from("  Y                Copy format dialog"),
            Line::from("  ?                Show this help"),
            Line::from("  Esc              Close dialog / panel"),
            Line::from("  q                Quit (or close dialog)"),
            Line::from(""),
            Line::styled(
                " j/k to scroll • Esc/q to close ",
                t.dialog.muted.to_style(),
            ),
        ]
    }

    fn total_lines(&self) -> u16 {
        self.help_lines().len() as u16
    }
}

impl UiComponent for HelpWindow<'_> {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let t = self.theme;
        let width = 58u16.min(area.width.saturating_sub(4));
        let height = area.height.saturating_sub(4).min(area.height).max(3);
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        let overlay = Rect::new(x, y, width, height);

        frame.render_widget(Clear, overlay);

        let help_text = self.help_lines();

        let help = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title(" Help ")
                    .borders(Borders::ALL)
                    .border_style(t.dialog.accent.to_style()),
            )
            .style(t.dialog.background.to_style())
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
        let max_scroll = self.total_lines().saturating_sub(self.visible_height);
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
