//! Bookmark manager overlay (M key).

#[cfg(test)]
#[path = "bookmark_manager_window_tests.rs"]
mod bookmark_manager_window_tests;

use crate::app::App;
use crate::ui::{ComponentResult, UiComponent};
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

/// An entry in the bookmark manager list.
pub struct BookmarkEntry {
    /// Filtered index (row position in the visible list).
    pub filtered_idx: usize,
    /// Original record ID.
    pub record_id: u64,
    /// Message preview.
    pub message: String,
}

pub enum BookmarkAction {
    Jump(usize),
}

/// Bookmark manager overlay — shows bookmarked lines, allows jump/delete.
pub struct BookmarkManagerWindow {
    pub cursor: usize,
    pub entries: Vec<BookmarkEntry>,
    pub action: Option<BookmarkAction>,
    /// Record IDs to delete (accumulated during interaction).
    pub deleted_ids: Vec<u64>,
}

impl BookmarkManagerWindow {
    pub fn from_app(app: &App) -> Self {
        let entries: Vec<BookmarkEntry> = app
            .bookmarked_filtered_indices()
            .into_iter()
            .map(|fi| {
                let ri = app.filtered_indices[fi];
                BookmarkEntry {
                    filtered_idx: fi,
                    record_id: app.records[ri].id,
                    message: app.records[ri].message.chars().take(50).collect(),
                }
            })
            .collect();
        let cursor = app
            .bookmark_manager_cursor
            .min(entries.len().saturating_sub(1));
        Self {
            cursor,
            entries,
            action: None,
            deleted_ids: Vec::new(),
        }
    }

    pub fn render_with_app(&self, frame: &mut Frame, app: &App, area: Rect) {
        let theme = &app.theme;
        let width = 60u16.min(area.width.saturating_sub(4));
        let height = (self.entries.len() as u16 + 7)
            .min(area.height.saturating_sub(4))
            .max(8);
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        let overlay = Rect::new(x, y, width, height);

        frame.render_widget(Clear, overlay);

        let mut lines = vec![
            Line::styled(
                format!(" Bookmarks ({})", self.entries.len()),
                theme.dialog.text.to_style().add_modifier(Modifier::BOLD),
            ),
            Line::from(""),
        ];

        if self.entries.is_empty() {
            lines.push(Line::styled(
                " (no bookmarks — press m to add)",
                theme.dialog.muted.to_style(),
            ));
        } else {
            for (i, entry) in self.entries.iter().enumerate() {
                let is_cursor = i == self.cursor;
                let prefix = if is_cursor { "▸ " } else { "  " };
                let style = if is_cursor {
                    theme.dialog.accent.to_style().add_modifier(Modifier::BOLD)
                } else {
                    theme.dialog.text.to_style()
                };
                lines.push(Line::styled(
                    format!("{}{:>6}  {}", prefix, entry.filtered_idx + 1, entry.message),
                    style,
                ));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::styled(
            " j/k:navigate  Enter:jump  d:delete  Esc:close",
            theme.dialog.muted.to_style(),
        ));

        let widget = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Bookmark Manager ")
                    .borders(Borders::ALL)
                    .border_style(theme.dialog.accent.to_style()),
            )
            .style(theme.dialog.background.to_style());
        frame.render_widget(widget, overlay);
    }

    pub fn apply_to_app(&self, app: &mut App) {
        app.bookmark_manager_cursor = self.cursor;
        // Remove deleted bookmarks
        for id in &self.deleted_ids {
            app.bookmarks.remove(id);
        }
        if !self.deleted_ids.is_empty() {
            app.set_status(format!("Bookmarks: {}", app.bookmarks.len()));
        }
        // Handle jump
        if let Some(BookmarkAction::Jump(fi)) = &self.action {
            app.selected = *fi;
            app.ensure_selected_visible();
        }
    }
}

impl UiComponent for BookmarkManagerWindow {
    fn render(&self, _frame: &mut Frame, _area: Rect) {
        // Use render_with_app instead
    }

    fn on_cancel(&mut self) -> ComponentResult {
        ComponentResult::Close
    }

    fn on_confirm(&mut self) -> ComponentResult {
        if !self.entries.is_empty() && self.cursor < self.entries.len() {
            self.action = Some(BookmarkAction::Jump(self.entries[self.cursor].filtered_idx));
            ComponentResult::Close
        } else {
            ComponentResult::Consumed
        }
    }

    fn on_up(&mut self) -> ComponentResult {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
        ComponentResult::Consumed
    }

    fn on_down(&mut self) -> ComponentResult {
        if !self.entries.is_empty() && self.cursor < self.entries.len() - 1 {
            self.cursor += 1;
        }
        ComponentResult::Consumed
    }

    fn enable_jk_navigation(&self) -> bool {
        true
    }

    fn on_char(&mut self, c: char) -> ComponentResult {
        match c {
            'd' => {
                if !self.entries.is_empty() && self.cursor < self.entries.len() {
                    let entry = self.entries.remove(self.cursor);
                    self.deleted_ids.push(entry.record_id);
                    if self.cursor > 0 && self.cursor >= self.entries.len() {
                        self.cursor = self.entries.len().saturating_sub(1);
                    }
                }
                ComponentResult::Consumed
            }
            'q' => ComponentResult::Close,
            _ => ComponentResult::Consumed,
        }
    }
}
