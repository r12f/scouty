//! Stats panel widget — renders statistics inside the panel system.

use crate::app::App;
use crate::ui::windows::stats_window::StatsData;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub struct StatsPanelWidget;

impl StatsPanelWidget {
    pub fn render_with_app(frame: &mut Frame, area: Rect, app: &App) {
        let stats = StatsData::compute(app);
        let theme = &app.theme;

        let section = |title: &str| -> Line<'_> {
            Line::styled(
                format!(" {title}"),
                theme.dialog.title.to_style().add_modifier(Modifier::BOLD),
            )
        };

        let mut lines: Vec<Line<'_>> = Vec::new();

        // Total Records
        lines.push(section("Total Records"));
        lines.push(Line::from(format!(
            "  {} / {} (filtered / total)",
            stats.filtered_records, stats.total_records
        )));
        lines.push(Line::from(""));

        // Time Range
        lines.push(section("Time Range"));
        if let (Some(ref first), Some(ref last)) = (&stats.time_first, &stats.time_last) {
            lines.push(Line::from(format!("  First: {}", first)));
            lines.push(Line::from(format!("  Last:  {}", last)));
        } else {
            lines.push(Line::from("  (no data)"));
        }
        lines.push(Line::from(""));

        // Level Distribution
        lines.push(section("Level Distribution"));
        if stats.level_counts.is_empty() {
            lines.push(Line::from("  (no level data)"));
        } else {
            let max_count = stats
                .level_counts
                .iter()
                .map(|(_, c)| *c)
                .max()
                .unwrap_or(1);
            let bar_max_width = 30usize;
            let total = stats.filtered_records.max(1) as f64;

            for (level, count) in &stats.level_counts {
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
        if stats.top_components.is_empty() {
            lines.push(Line::from("  (no component data)"));
        } else {
            for (i, (name, count)) in stats.top_components.iter().enumerate() {
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

        tracing::trace!(
            filtered = stats.filtered_records,
            total = stats.total_records,
            levels = stats.level_counts.len(),
            "rendering stats panel"
        );

        let para = Paragraph::new(lines);
        frame.render_widget(para, area);
    }
}
