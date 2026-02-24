//! Save dialog window (s key): path input + format selection.

#[cfg(test)]
#[path = "save_dialog_window_tests.rs"]
mod save_dialog_window_tests;

use crate::app::{App, ExportFormat};
use crate::config::Theme;
use crate::text_input::TextInput;
use crate::ui::{ComponentResult, UiComponent};
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

const FORMAT_OPTIONS: [(&str, ExportFormat); 3] = [
    ("Raw (one line per record)", ExportFormat::Raw),
    ("JSON", ExportFormat::Json),
    ("YAML", ExportFormat::Yaml),
];

/// Focus state within the save dialog.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    Path,
    Format,
}

/// Save dialog overlay for exporting logs to a file.
pub struct SaveDialogWindow {
    pub path_input: TextInput,
    pub format_cursor: usize,
    pub confirmed: bool,
    pub error: Option<String>,
    pub focus: Focus,
    theme: Theme,
}

impl SaveDialogWindow {
    pub fn from_app(app: &App) -> Self {
        Self {
            path_input: app.save_path_input.clone(),
            format_cursor: app.save_format_cursor,
            confirmed: false,
            error: None,
            focus: app.save_dialog_focus,
            theme: app.theme.clone(),
        }
    }

    /// Get selected export format.
    pub fn selected_format(&self) -> ExportFormat {
        FORMAT_OPTIONS[self.format_cursor].1
    }

    /// Get the path with ~ expansion.
    pub fn expanded_path(&self) -> String {
        let path = self.path_input.value().trim().to_string();
        if let Some(rest) = path.strip_prefix("~/") {
            if let Some(home) = dirs::home_dir() {
                return home.join(rest).to_string_lossy().to_string();
            }
        }
        path
    }

    /// Perform the export. Returns a status message.
    pub fn execute_save(app: &App, path: &str, format: ExportFormat) -> String {
        let count = app.filtered_indices.len();

        let content = match format {
            ExportFormat::Raw => {
                let mut lines = Vec::with_capacity(count);
                for &idx in &app.filtered_indices {
                    lines.push(app.records[idx].raw.as_str());
                }
                lines.join("\n") + "\n"
            }
            ExportFormat::Json => {
                let records: Vec<&scouty::record::LogRecord> = app
                    .filtered_indices
                    .iter()
                    .map(|&idx| app.records[idx].as_ref())
                    .collect();
                match serde_json::to_string_pretty(&records) {
                    Ok(json) => json + "\n",
                    Err(e) => return format!("Save failed: {}", e),
                }
            }
            ExportFormat::Yaml => {
                let records: Vec<&scouty::record::LogRecord> = app
                    .filtered_indices
                    .iter()
                    .map(|&idx| app.records[idx].as_ref())
                    .collect();
                match serde_yaml::to_string(&records) {
                    Ok(yaml) => yaml,
                    Err(e) => return format!("Save failed: {}", e),
                }
            }
        };

        let fmt_label = match format {
            ExportFormat::Raw => "raw",
            ExportFormat::Json => "json",
            ExportFormat::Yaml => "yaml",
        };

        match std::fs::write(path, content) {
            Ok(()) => format!("Saved {} records to {} ({})", count, path, fmt_label),
            Err(e) => format!("Save failed: {}", e),
        }
    }
}

