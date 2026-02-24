//! Load preset dialog — list of available presets with navigation.

#[cfg(test)]
#[path = "load_preset_window_tests.rs"]
mod load_preset_window_tests;

use crate::ui::{ComponentResult, UiComponent};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub struct LoadPresetWindow {
    pub presets: Vec<String>,
    pub cursor: usize,
    pub selected: Option<String>,
    pub delete_name: Option<String>,
    pub confirmed: bool,
}

impl LoadPresetWindow {
    pub fn new(presets: Vec<String>) -> Self {
        Self {
            cursor: 0,
            presets,
            selected: None,
            delete_name: None,
            confirmed: false,
        }
    }
}

impl UiComponent for LoadPresetWindow {
    fn enable_jk_navigation(&self) -> bool {
        true
    }

    fn on_up(&mut self) -> ComponentResult {
        self.cursor = self.cursor.saturating_sub(1);
        ComponentResult::Consumed
    }

    fn on_down(&mut self) -> ComponentResult {
        if !self.presets.is_empty() && self.cursor + 1 < self.presets.len() {
            self.cursor += 1;
        }
        ComponentResult::Consumed
    }

    fn on_confirm(&mut self) -> ComponentResult {
        if !self.presets.is_empty() {
            self.selected = Some(self.presets[self.cursor].clone());
            self.confirmed = true;
            ComponentResult::Close
        } else {
            ComponentResult::Consumed
        }
    }

    fn on_cancel(&mut self) -> ComponentResult {
        ComponentResult::Close
    }

    fn on_char(&mut self, c: char) -> ComponentResult {
        if c == 'd' && !self.presets.is_empty() {
            self.delete_name = Some(self.presets[self.cursor].clone());
            self.presets.remove(self.cursor);
            if self.cursor > 0 && self.cursor >= self.presets.len() {
                self.cursor = self.presets.len().saturating_sub(1);
            }
            ComponentResult::Consumed
        } else {
            ComponentResult::Ignored
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let width = 40u16.min(area.width.saturating_sub(4));
        let height = (self.presets.len() as u16 + 5)
            .min(area.height.saturating_sub(4))
            .max(6);
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        let overlay = Rect::new(x, y, width, height);

        frame.render_widget(Clear, overlay);

        let block = Block::default()
            .title(" Load Filter Preset ")
            .borders(Borders::ALL);
        let inner = block.inner(overlay);
        frame.render_widget(block, overlay);

        let mut lines = Vec::new();

        if self.presets.is_empty() {
            lines.push(Line::from(" (no saved presets)"));
        } else {
            for (i, name) in self.presets.iter().enumerate() {
                let style = if i == self.cursor {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };
                lines.push(Line::styled(format!(" {}", name), style));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(" Enter: Load  d: Delete  Esc: Close"));

        let content = Paragraph::new(lines);
        frame.render_widget(content, inner);
    }
}
