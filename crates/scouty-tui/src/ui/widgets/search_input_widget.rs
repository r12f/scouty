//! Search input widget — / search bar.

#[cfg(test)]
#[path = "search_input_widget_tests.rs"]
mod search_input_widget_tests;

use crate::app::App;
use crate::ui::{ComponentResult, UiComponent};
use ratatui::layout::Rect;

use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

#[allow(dead_code)]
pub struct SearchInputWidget;

#[allow(dead_code)]
impl SearchInputWidget {
    pub fn render_with_app(&self, frame: &mut Frame, area: Rect, app: &App) {
        let theme = &app.theme;
        let (before, cursor_ch, after) = app.search_input.render_parts();
        let input_line = Paragraph::new(Line::from(vec![
            Span::styled("/", theme.input.prompt.to_style()),
            Span::raw(before),
            Span::styled(cursor_ch, theme.input.cursor.to_style()),
            Span::raw(after),
        ]));
        frame.render_widget(input_line, area);
    }
}

impl UiComponent for SearchInputWidget {
    fn render(&self, _frame: &mut Frame, _area: Rect) {}

    fn enable_jk_navigation(&self) -> bool {
        false
    }

    fn on_cancel(&mut self) -> ComponentResult {
        ComponentResult::Close
    }

    fn on_confirm(&mut self) -> ComponentResult {
        ComponentResult::Close
    }

    fn on_char(&mut self, _c: char) -> ComponentResult {
        ComponentResult::Consumed
    }
}
