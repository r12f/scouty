//! Region panel widget — left-right split with region list and timeline Gantt.

#[cfg(test)]
#[path = "region_panel_widget_tests.rs"]
mod region_panel_widget_tests;

use crate::app::App;
use crate::panel::{Panel, PanelHeight};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

/// Sort modes for the region list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionSortMode {
    StartTime,
    Duration,
}

impl RegionSortMode {
    pub fn toggle(self) -> Self {
        match self {
            RegionSortMode::StartTime => RegionSortMode::Duration,
            RegionSortMode::Duration => RegionSortMode::StartTime,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            RegionSortMode::StartTime => "start",
            RegionSortMode::Duration => "duration",
        }
    }
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

/// Minimum width for the right (timeline) pane; below this, hide it.
const MIN_TIMELINE_WIDTH: u16 = 40;

/// A prepared region entry for display.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RegionDisplayEntry {
    pub name: String,
    pub definition_name: String,
    pub start_ts: String,
    pub end_ts: String,
    pub duration: String,
    pub description: String,
    pub start_index: usize,
    pub end_index: usize,
    pub timed_out: bool,
    /// Absolute start/end timestamps for timeline calculation.
    pub start_epoch_ms: i64,
    pub end_epoch_ms: i64,
}

pub struct RegionPanelWidget;

impl RegionPanelWidget {
    /// Build display entries from App state.
    pub fn build_entries(app: &App) -> Vec<RegionDisplayEntry> {
        let mut entries: Vec<RegionDisplayEntry> = app
            .regions
            .regions()
            .iter()
            .map(|region| {
                let start_rec = app.records.get(region.start_index);
                let end_rec = app.records.get(region.end_index);

                let start_ts = start_rec
                    .map(|r| r.timestamp.format("%H:%M:%S").to_string())
                    .unwrap_or_else(|| "?".to_string());
                let end_ts = if region.timed_out {
                    "?".to_string()
                } else {
                    end_rec
                        .map(|r| r.timestamp.format("%H:%M:%S").to_string())
                        .unwrap_or_else(|| "?".to_string())
                };

                let start_epoch_ms = start_rec
                    .map(|r| r.timestamp.timestamp_millis())
                    .unwrap_or(0);
                let end_epoch_ms = end_rec
                    .map(|r| r.timestamp.timestamp_millis())
                    .unwrap_or(start_epoch_ms);

                let duration_ms = end_epoch_ms - start_epoch_ms;
                let duration = if region.timed_out {
                    format!(">{}s ⏱", duration_ms / 1000)
                } else {
                    format_duration(duration_ms)
                };

                let description = region.description.clone().unwrap_or_default();

                RegionDisplayEntry {
                    name: region.name.clone(),
                    definition_name: region.definition_name.clone(),
                    start_ts,
                    end_ts,
                    duration,
                    description,
                    start_index: region.start_index,
                    end_index: region.end_index,
                    timed_out: region.timed_out,
                    start_epoch_ms,
                    end_epoch_ms,
                }
            })
            .collect();

        // Apply type filter
        if let Some(ref type_filter) = app.region_panel_type_filter {
            entries.retain(|e| e.definition_name == *type_filter);
        }

        // Sort
        match app.region_panel_sort {
            RegionSortMode::StartTime => {
                entries.sort_by(|a, b| {
                    a.start_epoch_ms
                        .cmp(&b.start_epoch_ms)
                        .then(a.end_epoch_ms.cmp(&b.end_epoch_ms))
                });
            }
            RegionSortMode::Duration => {
                entries.sort_by(|a, b| {
                    let da = a.end_epoch_ms - a.start_epoch_ms;
                    let db = b.end_epoch_ms - b.start_epoch_ms;
                    db.cmp(&da) // longest first
                });
            }
        }

        entries
    }

