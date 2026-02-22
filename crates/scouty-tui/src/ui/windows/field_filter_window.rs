//! Field filter dialog (_/+).

#[cfg(test)]
#[path = "field_filter_window_tests.rs"]
mod field_filter_window_tests;

use crate::app::{App, FieldEntry, FieldEntryKind};
use crate::ui::{ComponentResult, UiComponent};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

/// Field filter overlay — lets user select fields to include/exclude.
#[allow(dead_code)]
pub struct FieldFilterWindow {
    pub fields: Vec<FieldEntry>,
    pub cursor: usize,
    pub exclude: bool,
    pub logic_or: bool,
    pub confirmed: bool,
}

#[allow(dead_code)]
impl FieldFilterWindow {
    pub fn from_app(app: &App) -> Option<Self> {
        let ff = app.field_filter.as_ref()?;
        Some(Self {
            fields: ff.fields.clone(),
            cursor: ff.cursor,
            exclude: ff.exclude,
            logic_or: ff.logic_or,
            confirmed: false,
        })
    }

    pub fn sync_to_app(&self, app: &mut App) {
        if let Some(ref mut ff) = app.field_filter {
            ff.fields = self.fields.clone();
            ff.cursor = self.cursor;
            ff.exclude = self.exclude;
            ff.logic_or = self.logic_or;
        }
    }
}

#[allow(dead_code)]
impl UiComponent for FieldFilterWindow {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let width = 60u16.min(area.width.saturating_sub(4));
        let height = (self.fields.len() as u16 + 8).min(area.height.saturating_sub(4));
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        let overlay = Rect::new(x, y, width, height);

        frame.render_widget(Clear, overlay);

        let action = if self.exclude { "Exclude" } else { "Include" };
        let logic = if self.logic_or { "OR" } else { "AND" };
        let mut lines = vec![
            Line::from(vec![
                Span::raw(" Action: "),
                Span::styled(
                    format!("[{}]", action),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" (Tab)", Style::default().fg(Color::DarkGray)),
                Span::raw("  Logic: "),
                Span::styled(
                    format!("[{}]", logic),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" (o)", Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(""),
        ];

        let max_visible = (height as usize).saturating_sub(6);
        let scroll_offset = if self.cursor >= max_visible {
            self.cursor - max_visible + 1
        } else {
            0
        };
        let visible_end = (scroll_offset + max_visible).min(self.fields.len());

        for i in scroll_offset..visible_end {
            let entry = &self.fields[i];
            let checkbox = if entry.checked { "[x]" } else { "[ ]" };
            let is_cursor = i == self.cursor;
            let style = if is_cursor {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            let display = match &entry.kind {
                FieldEntryKind::TimeBefore { .. } | FieldEntryKind::TimeAfter { .. } => {
                    // Time entries show just the name (e.g. "Before 2025-01-01 12:00:00.000")
                    format!(" {} ⏱ {}", checkbox, entry.name)
                }
                FieldEntryKind::Field => {
                    let max_val = (width as usize).saturating_sub(22);
                    let display_val = if entry.value.len() > max_val {
                        format!("{}…", &entry.value[..max_val.saturating_sub(1)])
                    } else {
                        entry.value.clone()
                    };
                    format!(" {} {:<14} = {}", checkbox, entry.name, display_val)
                }
            };
            lines.push(Line::styled(display, style));
        }

        if self.fields.len() > max_visible {
            lines.push(Line::styled(
                format!(" ({}/{})", self.cursor + 1, self.fields.len()),
                Style::default().fg(Color::DarkGray),
            ));
        }

        lines.push(Line::from(""));
        lines.push(Line::styled(
            " Enter: Apply  Esc: Cancel  Space: Toggle",
            Style::default().fg(Color::DarkGray),
        ));

        let title = if self.exclude {
            " Exclude Fields (_) "
        } else {
            " Include Fields (+) "
        };

        let dialog = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .style(Style::default().bg(Color::Black));
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
        if self.cursor + 1 < self.fields.len() {
            self.cursor += 1;
        }
        ComponentResult::Consumed
    }

    fn on_page_up(&mut self) -> ComponentResult {
        self.cursor = self.cursor.saturating_sub(10);
        ComponentResult::Consumed
    }

    fn on_page_down(&mut self) -> ComponentResult {
        self.cursor = (self.cursor + 10).min(self.fields.len().saturating_sub(1));
        ComponentResult::Consumed
    }

    fn on_toggle(&mut self) -> ComponentResult {
        let cur = self.cursor;
        if cur < self.fields.len() {
            self.fields[cur].checked = !self.fields[cur].checked;
        }
        ComponentResult::Consumed
    }

    fn on_confirm(&mut self) -> ComponentResult {
        self.confirmed = true;
        ComponentResult::Close
    }

    fn on_cancel(&mut self) -> ComponentResult {
        ComponentResult::Close
    }

    fn on_char(&mut self, c: char) -> ComponentResult {
        match c {
            'o' => {
                self.logic_or = !self.logic_or;
                ComponentResult::Consumed
            }
            _ => ComponentResult::Ignored,
        }
    }

    fn on_key(&mut self, key: KeyEvent) -> ComponentResult {
        match key.code {
            KeyCode::Tab => {
                self.exclude = !self.exclude;
                ComponentResult::Consumed
            }
            _ => ComponentResult::Ignored,
        }
    }
}
