//! Copy format selection dialog (Y key).

#[cfg(test)]
#[path = "copy_format_window_tests.rs"]
mod copy_format_window_tests;

use crate::app::{self, App, CopyFormat};
use crate::ui::{ComponentResult, UiComponent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

/// Popup dialog for selecting copy format (Raw/JSON/YAML).
pub struct CopyFormatWindow;

impl CopyFormatWindow {
    /// Handle the selected format: copy to clipboard and return Close.
    pub fn select_format(app: &mut App, format: CopyFormat) -> ComponentResult {
        if let Some(text) = app.copy_as_format(format) {
            app::osc52_copy(&text);
        }
        ComponentResult::Close
    }
}

impl UiComponent for CopyFormatWindow {
    fn enable_jk_navigation(&self) -> bool {
        false
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let width = 35u16.min(area.width.saturating_sub(4));
        let height = 9u16.min(area.height.saturating_sub(4));
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        let overlay = Rect::new(x, y, width, height);

        frame.render_widget(Clear, overlay);

        let lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    " [r] ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Raw text (default)"),
            ]),
            Line::from(vec![
                Span::styled(
                    " [j] ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("JSON"),
            ]),
            Line::from(vec![
                Span::styled(
                    " [y] ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("YAML"),
            ]),
            Line::from(""),
            Line::styled(
                " Enter: Raw  Esc: Cancel",
                Style::default().fg(Color::DarkGray),
            ),
        ];

        let dialog = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Copy As (Y) ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .style(Style::default().bg(Color::Black));
        frame.render_widget(dialog, overlay);
    }

    fn on_confirm(&mut self) -> ComponentResult {
        // Enter defaults to Raw — actual copy happens in the dispatch layer
        // because we need access to App state.
        ComponentResult::Close
    }

    fn on_char(&mut self, c: char) -> ComponentResult {
        match c {
            'r' | 'j' | 'y' => ComponentResult::Close,
            _ => ComponentResult::Ignored,
        }
    }
}
