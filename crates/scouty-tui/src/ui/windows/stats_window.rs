//! Statistics summary overlay window (S key).

#[cfg(test)]
#[path = "stats_window_tests.rs"]
mod stats_window_tests;

use crate::app::App;
use crate::config::Theme;
use crate::ui::{ComponentResult, UiComponent};
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use scouty::record::LogLevel;
use std::collections::HashMap;

/// Pre-computed statistics for the overlay.
pub struct StatsData {
    pub total_records: usize,
    pub filtered_records: usize,
    pub time_first: Option<String>,
    pub time_last: Option<String>,
    pub level_counts: Vec<(LogLevel, usize)>,
    pub top_components: Vec<(String, usize)>,
}

impl StatsData {
    /// Compute statistics from the app state.
    pub fn compute(app: &App) -> Self {
        let total_records = app.total_records;
        let filtered_records = app.filtered_indices.len();

        let mut time_first: Option<chrono::DateTime<chrono::Utc>> = None;
        let mut time_last: Option<chrono::DateTime<chrono::Utc>> = None;
        let mut level_map: HashMap<LogLevel, usize> = HashMap::new();
        let mut component_map: HashMap<String, usize> = HashMap::new();

        for &idx in &app.filtered_indices {
            let record = &app.records[idx];

            // Time range
            let ts = record.timestamp;
            match time_first {
                None => time_first = Some(ts),
                Some(f) if ts < f => time_first = Some(ts),
                _ => {}
            }
            match time_last {
                None => time_last = Some(ts),
                Some(l) if ts > l => time_last = Some(ts),
                _ => {}
            }

            // Level distribution
            if let Some(level) = record.level {
                *level_map.entry(level).or_insert(0) += 1;
            }

            // Component counts
            if let Some(ref comp) = record.component_name {
                *component_map.entry(comp.clone()).or_insert(0) += 1;
            }
        }

        // Sort levels by severity order
        let mut level_counts: Vec<(LogLevel, usize)> = level_map.into_iter().collect();
        level_counts.sort_by_key(|(l, _)| *l);

        // Top 10 components by count
        let mut top_components: Vec<(String, usize)> = component_map.into_iter().collect();
        top_components.sort_by(|a, b| b.1.cmp(&a.1));
        top_components.truncate(10);

        let fmt_ts = |ts: chrono::DateTime<chrono::Utc>| -> String {
            ts.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
        };

        StatsData {
            total_records,
            filtered_records,
            time_first: time_first.map(fmt_ts),
            time_last: time_last.map(fmt_ts),
            level_counts,
            top_components,
        }
    }
}

/// Statistics overlay window.
pub struct StatsWindow<'a> {
    pub stats: &'a StatsData,
    pub theme: &'a Theme,
}

impl<'a> UiComponent for StatsWindow<'a> {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let width = 64u16.min(area.width.saturating_sub(4));
        // Calculate needed height dynamically
        let level_lines = self.stats.level_counts.len();
        let comp_lines = self.stats.top_components.len();
        // Header(1) + blank + Total(1) + TimeRange(2-3) + blank + LevelDist title(1) + levels + blank + Top10 title(1) + comps + padding
        let content_height = 4 + 2 + 1 + level_lines + 1 + 1 + comp_lines + 2;
        let height = (content_height as u16).min(area.height.saturating_sub(4));
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;
        let overlay = Rect::new(x, y, width, height);

        frame.render_widget(Clear, overlay);

        let section = |title: &str| -> Line<'_> {
            Line::styled(
                format!(" {title}"),
                self.theme
                    .dialog
                    .title
                    .to_style()
                    .add_modifier(Modifier::BOLD),
            )
        };

        let mut lines: Vec<Line<'_>> = Vec::new();

        // Total Records
        lines.push(section("Total Records"));
        lines.push(Line::from(format!(
            "  {} / {} (filtered / total)",
            self.stats.filtered_records, self.stats.total_records
        )));
        lines.push(Line::from(""));

        // Time Range
        lines.push(section("Time Range"));
        if let (Some(ref first), Some(ref last)) = (&self.stats.time_first, &self.stats.time_last) {
            lines.push(Line::from(format!("  First: {}", first)));
            lines.push(Line::from(format!("  Last:  {}", last)));
        } else {
            lines.push(Line::from("  (no data)"));
        }
        lines.push(Line::from(""));

        // Level Distribution
        lines.push(section("Level Distribution"));
        if self.stats.level_counts.is_empty() {
            lines.push(Line::from("  (no level data)"));
        } else {
            let max_count = self
                .stats
                .level_counts
                .iter()
                .map(|(_, c)| *c)
                .max()
                .unwrap_or(1);
            let bar_max_width = 30usize;
            let total = self.stats.filtered_records.max(1) as f64;

            for (level, count) in &self.stats.level_counts {
                let pct = (*count as f64 / total) * 100.0;
                let bar_len = (*count as f64 / max_count as f64 * bar_max_width as f64) as usize;
                let bar: String = "█".repeat(bar_len);
                lines.push(Line::from(format!(
                    "  {:<7} {:>6} ({:>5.1}%) {}",
                    format!("{}", level),
                    count,
                    pct,
                    bar
                )));
            }
        }
        lines.push(Line::from(""));

        // Top 10 Components
        lines.push(section("Top 10 Components"));
        if self.stats.top_components.is_empty() {
            lines.push(Line::from("  (no component data)"));
        } else {
            for (i, (name, count)) in self.stats.top_components.iter().enumerate() {
                let display_name = if name.chars().count() > 30 {
                    let truncated: String = name.chars().take(29).collect();
                    format!("{}…", truncated)
                } else {
                    name.clone()
                };
                lines.push(Line::from(format!(
                    "  {:>2}. {:<32} {}",
                    i + 1,
                    display_name,
                    count
                )));
            }
        }

        let para = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Statistics ")
                    .borders(Borders::ALL)
                    .border_style(self.theme.dialog.accent.to_style()),
            )
            .style(self.theme.dialog.background.to_style());
        frame.render_widget(para, overlay);
    }

    fn on_cancel(&mut self) -> ComponentResult {
        ComponentResult::Close
    }

    fn on_key(&mut self, _key: crossterm::event::KeyEvent) -> ComponentResult {
        ComponentResult::Close
    }

    fn on_char(&mut self, _c: char) -> ComponentResult {
        ComponentResult::Close
    }

    fn on_confirm(&mut self) -> ComponentResult {
        ComponentResult::Close
    }

    fn on_toggle(&mut self) -> ComponentResult {
        ComponentResult::Close
    }
}
