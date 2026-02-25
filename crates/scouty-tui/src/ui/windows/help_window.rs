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
            section("Global"),
            Line::from("  Esc              Close overlay / cancel input"),
            Line::from("  q                Quit"),
            Line::from("  ?                Help"),
            Line::from(""),
            section("Log Table View — Navigation"),
            Line::from("  j / k            Move up / down one row"),
            Line::from("  Ctrl+j / Ctrl+k  Page up / down"),
            Line::from("  g / G            First / last row"),
            Line::from("  Ctrl+G           Go to line number"),
            Line::from("  ] / [            Time jump forward / backward"),
            Line::from("  Ctrl+]           Toggle follow mode"),
            Line::from("  Enter            Toggle detail panel"),
            Line::from(""),
            section("Log Table View — Search & Filter"),
            Line::from("  /                Search (regex)"),
            Line::from("  n / N            Next / prev search match"),
            Line::from("  f                Filter expression input"),
            Line::from("  - / =            Quick exclude / include text"),
            Line::from("  _ / +            Field exclude / include dialog"),
            Line::from("  F                Filter manager"),
            Line::from("  l                Log level quick filter (1-8)"),
            Line::from(""),
            section("Log Table View — Display & Analysis"),
            Line::from("  c                Column selector"),
            Line::from("  d / D            Cycle / select density chart source"),
            Line::from("  h / H            Add highlight / highlight manager"),
            Line::from("  r / R            Region manager / next region"),
            Line::from("  S                Stats summary"),
            Line::from(""),
            section("Log Table View — Bookmarks"),
            Line::from("  m                Toggle bookmark"),
            Line::from("  ' / \"            Next / prev bookmark"),
            Line::from("  M                Bookmark manager"),
            Line::from(""),
            section("Log Table View — Copy & Export"),
            Line::from("  y / Y            Copy raw / format selection"),
            Line::from("  s                Save / export dialog"),
            Line::from(""),
            section("Dialog Navigation"),
            Line::from("  j / k / ↑ / ↓    Move selection"),
            Line::from("  PgUp / PgDn      Page through options"),
            Line::from("  Space            Toggle selection"),
            Line::from("  Enter            Confirm"),
            Line::from("  Esc              Cancel / close"),
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
