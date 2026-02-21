//! Go to line number dialog (Ctrl+G).

#[cfg(test)]
#[path = "goto_line_window_tests.rs"]
mod goto_line_window_tests;

use crate::ui::{ComponentResult, UiComponent};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::Frame;

/// Minimal component for goto-line input mode.
/// Actual rendering is in the footer (input bar), so render() is a no-op.
#[allow(dead_code)]
pub struct GotoLineWindow {
    pub input: String,
    pub confirmed: bool,
}

#[allow(dead_code)]
impl GotoLineWindow {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            confirmed: false,
        }
    }
}

#[allow(dead_code)]
impl UiComponent for GotoLineWindow {
    fn render(&self, _frame: &mut Frame, _area: Rect) {}

    fn enable_jk_navigation(&self) -> bool {
        false
    }

    fn on_confirm(&mut self) -> ComponentResult {
        self.confirmed = true;
        ComponentResult::Close
    }

    fn on_cancel(&mut self) -> ComponentResult {
        ComponentResult::Close
    }

    fn on_char(&mut self, c: char) -> ComponentResult {
        if c.is_ascii_digit() {
            self.input.push(c);
            ComponentResult::Consumed
        } else {
            ComponentResult::Ignored
        }
    }

    fn on_key(&mut self, key: KeyEvent) -> ComponentResult {
        match key.code {
            KeyCode::Backspace => {
                self.input.pop();
                ComponentResult::Consumed
            }
            _ => ComponentResult::Ignored,
        }
    }
}
