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
use unicode_width::UnicodeWidthStr;

/// Persistent status bar at the bottom of the screen.
pub struct StatusBarWidget;

/// Compute the display-column width of all spans combined.
fn spans_display_width(spans: &[Span]) -> usize {
    spans
        .iter()
        .map(|s| UnicodeWidthStr::width(s.content.as_ref()))
        .sum()
}

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
        }

        // Pad to push position info to the right edge
        // Use unicode_width for correct column count (braille chars are 3 bytes but 1 column)
        let used_width = spans_display_width(&spans);
        let total_width = area.width as usize;
        let right_len = right_text.len();
        if used_width + right_len < total_width {
            let pad = total_width - used_width - right_len;
            spans.push(Span::styled(
                " ".repeat(pad),
                Style::default().bg(Color::Rgb(20, 20, 40)),
            ));
        }

        spans.push(Span::styled(
            right_text,
            Style::default().fg(Color::White).bg(Color::DarkGray),
        ));

        let footer =
            Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Rgb(20, 20, 40)));
        frame.render_widget(footer, area);
    }

    /// Render line 2: mode label + shortcut hints or status message.
    pub fn render_line2(&self, frame: &mut Frame, area: Rect, app: &App) {
        let mut spans: Vec<Span> = Vec::new();

        // Command mode: show command input line
        if app.input_mode == crate::app::InputMode::Command {
            spans.push(Span::styled(
                " [CMD] ",
                Style::default().fg(Color::Black).bg(Color::Magenta),
            ));
            spans.push(Span::styled(
                format!(" :{}█", app.command_input),
                Style::default().fg(Color::White),
            ));
        } else if app.input_mode == crate::app::InputMode::JumpForward
            || app.input_mode == crate::app::InputMode::JumpBackward
        {
            // JumpForward / JumpBackward mode: show input line
            let label = if app.input_mode == crate::app::InputMode::JumpForward {
                "[JUMP+]"
            } else {
                "[JUMP-]"
            };
            spans.push(Span::styled(
                format!(" {} ", label),
                Style::default().fg(Color::Black).bg(Color::Magenta),
            ));
            spans.push(Span::styled(
                format!(" {}█", app.time_input),
                Style::default().fg(Color::White),
            ));
        } else {
            // Normal VIEW/FOLLOW mode
            let (mode_label, mode_color) = if app.follow_mode {
                ("[FOLLOW]", Color::Green)
            } else {
                ("[VIEW]", Color::Cyan)
            };

            spans.push(Span::styled(
                format!(" {} ", mode_label),
                Style::default().fg(Color::Black).bg(mode_color),
            ));

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

                let used = spans_display_width(&spans);
                let mut remaining = (area.width as usize).saturating_sub(used);

                for (i, (key, desc)) in shortcuts.iter().enumerate() {
                    let entry = if i == 0 {
                        format!(" {}: {}", key, desc)
                    } else {
                        format!(" │ {}: {}", key, desc)
                    };
                    let entry_width = UnicodeWidthStr::width(entry.as_str());
                    if entry_width > remaining {
                        break;
                    }
                    remaining -= entry_width;
                    spans.push(Span::styled(entry, Style::default().fg(Color::DarkGray)));
                }
            }
        }

        // Fill remaining width to prevent stale cells from previous frames
        let current_width = spans_display_width(&spans);
        let area_width = area.width as usize;
        if current_width < area_width {
            spans.push(Span::raw(" ".repeat(area_width - current_width)));
        }

        let footer =
            Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Rgb(30, 30, 30)));
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
