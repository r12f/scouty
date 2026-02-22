//! Filter input widget — f filter expression bar.

#[cfg(test)]
#[path = "filter_input_widget_tests.rs"]
mod filter_input_widget_tests;

use crate::app::App;
use crate::ui::{ComponentResult, UiComponent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

#[allow(dead_code)]
pub struct FilterInputWidget;

#[allow(dead_code)]
impl FilterInputWidget {
    pub fn render_with_app(&self, frame: &mut Frame, area: Rect, app: &App) {
        let mut spans = vec![
            Span::styled("Filter: ", Style::default().fg(Color::Yellow)),
            Span::raw(&app.filter_input),
            Span::styled("█", Style::default().fg(Color::White)),
        ];

        if let Some(ref err) = app.filter_error {
            spans.push(Span::styled(
                format!("  {}", err),
                Style::default().fg(Color::Red),
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
