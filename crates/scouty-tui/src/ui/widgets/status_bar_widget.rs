//! Status bar widget — 2-line bottom bar.
//!
//! Line 1: density chart (left) + position info (right)
//! Line 2: mode label + shortcut hints

#[cfg(test)]
#[path = "status_bar_widget_tests.rs"]
mod status_bar_widget_tests;

use crate::app::App;
use crate::ui::UiComponent;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

/// Persistent status bar at the bottom of the screen.
pub struct StatusBarWidget;

impl StatusBarWidget {
    /// Render line 1: density chart + position info.
    pub fn render_line1(&self, frame: &mut Frame, area: Rect, app: &App) {
        let position = if app.total() == 0 {
            format!("0/0 (Total: {})", app.total_records)
        } else {
            let current = app.selected + 1;
            let filtered = app.total();
            let total = app.total_records;
            if filtered == total {
                format!("{}/{}", current, total)
            } else {
                format!("{}/{} (Total: {})", current, filtered, total)
            }
        };

        let right_text = format!(" {} ", position);
        let right_width = right_text.len() as u16;

        let chart_width = area.width.saturating_sub(right_width + 2) as usize;

        let mut spans: Vec<Span> = Vec::new();

        if chart_width >= 4 && app.total() > 0 {
            if let Some(cache) = &app.density_cache {
                let cursor_char_idx = app.cursor_char_in_density();

                for (i, ch) in cache.braille_text.chars().enumerate() {
                    let style = if Some(i) == cursor_char_idx {
                        Style::default()
                            .fg(Color::Yellow)
                            .bg(Color::Rgb(40, 40, 60))
                    } else {
                        Style::default().fg(Color::Cyan)
                    };
                    spans.push(Span::styled(ch.to_string(), style));
                }
            }

            spans.push(Span::styled(" │", Style::default().fg(Color::DarkGray)));
        }

        spans.push(Span::styled(
            right_text,
            Style::default().fg(Color::White).bg(Color::DarkGray),
        ));

        let footer = Paragraph::new(Line::from(spans));
        frame.render_widget(footer, area);
    }

    /// Render line 2: mode label + shortcut hints or status message.
    pub fn render_line2(&self, frame: &mut Frame, area: Rect, app: &App) {
        let (mode_label, mode_color) = if app.follow_mode {
            ("[FOLLOW]", Color::Green)
        } else {
            ("[VIEW]", Color::Cyan)
        };

        let mut spans = vec![Span::styled(
            format!(" {} ", mode_label),
            Style::default().fg(Color::Black).bg(mode_color),
        )];

        // Show status message if present, otherwise show shortcut hints
        if let Some(ref msg) = app.status_message {
            spans.push(Span::styled(
                format!(" {} ", msg),
                Style::default().fg(Color::Yellow),
            ));
        } else {
            let shortcuts = [
                ("/", "Search"),
                ("f", "Filter"),
                ("-", "Exclude"),
                ("=", "Include"),
                ("_", "ExclField"),
                ("+", "InclField"),
                ("Enter", "Detail"),
                ("c", "Columns"),
                ("?", "Help"),
            ];

            let mode_width = mode_label.len() + 2; // + spaces for " {} "
            let mut remaining = area.width.saturating_sub(mode_width as u16) as usize;

            for (i, (key, desc)) in shortcuts.iter().enumerate() {
                let entry = if i == 0 {
                    format!(" {}: {}", key, desc)
                } else {
                    format!(" │ {}: {}", key, desc)
                };
                if entry.len() > remaining {
                    break;
                }
                remaining -= entry.len();
                spans.push(Span::styled(entry, Style::default().fg(Color::DarkGray)));
            }
        }

        let footer = Paragraph::new(Line::from(spans));
        frame.render_widget(footer, area);
    }
}

impl UiComponent for StatusBarWidget {
    fn render(&self, _frame: &mut Frame, _area: Rect) {
        // No-op: use render_with_app() for app-aware rendering.
    }

    fn enable_jk_navigation(&self) -> bool {
        false
    }
}