impl UiComponent for SaveDialogWindow {
    fn enable_jk_navigation(&self) -> bool {
        self.focus == Focus::Format
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let t = &self.theme;
        let width = 50u16.min(area.width.saturating_sub(4));
        let height = 12u16.min(area.height.saturating_sub(4));
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        let overlay = Rect::new(x, y, width, height);

        frame.render_widget(Clear, overlay);

        let mut lines = vec![Line::from("")];

        // Path label + input with cursor rendering
        let path_style = if self.focus == Focus::Path {
            t.dialog.accent.to_style()
        } else {
            t.dialog.text.to_style()
        };

        let (before, cursor_ch, after) = self.path_input.render_parts();
        if self.focus == Focus::Path {
            lines.push(Line::from(vec![
                Span::styled("  Path: ", t.dialog.text.to_style()),
                Span::styled(before.to_string(), path_style),
                Span::styled(
                    cursor_ch,
                    t.dialog.accent.to_style().add_modifier(Modifier::REVERSED),
                ),
                Span::styled(after.to_string(), path_style),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("  Path: ", t.dialog.text.to_style()),
                Span::styled(self.path_input.value().to_string(), path_style),
            ]));
        }

        // Show error if any
        if let Some(ref err) = self.error {
            lines.push(Line::styled(
                format!("  {}", err),
                t.dialog.accent.to_style(),
            ));
        } else {
            lines.push(Line::from(""));
        }

        // Format label
        lines.push(Line::styled("  Format:", t.dialog.text.to_style()));

        // Format options
        for (i, (label, _)) in FORMAT_OPTIONS.iter().enumerate() {
            let is_selected = i == self.format_cursor;
            let marker = if is_selected { "▸ " } else { "  " };
            let style = if self.focus == Focus::Format && is_selected {
                t.dialog.accent.to_style().add_modifier(Modifier::BOLD)
            } else if is_selected {
                t.dialog.text.to_style().add_modifier(Modifier::BOLD)
            } else {
                t.dialog.text.to_style()
            };
            lines.push(Line::from(Span::styled(
                format!("    {}{}", marker, label),
                style,
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::styled(
            "  Enter: Save  Tab: Switch  Esc: Cancel",
            t.dialog.muted.to_style(),
        ));

        let dialog = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Save Logs (s) ")
                    .borders(Borders::ALL)
                    .border_style(t.dialog.border.to_style()),
            )
            .style(t.dialog.background.to_style());
        frame.render_widget(dialog, overlay);
    }

    fn on_up(&mut self) -> ComponentResult {
        if self.focus == Focus::Format && self.format_cursor > 0 {
            self.format_cursor -= 1;
        }
        ComponentResult::Consumed
    }

    fn on_down(&mut self) -> ComponentResult {
        if self.focus == Focus::Format && self.format_cursor < FORMAT_OPTIONS.len() - 1 {
            self.format_cursor += 1;
        } else if self.focus == Focus::Path {
            self.focus = Focus::Format;
        }
        ComponentResult::Consumed
    }

    fn on_confirm(&mut self) -> ComponentResult {
        let path = self.path_input.value().trim().to_string();
        if path.is_empty() {
            self.error = Some("Path required".to_string());
            return ComponentResult::Consumed;
        }
        self.confirmed = true;
        ComponentResult::Close
    }

    fn on_cancel(&mut self) -> ComponentResult {
        self.confirmed = false;
        ComponentResult::Close
    }

    fn on_char(&mut self, c: char) -> ComponentResult {
        if self.focus == Focus::Path {
            self.path_input.insert(c);
            self.error = None;
            ComponentResult::Consumed
        } else {
            ComponentResult::Ignored
        }
    }

    fn on_toggle(&mut self) -> ComponentResult {
        // Space in path input should insert a space character
        if self.focus == Focus::Path {
            self.path_input.insert(' ');
            self.error = None;
            ComponentResult::Consumed
        } else {
            ComponentResult::Ignored
        }
    }

    fn on_key(&mut self, key: crossterm::event::KeyEvent) -> ComponentResult {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Tab | KeyCode::BackTab => {
                self.focus = match self.focus {
                    Focus::Path => Focus::Format,
                    Focus::Format => Focus::Path,
                };
                ComponentResult::Consumed
            }
            _ if self.focus == Focus::Path => {
                if self.path_input.handle_key(key) {
                    self.error = None;
                    ComponentResult::Consumed
                } else {
                    ComponentResult::Ignored
                }
            }
            _ => ComponentResult::Ignored,
        }
    }
}
