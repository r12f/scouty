//! Status bar widget — 2-line bottom bar.
//!
//! Line 1: density chart (left) with time-per-column label + position info (right)
//! Line 2: mode label + shortcut hints

#[cfg(test)]
#[path = "status_bar_widget_tests.rs"]
mod status_bar_widget_tests;

use crate::app::App;
use crate::ui::UiComponent;
use ratatui::layout::Rect;

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
    /// Format the time-per-column label for the density chart.
    /// Returns the label string (e.g. "[5s/█]") or None if not applicable.
    fn time_per_column_label(cache: &crate::app::DensityCache) -> Option<String> {
        if cache.num_buckets == 0 || cache.min_ts == cache.max_ts {
            return None;
        }
        let range_ms = (cache.max_ts - cache.min_ts).num_milliseconds() as f64;
        let ms_per_bucket = range_ms / cache.num_buckets as f64;

        let label = if ms_per_bucket < 1000.0 {
            format!("[{}ms/█]", ms_per_bucket.round() as u64)
        } else if ms_per_bucket < 60_000.0 {
            let secs = ms_per_bucket / 1000.0;
            // Show integer if close to whole number (e.g. 5.02→5, 5.97→6), else show 1 decimal (e.g. 5.5)
            if secs.fract() < 0.05 || secs.fract() > 0.95 {
                format!("[{}s/█]", secs.round() as u64)
            } else {
                format!("[{:.1}s/█]", secs)
            }
        } else if ms_per_bucket < 3_600_000.0 {
            let mins = ms_per_bucket / 60_000.0;
            if mins.fract() < 0.05 || mins.fract() > 0.95 {
                format!("[{}m/█]", mins.round() as u64)
            } else {
                format!("[{:.1}m/█]", mins)
            }
        } else {
            let hours = ms_per_bucket / 3_600_000.0;
            if hours.fract() < 0.05 || hours.fract() > 0.95 {
                format!("[{}h/█]", hours.round() as u64)
            } else {
                format!("[{:.1}h/█]", hours)
            }
        };
        Some(label)
    }

    /// Render line 1: density chart + position info.
    pub fn render_line1(&self, frame: &mut Frame, area: Rect, app: &App) {
        let theme = &app.theme;
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
                // Show time-per-column label before chart
                if let Some(label) = Self::time_per_column_label(cache) {
                    spans.push(Span::styled(
                        label,
                        ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray),
                    ));
                }

                let cursor_char_idx = app.cursor_char_in_density();

                for (i, ch) in cache.braille_text.chars().enumerate() {
                    let style = if Some(i) == cursor_char_idx {
                        theme.status_bar.density_hot.to_style()
                    } else {
                        theme.status_bar.density_normal.to_style()
                    };
                    spans.push(Span::styled(ch.to_string(), style));
                }
            }
        }

        let used_width = spans_display_width(&spans);
        let total_width = area.width as usize;
        let right_len = right_text.len();
        if used_width + right_len < total_width {
            let pad = total_width - used_width - right_len;
            spans.push(Span::styled(
                " ".repeat(pad),
                theme.status_bar.line1_bg.to_style(),
            ));
        }

        spans.push(Span::styled(
            right_text,
            theme.status_bar.position.to_style(),
        ));

        let footer = Paragraph::new(Line::from(spans)).style(theme.status_bar.line1_bg.to_style());
        frame.render_widget(footer, area);
    }

    /// Render line 2: mode label + shortcut hints or status message.
    pub fn render_line2(&self, frame: &mut Frame, area: Rect, app: &App) {
        let theme = &app.theme;
        let mut spans: Vec<Span> = Vec::new();

        if app.input_mode == crate::app::InputMode::Command {
            spans.push(Span::styled(
                " [CMD] ",
                theme.status_bar.command_mode_label.to_style(),
            ));
            spans.push(Span::styled(
                format!(" :{}█", app.command_input),
                theme.dialog.text.to_style(),
            ));
        } else if app.input_mode == crate::app::InputMode::JumpForward
            || app.input_mode == crate::app::InputMode::JumpBackward
        {
            let label = if app.input_mode == crate::app::InputMode::JumpForward {
                "[JUMP+]"
            } else {
                "[JUMP-]"
            };
            spans.push(Span::styled(
                format!(" {} ", label),
                theme.status_bar.mode_label.to_style(),
            ));
            spans.push(Span::styled(
                format!(" {}█", app.time_input),
                theme.dialog.text.to_style(),
            ));
        } else {
            let (mode_label, mode_style) = if app.follow_mode {
                ("[FOLLOW]", theme.status_bar.mode_follow.to_style())
            } else {
                ("[VIEW]", theme.status_bar.mode_view.to_style())
            };

            spans.push(Span::styled(format!(" {} ", mode_label), mode_style));

            if let Some(ref msg) = app.status_message {
                spans.push(Span::styled(
                    format!(" {} ", msg),
                    theme.status_bar.shortcut_key.to_style(),
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
                    spans.push(Span::styled(
                        entry,
                        theme.status_bar.shortcut_sep.to_style(),
                    ));
                }
            }
        }

        let current_width = spans_display_width(&spans);
        let area_width = area.width as usize;
        if current_width < area_width {
            spans.push(Span::raw(" ".repeat(area_width - current_width)));
        }

        let footer = Paragraph::new(Line::from(spans)).style(theme.status_bar.line2_bg.to_style());
        frame.render_widget(footer, area);
    }
}

impl UiComponent for StatusBarWidget {
    fn render(&self, _frame: &mut Frame, _area: Rect) {}

    fn enable_jk_navigation(&self) -> bool {
        false
    }
}
