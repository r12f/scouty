//! Copy format selection dialog (Y key).

#[cfg(test)]
#[path = "copy_format_window_tests.rs"]
mod copy_format_window_tests;

use crate::app::{self, App, CopyFormat};
use crate::config::Theme;
use crate::ui::{ComponentResult, UiComponent};
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

const FORMAT_OPTIONS: [(&str, CopyFormat); 3] = [
    ("Raw text", CopyFormat::Raw),
    ("JSON", CopyFormat::Json),
    ("YAML", CopyFormat::Yaml),
];

/// Popup dialog for selecting copy format (Raw/JSON/YAML).
pub struct CopyFormatWindow {
    pub cursor: usize,
    pub confirmed: bool,
    pub theme: Theme,
}

impl CopyFormatWindow {
    pub fn from_app(app: &App) -> Self {
        Self {
            cursor: app.copy_format_cursor,
            confirmed: false,
            theme: app.theme.clone(),
        }
    }

    /// Get the selected format.
    pub fn selected_format(&self) -> CopyFormat {
        FORMAT_OPTIONS[self.cursor].1
    }

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
        true
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let t = &self.theme;
        let width = 30u16.min(area.width.saturating_sub(4));
        let height = 8u16.min(area.height.saturating_sub(4));
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        let overlay = Rect::new(x, y, width, height);

        frame.render_widget(Clear, overlay);

        let mut lines = vec![Line::from("")];

        for (i, (label, _)) in FORMAT_OPTIONS.iter().enumerate() {
            let is_selected = i == self.cursor;
            let marker = if is_selected { "▸ " } else { "  " };
            let style = if is_selected {
                t.dialog.accent.to_style().add_modifier(Modifier::BOLD)
            } else {
                t.dialog.text.to_style()
            };
            lines.push(Line::from(Span::styled(
                format!(" {}{}", marker, label),
                style,
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::styled(
            " Enter: Copy  Esc: Cancel",
            t.dialog.muted.to_style(),
        ));

        let dialog = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Copy As (Y) ")
                    .borders(Borders::ALL)
                    .border_style(t.dialog.border.to_style()),
            )
            .style(t.dialog.background.to_style());
        frame.render_widget(dialog, overlay);
    }

    fn on_up(&mut self) -> ComponentResult {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
        ComponentResult::Consumed
    }

    fn on_down(&mut self) -> ComponentResult {
        if self.cursor < FORMAT_OPTIONS.len() - 1 {
            self.cursor += 1;
        }
        ComponentResult::Consumed
    }

    fn on_confirm(&mut self) -> ComponentResult {
        self.confirmed = true;
        ComponentResult::Close
    }

    fn on_cancel(&mut self) -> ComponentResult {
        self.confirmed = false;
        ComponentResult::Close
    }
}
