//! Status bar widget — bottom bar with density chart and position info.

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
    /// Render the status bar with app context.
    pub fn render_with_app(&self, frame: &mut Frame, area: Rect, app: &App) {
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

        let follow_indicator = if app.follow_mode { " [FOLLOW]" } else { "" };
        let mut right_text = format!(" {}{} ", position, follow_indicator);
        if let Some(ref msg) = app.status_message {
            right_text = format!(" {} │{}", msg, right_text);
        }
        let right_width = right_text.len() as u16 + 1;

        let chart_width = area.width.saturating_sub(right_width + 3) as usize;

        let mut spans: Vec<Span> = Vec::new();

        if chart_width >= 4 && app.total() > 0 {
            let timestamps: Vec<chrono::DateTime<chrono::Utc>> = app
                .filtered_indices
                .iter()
                .map(|&i| app.records[i].timestamp)
                .collect();

            let num_buckets = (chart_width * 2).min(200);
            let buckets = crate::density::compute_density(&timestamps, num_buckets);

            let cursor_ts = app.selected_record().map(|r| r.timestamp);
            let cursor_bucket = cursor_ts
                .and_then(|ts| crate::density::cursor_bucket(ts, &timestamps, num_buckets));

            let (braille_text, cursor_char_idx) =
                crate::density::render_braille(&buckets, cursor_bucket);

            for (i, ch) in braille_text.chars().enumerate() {
                let style = if Some(i) == cursor_char_idx {
                    Style::default()
                        .fg(Color::Yellow)
                        .bg(Color::Rgb(40, 40, 60))
                } else {
                    Style::default().fg(Color::Cyan)
                };
                spans.push(Span::styled(ch.to_string(), style));
            }

            spans.push(Span::styled(" │", Style::default().fg(Color::DarkGray)));
        }

        if let Some(ref msg) = app.status_message {
            spans.push(Span::styled(
                format!(" {} │", msg),
                Style::default().fg(Color::Yellow),
            ));
        }
        spans.push(Span::styled(
            format!(" {}{} ", position, follow_indicator),
            Style::default().fg(Color::White).bg(Color::DarkGray),
        ));

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
