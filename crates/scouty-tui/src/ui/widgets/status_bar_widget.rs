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
use ratatui::style::{Color, Modifier, Style};
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
    /// Return the shortcut hints for the current mode/panel state.
    pub fn shortcut_hints(
        panel_focused: bool,
        active: crate::panel::PanelId,
    ) -> Vec<(&'static str, &'static str)> {
        if panel_focused {
            // Panel-specific hints from registry + common panel hints
            let mut hints = crate::ui::windows::main_window::panel_shortcut_hints(active);
            hints.push(("Tab/S-Tab", "Switch"));
            hints.push(("z", "Max"));
            hints.push(("Esc", "Close"));
            hints
        } else {
            vec![
                ("j/k", "↑↓"),
                ("/", "Search"),
                ("f", "Filter"),
                ("-/=", "Exclude/Include"),
                ("Enter", "Detail"),
                ("?", "Help"),
            ]
        }
    }

    /// Snap a raw ms-per-bucket value up to the nearest human-friendly interval.
    ///
    /// Covers the full range from sub-millisecond to 24h+:
    /// - Sub-second: 1ms, 2ms, 5ms, 10ms, 20ms, 50ms, 100ms, 200ms, 500ms
    /// - Seconds: 1s, 2s, 5s, 10s, 15s, 30s
    /// - Minutes: 1m, 2m, 5m, 10m, 15m, 30m
    /// - Hours: 1h, 2h, 6h, 12h, 24h
    ///
    /// Values beyond 24h are returned as-is.
    fn snap_to_standard(ms: f64) -> f64 {
        const INTERVALS_MS: &[f64] = &[
            1.0,          // 1ms
            2.0,          // 2ms
            5.0,          // 5ms
            10.0,         // 10ms
            20.0,         // 20ms
            50.0,         // 50ms
            100.0,        // 100ms
            200.0,        // 200ms
            500.0,        // 500ms
            1_000.0,      // 1s
            2_000.0,      // 2s
            5_000.0,      // 5s
            10_000.0,     // 10s
            15_000.0,     // 15s
            30_000.0,     // 30s
            60_000.0,     // 1m
            120_000.0,    // 2m
            300_000.0,    // 5m
            600_000.0,    // 10m
            900_000.0,    // 15m
            1_800_000.0,  // 30m
            3_600_000.0,  // 1h
            7_200_000.0,  // 2h
            21_600_000.0, // 6h
            43_200_000.0, // 12h
            86_400_000.0, // 24h
        ];
        if ms <= 0.0 {
            return INTERVALS_MS[0];
        }
        for &iv in INTERVALS_MS {
            if ms <= iv {
                return iv;
            }
        }
        // Beyond 24h, return as-is
        ms
    }

    /// Format the time-per-column label for the density chart.
    /// Returns the label string (e.g. "[█=5s]") or None if not applicable.
    fn time_per_column_label(cache: &crate::app::DensityCache) -> Option<String> {
        if cache.num_buckets == 0 || cache.min_ts == cache.max_ts {
            return None;
        }
        let range_ms = (cache.max_ts - cache.min_ts).num_milliseconds() as f64;
        let raw_ms = range_ms / cache.num_buckets as f64;
        let ms_per_bucket = Self::snap_to_standard(raw_ms);

        let label = if ms_per_bucket < 1000.0 {
            format!("[█={}ms]", ms_per_bucket.round() as u64)
        } else if ms_per_bucket < 60_000.0 {
            let secs = ms_per_bucket / 1000.0;
            // Show integer if close to whole number (e.g. 5.02→5, 5.97→6), else show 1 decimal (e.g. 5.5)
            if secs.fract() < 0.05 || secs.fract() > 0.95 {
                format!("[█={}s]", secs.round() as u64)
            } else {
                format!("[█={:.1}s]", secs)
            }
        } else if ms_per_bucket < 3_600_000.0 {
            let mins = ms_per_bucket / 60_000.0;
            if mins.fract() < 0.05 || mins.fract() > 0.95 {
                format!("[█={}m]", mins.round() as u64)
            } else {
                format!("[█={:.1}m]", mins)
            }
        } else {
            let hours = ms_per_bucket / 3_600_000.0;
            if hours.fract() < 0.05 || hours.fract() > 0.95 {
                format!("[█={}h]", hours.round() as u64)
            } else {
                format!("[█={:.1}h]", hours)
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

        let bookmark_text = if !app.bookmarks.is_empty() {
            format!(" Bookmarks: {} ", app.bookmarks.len())
        } else {
            String::new()
        };

        let position_text = format!(" {} ", position);
        let right_text = format!("{}{}", bookmark_text, position_text);
        let right_width = UnicodeWidthStr::width(right_text.as_str()) as u16;

        // Account for follow indicator width in chart space
        let follow_width: u16 = if app.follow_mode {
            if app.follow_new_count > 0 {
                (format!("[FOLLOW ↓{}] ", app.follow_new_count).len() + 2) as u16
            } else {
                "[FOLLOW] ".len() as u16
            }
        } else {
            0
        };

        let chart_width = area.width.saturating_sub(right_width + follow_width + 2) as usize;

        let mut spans: Vec<Span> = Vec::new();

        // Follow mode indicator
        if app.follow_mode {
            if app.follow_new_count > 0 {
                spans.push(Span::styled(
                    format!("[FOLLOW ↓{}] ", app.follow_new_count),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::styled(
                    "[FOLLOW] ".to_string(),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ));
            }
        }

        if chart_width >= 4 && app.total() > 0 {
            if let Some(cache) = &app.density_cache {
                // Show time-per-column label before chart
                if let Some(label) = Self::time_per_column_label(cache) {
                    let source_label = app.density_source_label();
                    let full_label = if source_label == "All" {
                        label
                    } else {
                        match label.strip_suffix(']') {
                            Some(prefix) => format!("{prefix} {source_label}]"),
                            None => format!("{label} {source_label}"),
                        }
                    };
                    spans.push(Span::styled(
                        full_label,
                        theme.status_bar.density_label.to_style(),
                    ));
                }

                let cursor_char_idx = app.cursor_char_in_density();

                for (i, ch) in cache.braille_text.chars().enumerate() {
                    let style = if Some(i) == cursor_char_idx {
                        theme.status_bar.cursor_marker.to_style()
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
            let (before, cursor_ch, after) = app.command_input.render_parts();
            spans.push(Span::styled(
                format!(" :{}", before),
                theme.dialog.text.to_style(),
            ));
            spans.push(Span::styled(cursor_ch, theme.input.cursor.to_style()));
            if !after.is_empty() {
                spans.push(Span::styled(
                    after.to_string(),
                    theme.dialog.text.to_style(),
                ));
            }
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
            let (before, cursor_ch, after) = app.time_input.render_parts();
            spans.push(Span::styled(
                format!(" {}", before),
                theme.dialog.text.to_style(),
            ));
            spans.push(Span::styled(cursor_ch, theme.input.cursor.to_style()));
            if !after.is_empty() {
                spans.push(Span::styled(
                    after.to_string(),
                    theme.dialog.text.to_style(),
                ));
            }
        } else {
            let panel_focused = app.panel_state.expanded
                && app.panel_state.focus == crate::panel::PanelFocus::PanelContent;

            let (mode_label, mode_style) = if panel_focused {
                let label = app.panel_state.active.status_label();
                (label, theme.status_bar.mode_view.to_style())
            } else if app.follow_mode {
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
                let shortcuts: Vec<(&str, &str)> = if app.shortcut_hints_cache.is_empty() {
                    // Fallback to static hints if cache not populated
                    Self::shortcut_hints(panel_focused, app.panel_state.active)
                } else {
                    app.shortcut_hints_cache
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.as_str()))
                        .collect()
                };

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
