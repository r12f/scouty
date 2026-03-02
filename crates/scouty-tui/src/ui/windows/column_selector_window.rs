//! Column selector dialog (c key).

#[cfg(test)]
#[path = "column_selector_window_tests.rs"]
mod column_selector_window_tests;

use crate::app::{App, Column};
use crate::config::Theme;
use crate::ui::{ComponentResult, UiComponent};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

/// Column selector overlay.
#[allow(dead_code)]
pub struct ColumnSelectorWindow {
    pub cursor: usize,
    pub columns: Vec<(Column, bool)>,
    pub width_overrides: Vec<Option<u16>>,
    pub col_widths: [u16; 8],
    pub theme: Theme,
}

#[allow(dead_code)]
impl ColumnSelectorWindow {
    pub fn from_app(app: &App) -> Self {
        let cw = &app.col_widths;
        let mut auto_widths = std::collections::HashMap::new();
        auto_widths.insert(Column::Time, cw[0]);
        auto_widths.insert(Column::Level, cw[1]);
        auto_widths.insert(Column::ProcessName, cw[2]);
        auto_widths.insert(Column::Pid, cw[3]);
        auto_widths.insert(Column::Tid, cw[4]);
        auto_widths.insert(Column::Component, cw[5]);
        auto_widths.insert(Column::Context, cw[6]);
        auto_widths.insert(Column::Function, cw[7]);
        auto_widths.insert(Column::Hostname, 20);
        auto_widths.insert(Column::Container, 15);
        auto_widths.insert(Column::Source, 15);

        Self {
            cursor: app.column_config.cursor,
            columns: app.column_config.columns.clone(),
            width_overrides: app.column_config.width_overrides.clone(),
            col_widths: app.col_widths,
            theme: app.theme.clone(),
        }
    }

    pub fn sync_to_app(&self, app: &mut App) {
        app.column_config.cursor = self.cursor;
        app.column_config.columns = self.columns.clone();
        app.column_config.width_overrides = self.width_overrides.clone();
    }

    fn toggle_current(&mut self) {
        let cur = self.cursor;
        if cur < self.columns.len() && self.columns[cur].0 != Column::Log {
            self.columns[cur].1 = !self.columns[cur].1;
        }
    }

    fn auto_width_for(&self, index: usize) -> u16 {
        if index >= self.columns.len() {
            return 0;
        }
        let col = &self.columns[index].0;
        if let Some(cw_idx) = col.col_widths_index() {
            self.col_widths[cw_idx]
        } else {
            col.default_fixed_width()
        }
    }

    fn effective_width(&self, index: usize) -> u16 {
        let auto = self.auto_width_for(index);
        if index < self.width_overrides.len() {
            self.width_overrides[index].unwrap_or(auto)
        } else {
            auto
        }
    }

    fn adjust_width(&mut self, delta: i16) {
        let cur = self.cursor;
        if cur >= self.columns.len() {
            return;
        }
        let (col, visible) = &self.columns[cur];
        if *col == Column::Log || !visible {
            return;
        }
        let current = self.effective_width(cur);
        let min = col.min_width();
        let new_width = ((current as i32) + (delta as i32))
            .max(min as i32)
            .min(u16::MAX as i32) as u16;
        if new_width != current {
            self.width_overrides[cur] = Some(new_width);
        }
    }

    fn reset_width(&mut self) {
        let cur = self.cursor;
        if cur < self.width_overrides.len() {
            self.width_overrides[cur] = None;
        }
    }

    /// Format the width display for a column.
    fn width_display(&self, index: usize) -> String {
        if index >= self.columns.len() {
            return String::new();
        }
        let (col, visible) = &self.columns[index];
        if *col == Column::Log {
            "fill".to_string()
        } else if !visible {
            "-".to_string()
        } else {
            format!("{}", self.effective_width(index))
        }
    }
}

#[allow(dead_code)]
impl UiComponent for ColumnSelectorWindow {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let t = &self.theme;
        let width = 40u16.min(area.width.saturating_sub(4));
        let height = (self.columns.len() as u16 + 6).min(area.height.saturating_sub(4));
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        let overlay = Rect::new(x, y, width, height);

        frame.render_widget(Clear, overlay);

        let mut lines = vec![
            Line::styled(" Toggle (Space) / Width (h/l)", t.dialog.muted.to_style()),
            Line::from(""),
        ];

        for (i, (col, visible)) in self.columns.iter().enumerate() {
            let checkbox = if *visible { "[x]" } else { "[ ]" };
            let is_cursor = i == self.cursor;
            let width_str = self.width_display(i);
            let style = if is_cursor {
                t.dialog.selected.to_style()
            } else {
                Style::default()
            };
            lines.push(Line::styled(
                format!(" {} {:<12} {:>4}", checkbox, col.label(), width_str),
                style,
            ));
        }

        lines.push(Line::from(""));
        lines.push(Line::styled(
            " r: Reset width  Esc: Close",
            t.dialog.muted.to_style(),
        ));

        let dialog = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Columns (c) ")
                    .borders(Borders::ALL)
                    .border_style(t.dialog.border.to_style()),
            )
            .style(t.dialog.background.to_style());
        frame.render_widget(dialog, overlay);
    }

    fn enable_jk_navigation(&self) -> bool {
        true
    }

    fn on_up(&mut self) -> ComponentResult {
        self.cursor = self.cursor.saturating_sub(1);
        ComponentResult::Consumed
    }

    fn on_down(&mut self) -> ComponentResult {
        if self.cursor + 1 < self.columns.len() {
            self.cursor += 1;
        }
        ComponentResult::Consumed
    }

    fn on_toggle(&mut self) -> ComponentResult {
        self.toggle_current();
        ComponentResult::Consumed
    }

    fn on_confirm(&mut self) -> ComponentResult {
        self.toggle_current();
        ComponentResult::Consumed
    }

    fn on_cancel(&mut self) -> ComponentResult {
        ComponentResult::Close
    }

    fn on_char(&mut self, c: char) -> ComponentResult {
        match c {
            'h' => {
                self.adjust_width(-1);
                ComponentResult::Consumed
            }
            'l' => {
                self.adjust_width(1);
                ComponentResult::Consumed
            }
            'r' => {
                self.reset_width();
                ComponentResult::Consumed
            }
            _ => ComponentResult::Ignored,
        }
    }

    fn on_key(&mut self, key: KeyEvent) -> ComponentResult {
        match key.code {
            KeyCode::Left => {
                self.adjust_width(-1);
                ComponentResult::Consumed
            }
            KeyCode::Right => {
                self.adjust_width(1);
                ComponentResult::Consumed
            }
            _ => ComponentResult::Ignored,
        }
    }
}
