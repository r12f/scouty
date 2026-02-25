//! Region manager overlay (r key) — lists detected regions with navigation.

#[cfg(test)]
#[path = "region_manager_window_tests.rs"]
mod region_manager_window_tests;

use crate::app::App;
use crate::ui::{ComponentResult, UiComponent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub enum RegionAction {
    Jump(usize),
    Filter(usize, usize),
}

pub struct RegionManagerWindow {
    pub cursor: usize,
    pub entries: Vec<RegionEntry>,
    pub action: Option<RegionAction>,
}

pub struct RegionEntry {
    pub name: String,
    pub definition_name: String,
    pub time_range: String,
    pub start_index: usize,
    pub end_index: usize,
}

/// Highlight palette colors for region types.
const REGION_COLORS: &[Color] = &[
    Color::Cyan,
    Color::Yellow,
    Color::Green,
    Color::Magenta,
    Color::Blue,
    Color::Red,
];

impl RegionManagerWindow {
    pub fn from_app(app: &App) -> Self {
        let entries: Vec<RegionEntry> = app
            .regions
            .regions()
            .iter()
            .map(|region| {
                let start_ts = app
                    .records
                    .get(region.start_index)
                    .map(|r| r.timestamp.format("%H:%M:%S").to_string())
                    .unwrap_or_else(|| "?".to_string());
                let end_ts = app
                    .records
                    .get(region.end_index)
                    .map(|r| r.timestamp.format("%H:%M:%S").to_string())
                    .unwrap_or_else(|| "?".to_string());

                RegionEntry {
                    name: region.name.clone(),
                    definition_name: region.definition_name.clone(),
                    time_range: format!("{} → {}", start_ts, end_ts),
                    start_index: region.start_index,
                    end_index: region.end_index,
                }
            })
            .collect();
        let cursor = app
            .region_manager_cursor
            .min(entries.len().saturating_sub(1));
        Self {
            cursor,
            entries,
            action: None,
        }
    }

    pub fn render_with_app(&self, frame: &mut Frame, area: Rect, _app: &App) {
        // Centered popup: 80% width, 60% height
        let popup_width = (area.width as f32 * 0.8) as u16;
        let popup_height = (area.height as f32 * 0.6).max(8.0) as u16;
        let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
        let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
        let popup_area = Rect::new(x, y, popup_width, popup_height);

        frame.render_widget(Clear, popup_area);

        // Collect unique definition names for type count
        let mut type_names: Vec<String> = self
            .entries
            .iter()
            .map(|e| e.definition_name.clone())
            .collect();
        type_names.sort();
        type_names.dedup();
        let type_count = type_names.len();

        let inner = Block::default()
            .borders(Borders::ALL)
            .title(format!(
                " Regions — {} total ({} types) ",
                self.entries.len(),
                type_count
            ))
            .style(Style::default().fg(Color::White));

        let inner_area = inner.inner(popup_area);
        frame.render_widget(inner, popup_area);

        if self.entries.is_empty() {
            let msg = Paragraph::new("No regions detected.");
            frame.render_widget(msg, inner_area);
            return;
        }

        // Build type → color map
        let type_color_map: std::collections::HashMap<&str, Color> = type_names
            .iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), REGION_COLORS[i % REGION_COLORS.len()]))
            .collect();

        // Scrollable region list (reserve 2 lines for footer)
        let list_height = inner_area.height.saturating_sub(2) as usize;
        let scroll_offset = if self.cursor >= list_height {
            self.cursor - list_height + 1
        } else {
            0
        };

        let mut lines: Vec<Line> = Vec::new();
        for (i, entry) in self.entries.iter().enumerate().skip(scroll_offset) {
            if lines.len() >= list_height {
                break;
            }
            let color = type_color_map
                .get(entry.definition_name.as_str())
                .copied()
                .unwrap_or(Color::White);
            let prefix = if i == self.cursor { "▶ " } else { "  " };
            let style = if i == self.cursor {
                Style::default()
                    .fg(color)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default().fg(color)
            };

            // Truncate name to fit
            let max_name = (inner_area.width as usize).saturating_sub(25);
            let name: String = entry.name.chars().take(max_name).collect();
            let line_str = format!(
                "{}{:<width$}  {}",
                prefix,
                name,
                entry.time_range,
                width = max_name
            );
            lines.push(Line::styled(line_str, style));
        }

        // Footer
        let footer_y = inner_area.y + inner_area.height.saturating_sub(1);
        let footer = Paragraph::new(Line::styled(
            " [Enter] Jump  [f] Filter  [Esc] Close",
            Style::default().fg(Color::DarkGray),
        ));
        let footer_area = Rect::new(inner_area.x, footer_y, inner_area.width, 1);
        frame.render_widget(footer, footer_area);

        let list_area = Rect::new(
            inner_area.x,
            inner_area.y,
            inner_area.width,
            list_height as u16,
        );
        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, list_area);
    }
}

impl UiComponent for RegionManagerWindow {
    fn render(&self, _frame: &mut Frame, _area: Rect) {
        // Use render_with_app instead
    }

    fn on_up(&mut self) -> ComponentResult {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
        ComponentResult::Consumed
    }

    fn on_down(&mut self) -> ComponentResult {
        if self.cursor + 1 < self.entries.len() {
            self.cursor += 1;
        }
        ComponentResult::Consumed
    }

    fn on_confirm(&mut self) -> ComponentResult {
        if let Some(entry) = self.entries.get(self.cursor) {
            self.action = Some(RegionAction::Jump(entry.start_index));
            ComponentResult::Close
        } else {
            ComponentResult::Consumed
        }
    }

    fn on_char(&mut self, c: char) -> ComponentResult {
        match c {
            'f' | 'F' => {
                if let Some(entry) = self.entries.get(self.cursor) {
                    self.action = Some(RegionAction::Filter(entry.start_index, entry.end_index));
                    ComponentResult::Close
                } else {
                    ComponentResult::Consumed
                }
            }
            _ => ComponentResult::Ignored,
        }
    }
}
