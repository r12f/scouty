//! Category panel widget — list view with name, count, and density sparkline.

#[cfg(test)]
#[path = "category_panel_widget_tests.rs"]
mod category_panel_widget_tests;

use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use unicode_width::UnicodeWidthStr;

/// Block characters for density sparkline (8 levels).
const SPARK_CHARS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// A prepared category entry for display.
#[derive(Debug, Clone)]
pub struct CategoryDisplayEntry {
    pub name: String,
    pub count: usize,
    pub density: Vec<u64>,
}

pub struct CategoryPanelWidget;

impl CategoryPanelWidget {
    /// Build display entries from the current App state.
    pub fn build_entries(app: &App) -> Vec<CategoryDisplayEntry> {
        let Some(cp) = &app.category_processor else {
            return Vec::new();
        };
        cp.store
            .categories
            .iter()
            .map(|cat| CategoryDisplayEntry {
                name: cat.definition.name.clone(),
                count: cat.count,
                density: cat.density.clone(),
            })
            .collect()
    }

    /// Render the category panel into the given area.
    pub fn render(frame: &mut Frame, app: &App, area: Rect) {
        let entries = Self::build_entries(app);
        let focused = app.panel_state.expanded
            && app.panel_state.focus == crate::panel::PanelFocus::PanelContent
            && app.panel_state.active == crate::panel::PanelId::Category;

        // Use theme-driven border style matching other panels (Detail, Region).
        let block = Block::default().borders(Borders::NONE);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if entries.is_empty() {
            let msg = Paragraph::new("No categories configured")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(msg, inner);
            return;
        }

        // Calculate column widths using display width for Unicode correctness.
        let max_name_width = entries
            .iter()
            .map(|e| UnicodeWidthStr::width(e.name.as_str()))
            .max()
            .unwrap_or(8);
        let count_width: usize = 10;
        let sparkline_width = inner
            .width
            .saturating_sub(max_name_width as u16 + count_width as u16 + 4)
            as usize;

        // Clamp cursor in case categories changed since last interaction.
        let cursor = app.category_cursor.min(entries.len().saturating_sub(1));
        let visible_height = inner.height as usize;

        // Scroll offset to keep cursor visible
        let scroll = if cursor >= visible_height {
            cursor - visible_height + 1
        } else {
            0
        };

        let lines: Vec<Line> = entries
            .iter()
            .enumerate()
            .skip(scroll)
            .take(visible_height)
            .map(|(i, entry)| {
                let is_selected = focused && i == cursor;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let name_display_width = UnicodeWidthStr::width(entry.name.as_str());
                let padding = max_name_width.saturating_sub(name_display_width);
                let name = format!("{}{}", entry.name, " ".repeat(padding));
                let count_str = format_count(entry.count);
                let count = format!("{:>width$}", count_str, width = count_width);
                let sparkline = render_sparkline(&entry.density, sparkline_width);

                Line::from(vec![
                    Span::styled(name, style),
                    Span::styled("  ", style),
                    Span::styled(count, style),
                    Span::styled("  ", style),
                    Span::styled(sparkline, style),
                ])
            })
            .collect();

        let para = Paragraph::new(lines);
        frame.render_widget(para, inner);
    }
}

/// Format count with comma separators (e.g., 1,234,567).
pub fn format_count(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Render a density sparkline string from bucket values.
pub fn render_sparkline(buckets: &[u64], width: usize) -> String {
    if buckets.is_empty() || width == 0 {
        return String::new();
    }

    let resampled = resample(buckets, width);
    let max_val = *resampled.iter().max().unwrap_or(&0);

    resampled
        .iter()
        .map(|&v| {
            if max_val == 0 || v == 0 {
                ' '
            } else {
                let idx = ((v as f64 / max_val as f64) * 7.0).round() as usize;
                SPARK_CHARS[idx.min(7)]
            }
        })
        .collect()
}

/// Resample N buckets into target_width bins using simple averaging.
pub fn resample(buckets: &[u64], target: usize) -> Vec<u64> {
    if target == 0 {
        return Vec::new();
    }
    if buckets.len() <= target {
        let mut out = buckets.to_vec();
        out.resize(target, 0);
        return out;
    }
    let step = buckets.len() as f64 / target as f64;
    (0..target)
        .map(|i| {
            let start = (i as f64 * step) as usize;
            let end = (((i + 1) as f64 * step) as usize).min(buckets.len());
            if end <= start {
                0
            } else {
                let sum: u64 = buckets[start..end].iter().sum();
                sum / (end - start) as u64
            }
        })
        .collect()
}
