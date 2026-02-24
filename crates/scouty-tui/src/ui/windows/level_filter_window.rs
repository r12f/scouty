//! Level filter overlay — quick log level selection.

#[cfg(test)]
#[path = "level_filter_window_tests.rs"]
mod level_filter_window_tests;

use crate::app::LevelFilterPreset;
use crate::config::Theme;
use crate::ui::{ComponentResult, UiComponent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

const OPTIONS: [(u8, LevelFilterPreset); 5] = [
    (1, LevelFilterPreset::All),
    (2, LevelFilterPreset::DebugPlus),
    (3, LevelFilterPreset::InfoPlus),
    (4, LevelFilterPreset::WarnPlus),
    (5, LevelFilterPreset::ErrorPlus),
];

pub struct LevelFilterWindow<'a> {
    pub cursor: usize,
    pub selected: Option<LevelFilterPreset>,
    pub confirmed: bool,
    pub current_level: Option<LevelFilterPreset>,
    pub theme: &'a Theme,
}

impl<'a> LevelFilterWindow<'a> {
    pub fn new(current: Option<LevelFilterPreset>, theme: &'a Theme) -> Self {
        let cursor = current.map(|l| (l.as_number() - 1) as usize).unwrap_or(0);
        Self {
            cursor,
            selected: None,
            confirmed: false,
            current_level: current,
            theme,
        }
    }

    pub fn from_app(app: &'a crate::app::App) -> Self {
        Self::new(app.level_filter, &app.theme)
    }
}

impl<'a> UiComponent for LevelFilterWindow<'a> {
    fn enable_jk_navigation(&self) -> bool {
        true
    }

    fn on_up(&mut self) -> ComponentResult {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
        ComponentResult::Consumed
    }

    fn on_down(&mut self) -> ComponentResult {
        if self.cursor < OPTIONS.len() - 1 {
            self.cursor += 1;
        }
        ComponentResult::Consumed
    }

    fn on_confirm(&mut self) -> ComponentResult {
        self.selected = Some(OPTIONS[self.cursor].1);
        self.confirmed = true;
        ComponentResult::Close
    }

    fn on_cancel(&mut self) -> ComponentResult {
        ComponentResult::Close
    }

    fn on_char(&mut self, c: char) -> ComponentResult {
        if let Some(n) = c.to_digit(10) {
            if (1..=5).contains(&n) {
                if let Some(preset) = LevelFilterPreset::from_number(n as u8) {
                    self.selected = Some(preset);
                    self.confirmed = true;
                    return ComponentResult::Close;
                }
            }
        }
        ComponentResult::Consumed
    }

    fn render(&self, frame: &mut Frame, _area: Rect) {
        let area = frame.area();
        let width = 24u16;
        let height = (OPTIONS.len() as u16) + 2;
        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;
        let overlay = Rect::new(x, y, width.min(area.width), height.min(area.height));

        frame.render_widget(Clear, overlay);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.dialog.border.to_style())
            .title(" Level Filter ")
            .title_style(self.theme.dialog.title.to_style());

        let inner = block.inner(overlay);
        frame.render_widget(block, overlay);

        for (i, (num, preset)) in OPTIONS.iter().enumerate() {
            if i as u16 >= inner.height {
                break;
            }
            let is_current = self
                .current_level
                .map_or(*preset == LevelFilterPreset::All, |l| l == *preset);
            let is_selected = i == self.cursor;

            let marker = if is_current { "●" } else { " " };
            let line = format!(" {} {}  {}", num, preset.label(), marker);

            let style = if is_selected {
                self.theme.dialog.selected.to_style()
            } else {
                self.theme.dialog.text.to_style()
            };

            let row_area = Rect::new(inner.x, inner.y + i as u16, inner.width, 1);
            frame.render_widget(Paragraph::new(line).style(style), row_area);
        }
    }
}
