//! Highlight manager overlay (H).

#[cfg(test)]
#[path = "highlight_manager_window_tests.rs"]
mod highlight_manager_window_tests;

use crate::app::App;
use crate::ui::{ComponentResult, UiComponent};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

/// Highlight manager overlay — shows highlight rules, allows deletion.
pub struct HighlightManagerWindow {
    pub cursor: usize,
    pub rule_count: usize,
    pub action: Option<&'static str>,
}

impl HighlightManagerWindow {
    pub fn from_app(app: &App) -> Self {
        Self {
            cursor: app.highlight_manager_cursor,
            rule_count: app.highlight_rules.len(),
            action: None,
        }
    }

    pub fn render_with_app(&self, frame: &mut Frame, app: &App, area: Rect) {
        let theme = &app.theme;
        let width = 55u16.min(area.width.saturating_sub(4));
        let height = (app.highlight_rules.len() as u16 + 7)
            .min(area.height.saturating_sub(4))
            .max(8);
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        let overlay = Rect::new(x, y, width, height);

        frame.render_widget(Clear, overlay);

        let mut lines = vec![
            Line::styled(
                format!(" Highlight Rules ({})", app.highlight_rules.len()),
                theme.dialog.text.to_style().add_modifier(Modifier::BOLD),
            ),
            Line::from(""),
        ];

        if app.highlight_rules.is_empty() {
            lines.push(Line::styled(
                " (no highlight rules)",
                theme.dialog.muted.to_style(),
            ));
        } else {
            for (i, rule) in app.highlight_rules.iter().enumerate() {
                let is_cursor = i == self.cursor;
                let color_name = match rule.color {
                    Color::Red => "Red",
                    Color::Green => "Green",
                    Color::Blue => "Blue",
                    Color::Yellow => "Yellow",
                    Color::Magenta => "Magenta",
                    Color::Cyan => "Cyan",
                    _ => "?",
                };
                let style = if is_cursor {
                    theme.dialog.selected.to_style().fg(rule.color)
                } else {
                    Style::default().fg(rule.color)
                };
                lines.push(Line::styled(
                    format!(" [{}] {}", color_name, rule.pattern),
                    style,
                ));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::styled(
            " d: Delete  Esc: Close",
            theme.dialog.muted.to_style(),
        ));

        let dialog = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Highlight Manager (H) ")
                    .borders(Borders::ALL)
                    .border_style(theme.dialog.border.to_style()),
            )
            .style(theme.dialog.background.to_style());
        frame.render_widget(dialog, overlay);
    }

    pub fn apply_to_app(&self, app: &mut App) {
        app.highlight_manager_cursor = self.cursor;
        if let Some("delete") = self.action {
            if !app.highlight_rules.is_empty() {
                let idx = self.cursor;
                app.remove_highlight_rule(idx);
                if app.highlight_manager_cursor > 0
                    && app.highlight_manager_cursor >= app.highlight_rules.len()
                {
                    app.highlight_manager_cursor = app.highlight_rules.len().saturating_sub(1);
                }
            }
        }
    }
}

impl UiComponent for HighlightManagerWindow {
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
        if self.rule_count > 0 && self.cursor + 1 < self.rule_count {
            self.cursor += 1;
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
