//! Save preset dialog — text input for preset name.

#[cfg(test)]
#[path = "save_preset_window_tests.rs"]
mod save_preset_window_tests;

use crate::text_input::TextInput;
use crate::ui::{ComponentResult, UiComponent};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub struct SavePresetWindow {
    pub input: TextInput,
    pub confirmed: bool,
}

impl SavePresetWindow {
    pub fn new() -> Self {
        Self {
            input: TextInput::new(),
            confirmed: false,
        }
    }
}

impl UiComponent for SavePresetWindow {
    fn enable_jk_navigation(&self) -> bool {
        false
    }

    fn on_confirm(&mut self) -> ComponentResult {
        if !self.input.is_empty() {
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
        self.input.insert(c);
        ComponentResult::Consumed
    }

    fn on_key(&mut self, key: KeyEvent) -> ComponentResult {
        match key.code {
            KeyCode::Backspace => {
                self.input.backspace();
                ComponentResult::Consumed
            }
            KeyCode::Delete => {
                self.input.delete();
                ComponentResult::Consumed
            }
            KeyCode::Left => {
                self.input.move_left();
                ComponentResult::Consumed
            }
            KeyCode::Right => {
                self.input.move_right();
                ComponentResult::Consumed
            }
            _ => ComponentResult::Ignored,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let width = 40u16.min(area.width.saturating_sub(4));
        let height = 5u16;
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        let overlay = Rect::new(x, y, width, height);

        frame.render_widget(Clear, overlay);

        let block = Block::default()
            .title(" Save Filter Preset ")
            .borders(Borders::ALL);

        let inner = block.inner(overlay);
        frame.render_widget(block, overlay);

        if inner.height >= 2 {
            let label = Paragraph::new(" Name:");
            frame.render_widget(label, Rect::new(inner.x, inner.y, inner.width, 1));

            let input_text = format!(" {}", self.input.value());
            let input_line = Paragraph::new(input_text);
            frame.render_widget(input_line, Rect::new(inner.x, inner.y + 1, inner.width, 1));
        }
    }
}