    pub fn render_with_app(&self, frame: &mut Frame, area: Rect, app: &App) {
        let entries = Self::build_entries(app);

        let block = Block::default()
            .title(" Region ")
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if entries.is_empty() {
            let msg = Paragraph::new("No regions detected.");
            frame.render_widget(msg, inner);
            return;
        }

        // Collect unique type names for color mapping and type count
        let mut type_names: Vec<String> =
            entries.iter().map(|e| e.definition_name.clone()).collect();
        type_names.sort();
        type_names.dedup();

        let type_color_map: std::collections::HashMap<&str, Color> = type_names
            .iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), REGION_COLORS[i % REGION_COLORS.len()]))
            .collect();

        // Decide if right pane should be shown
        let show_timeline = inner.width >= MIN_TIMELINE_WIDTH + 30; // need at least 30 for left

        if show_timeline {
            let chunks =
                Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)])
                    .split(inner);
            self.render_list(frame, chunks[0], &entries, &type_color_map, app);
            self.render_timeline(
                frame,
                chunks[1],
                &entries,
                &type_names,
                &type_color_map,
                app,
            );
        } else {
            self.render_list(frame, inner, &entries, &type_color_map, app);
        }
    }

    fn render_list(
        &self,
        frame: &mut Frame,
        area: Rect,
        entries: &[RegionDisplayEntry],
        type_color_map: &std::collections::HashMap<&str, Color>,
        app: &App,
    ) {
        let has_focus =
            app.panel_state.has_focus() && app.panel_state.active == crate::panel::PanelId::Region;

        // Determine cursor: manual when focused, auto-follow when not
        let cursor = if has_focus {
            app.region_manager_cursor
                .min(entries.len().saturating_sub(1))
        } else {
            // Auto-follow: find region containing the current log cursor
            let record_idx = app.filtered_indices.get(app.selected).copied();
            record_idx
                .and_then(|idx| {
                    entries
                        .iter()
                        .position(|e| e.start_index <= idx && idx <= e.end_index)
                })
                .unwrap_or(
                    app.region_manager_cursor
                        .min(entries.len().saturating_sub(1)),
                )
        };

        // Reserve 1 line for summary footer
        let list_height = area.height.saturating_sub(1) as usize;

        let scroll_offset = if cursor >= list_height {
            cursor - list_height + 1
        } else {
            0
        };

        let mut lines: Vec<Line> = Vec::new();
        let width = area.width as usize;

        for (i, entry) in entries.iter().enumerate().skip(scroll_offset) {
            if lines.len() >= list_height {
                break;
            }

            let color = type_color_map
                .get(entry.definition_name.as_str())
                .copied()
                .unwrap_or(Color::White);

            let is_selected = i == cursor && has_focus;
            let prefix = if i == cursor { "▸" } else { " " };
            let timeout_mark = if entry.timed_out { " ⏱" } else { "" };

            // Format: ▸ Name  HH:MM:SS→HH:MM:SS  duration
            let time_range = format!("{}→{}", entry.start_ts, entry.end_ts);
            let name_max = width
                .saturating_sub(time_range.len() + entry.duration.len() + timeout_mark.len() + 8);
            let name: String = entry.name.chars().take(name_max).collect();

            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color)
            };

            let line_str = format!(
                "{} {:<nw$}  {}  {}{}",
                prefix,
                name,
                time_range,
                entry.duration,
                timeout_mark,
                nw = name_max,
            );

            // Truncate to width
            let display: String = line_str.chars().take(width).collect();
            lines.push(Line::styled(display, style));
        }

        // Footer summary
        let completed = entries.iter().filter(|e| !e.timed_out).count();
        let timed_out = entries.iter().filter(|e| e.timed_out).count();
        let sort_label = app.region_panel_sort.label();
        let filter_info = if app.region_panel_type_filter.is_some() {
            " [filtered]"
        } else {
            ""
        };
        let footer_text = format!(
            " Total: {} regions | {} completed | {} timeout | sort: {}{}",
            entries.len(),
            completed,
            timed_out,
            sort_label,
            filter_info,
        );

        let list_area = Rect::new(area.x, area.y, area.width, list_height as u16);
        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, list_area);

        let footer_area = Rect::new(area.x, area.y + list_height as u16, area.width, 1);
        let footer = Paragraph::new(Line::styled(
            footer_text,
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(footer, footer_area);
    }

    fn render_timeline(
        &self,
        frame: &mut Frame,
        area: Rect,
        entries: &[RegionDisplayEntry],
        type_names: &[String],
        type_color_map: &std::collections::HashMap<&str, Color>,
        app: &App,
    ) {
        let block = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(Color::DarkGray));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.width < 2 || inner.height == 0 {
            return;
        }

        let cursor = app
            .region_manager_cursor
            .min(entries.len().saturating_sub(1));
        let selected_type = entries.get(cursor).map(|e| e.definition_name.as_str());

        // Compute global time range
        let global_min = entries.iter().map(|e| e.start_epoch_ms).min().unwrap_or(0);
        let global_max = entries.iter().map(|e| e.end_epoch_ms).max().unwrap_or(0);
        let time_span = (global_max - global_min).max(1);

        let bar_width = inner.width.saturating_sub(1) as usize; // leave 1 char margin

        let mut lines: Vec<Line> = Vec::new();
        let max_lines = inner.height as usize;

        for type_name in type_names {
            if lines.len() + 2 > max_lines {
                break;
            }

            let color = type_color_map
                .get(type_name.as_str())
                .copied()
                .unwrap_or(Color::White);

            let is_selected_type = selected_type == Some(type_name.as_str());

            // Type name line
            let name_style = if is_selected_type {
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let name_display: String = type_name.chars().take(bar_width).collect();
            lines.push(Line::styled(format!(" {}", name_display), name_style));

            if lines.len() >= max_lines {
                break;
            }

            // Build timeline bar
            let type_entries: Vec<&RegionDisplayEntry> = entries
                .iter()
                .filter(|e| e.definition_name == *type_name)
                .collect();

            let mut bar = vec![' '; bar_width];

            for entry in &type_entries {
                let start_pos = ((entry.start_epoch_ms - global_min) as f64 / time_span as f64
                    * bar_width as f64) as usize;
                let end_pos = ((entry.end_epoch_ms - global_min) as f64 / time_span as f64
                    * bar_width as f64) as usize;

                let start_pos = start_pos.min(bar_width.saturating_sub(1));
                let end_pos = end_pos.min(bar_width).max(start_pos + 1);

                let ch = if entry.timed_out { '░' } else { '█' };
                for item in bar.iter_mut().take(end_pos).skip(start_pos) {
                    *item = ch;
                }
            }

            // Highlight selected region's bar segment with bright color
            let selected_entry = entries.get(cursor);
            let bar_str: String = bar.iter().collect();

            let mut spans: Vec<Span> = Vec::new();
            spans.push(Span::raw(" "));

            if is_selected_type {
                // Color the bar, with the selected region's segment in bright
                if let Some(sel) = selected_entry {
                    if sel.definition_name == *type_name {
                        let sel_start = ((sel.start_epoch_ms - global_min) as f64
                            / time_span as f64
                            * bar_width as f64) as usize;
                        let sel_end = ((sel.end_epoch_ms - global_min) as f64 / time_span as f64
                            * bar_width as f64) as usize;
                        let sel_start = sel_start.min(bar_width.saturating_sub(1));
                        let sel_end = sel_end.min(bar_width).max(sel_start + 1);

                        // Before selected
                        if sel_start > 0 {
                            spans.push(Span::styled(
                                bar_str[..sel_start].to_string(),
                                Style::default().fg(color),
                            ));
                        }
                        // Selected segment — bright / reversed
                        spans.push(Span::styled(
                            bar_str[sel_start..sel_end].to_string(),
                            Style::default().fg(Color::White).bg(color),
                        ));
                        // After selected
                        if sel_end < bar_width {
                            spans.push(Span::styled(
                                bar_str[sel_end..].to_string(),
                                Style::default().fg(color),
                            ));
                        }
                    } else {
                        spans.push(Span::styled(bar_str.clone(), Style::default().fg(color)));
                    }
                } else {
                    spans.push(Span::styled(bar_str.clone(), Style::default().fg(color)));
                }
            } else {
                spans.push(Span::styled(bar_str, Style::default().fg(Color::DarkGray)));
            }

            lines.push(Line::from(spans));
        }

        let timeline_para = Paragraph::new(lines);
        frame.render_widget(timeline_para, inner);
    }
}

impl Panel for RegionPanelWidget {
    fn name(&self) -> &str {
        "Region"
    }

    fn shortcut(&self) -> Option<char> {
        Some('r')
    }

    fn default_height(&self) -> PanelHeight {
        PanelHeight::Percentage(40)
    }

    fn is_available(&self) -> bool {
        true
    }

    fn on_log_cursor_changed(&mut self, _index: usize) {
        // Cursor follow is handled in App by checking regions_at(index)
    }
}

/// Format a duration in milliseconds to a human-readable string.
pub(crate) fn format_duration(ms: i64) -> String {
    if ms < 0 {
        return "0ms".to_string();
    }
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        let secs = ms as f64 / 1000.0;
        format!("{:.1}s", secs)
    } else {
        let mins = ms / 60_000;
        let secs = (ms % 60_000) / 1000;
        format!("{}m{}s", mins, secs)
    }
}
