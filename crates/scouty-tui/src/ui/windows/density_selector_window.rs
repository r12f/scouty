//! Density source selector overlay (D key).

#[cfg(test)]
#[path = "density_selector_window_tests.rs"]
mod density_selector_window_tests;

use crate::app::DensitySource;
use crate::ui::{ComponentResult, UiComponent};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub struct DensitySelectorWindow {
    pub options: Vec<DensitySource>,
    pub cursor: usize,
    pub selected: Option<DensitySource>,
    pub confirmed: bool,
}

impl DensitySelectorWindow {
    pub fn new(options: Vec<DensitySource>, cursor: usize) -> Self {
        Self {
            cursor: cursor.min(options.len().saturating_sub(1)),
            options,
            selected: None,
            confirmed: false,
        }
    }

    fn option_label(source: &DensitySource) -> String {
        match source {
            DensitySource::All => "All records".to_string(),
            DensitySource::Level(l) => format!("{} only", l),
            DensitySource::Highlight(p) => format!("Highlight: \"{}\"", p),
        }
    }
}

impl UiComponent for DensitySelectorWindow {
    fn enable_jk_navigation(&self) -> bool {
        true
    }

    fn on_up(&mut self) -> ComponentResult {
        self.cursor = self.cursor.saturating_sub(1);
        ComponentResult::Consumed
    }

    fn on_down(&mut self) -> ComponentResult {
        if !self.options.is_empty() && self.cursor + 1 < self.options.len() {
            self.cursor += 1;
        }
        ComponentResult::Consumed
    }

    fn on_confirm(&mut self) -> ComponentResult {
        if !self.options.is_empty() {
            self.selected = Some(self.options[self.cursor].clone());
            self.confirmed = true;
        }
        ComponentResult::Close
    }

    fn on_cancel(&mut self) -> ComponentResult {
        ComponentResult::Close
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let width = 40u16.min(area.width.saturating_sub(4));
        let height = (self.options.len() as u16 + 5)
            .min(area.height.saturating_sub(4))
            .max(6);
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        let overlay = Rect::new(x, y, width, height);

        frame.render_widget(Clear, overlay);

        let block = Block::default()
            .title(" Density Source (D) ")
            .borders(Borders::ALL);
        let inner = block.inner(overlay);
        frame.render_widget(block, overlay);

        let mut lines = Vec::new();
        for (i, opt) in self.options.iter().enumerate() {
            let style = if i == self.cursor {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            lines.push(Line::styled(format!(" {}", Self::option_label(opt)), style));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(" Enter: Select  Esc: Cancel"));

        let content = Paragraph::new(lines);
        frame.render_widget(content, inner);
    }
}
