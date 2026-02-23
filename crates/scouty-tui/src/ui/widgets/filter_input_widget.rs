//! Filter input widget — f filter expression bar.

#[cfg(test)]
#[path = "filter_input_widget_tests.rs"]
mod filter_input_widget_tests;

use crate::app::App;
use crate::ui::{ComponentResult, UiComponent};
use ratatui::layout::Rect;

use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

#[allow(dead_code)]
pub struct FilterInputWidget;

#[allow(dead_code)]
impl FilterInputWidget {
    pub fn render_with_app(&self, frame: &mut Frame, area: Rect, app: &App) {
        let theme = &app.theme;
        let (before, cursor_ch, after) = app.filter_input.render_parts();
        let mut spans = vec![
            Span::styled("Filter: ", theme.input.prompt.to_style()),
            Span::raw(before),
            Span::styled(cursor_ch, theme.input.cursor.to_style()),
            Span::raw(after),
        ];

        if let Some(ref err) = app.filter_error {
            spans.push(Span::styled(
                format!("  {}", err),
                theme.input.error.to_style(),
            ));
        }

        let input_line = Paragraph::new(Line::from(spans));
        frame.render_widget(input_line, area);
    }
}

impl UiComponent for FilterInputWidget {
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
