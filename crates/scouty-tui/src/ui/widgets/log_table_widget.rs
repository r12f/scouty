//! Log table widget — main scrollable log table.

#[cfg(test)]
#[path = "log_table_widget_tests.rs"]
mod log_table_widget_tests;

use crate::app::{App, Column};
use crate::config::Theme;
use crate::ui::{ComponentResult, UiComponent};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Cell, Row, Table};
use ratatui::Frame;
use scouty::record::LogLevel;
use scouty::region::store::RegionStore;

/// Region gutter marker colors (matching RegionManagerWindow palette).
const REGION_COLORS: &[Color] = &[
    Color::Cyan,
    Color::Yellow,
    Color::Green,
    Color::Magenta,
    Color::Blue,
    Color::Red,
];

/// Build a gutter cell showing region markers for a record.
/// When overlapping, shows the innermost (shortest) region marker.
fn region_gutter_marker(record_idx: usize, store: &RegionStore) -> Cell<'static> {
    if let Some(region) = store.innermost_at(record_idx) {
        // Determine color by definition name hash for consistency
        let color_idx = region
            .definition_name
            .bytes()
            .fold(0usize, |acc, b| acc.wrapping_add(b as usize));
        let color = REGION_COLORS[color_idx % REGION_COLORS.len()];
        let marker = if region.timed_out {
            "░ "
        } else if record_idx == region.start_index {
            "▶ "
        } else if record_idx == region.end_index {
            "◀ "
        } else {
            "│ "
        };
        Cell::from(marker.to_string()).style(Style::default().fg(color))
    } else {
        Cell::from("  ".to_string())
    }
}

pub fn level_style(level: Option<LogLevel>, theme: &Theme) -> Style {
    match level {
        Some(LogLevel::Fatal) => theme.log_levels.fatal.to_style(),
        Some(LogLevel::Error) => theme.log_levels.error.to_style(),
        Some(LogLevel::Warn) => theme.log_levels.warn.to_style(),
        Some(LogLevel::Notice) => theme.log_levels.notice.to_style(),
        Some(LogLevel::Info) => theme.log_levels.info.to_style(),
        Some(LogLevel::Debug) => theme.log_levels.debug.to_style(),
        Some(LogLevel::Trace) => theme.log_levels.trace.to_style(),
        None => Style::default(),
    }
}

pub struct LogTableWidget;

impl LogTableWidget {
    pub fn render_with_app(&self, frame: &mut Frame, area: Rect, app: &App) {
        let theme = &app.theme;
        tracing::trace!(
            panel_focused = app.panel_state.expanded
                && app.panel_state.focus == crate::panel::PanelFocus::PanelContent,
            "rendering log table header"
        );
        let visible = app.visible_records();
        let cw = &app.col_widths;
        let vis_cols = app.column_config.visible_columns();

        let sep_style = theme.table.separator.to_style_entry().to_style();
        let sep_char = theme.table.separator.separator_char();
        let has_regions = !app.regions.is_empty();

        let mut widths: Vec<Constraint> = Vec::new();
        // Gutter column for region markers (2 chars)
        if has_regions {
            widths.push(Constraint::Length(2));
            widths.push(Constraint::Length(1)); // separator
        }
        widths.extend(
            vis_cols
                .iter()
                .enumerate()
                .flat_map(|(i, col)| {
                    let w = if *col == Column::Log {
                        Constraint::Fill(1)
                    } else {
                        let cfg_idx = app.column_config.columns.iter().position(|(c, _)| c == col);
                        let auto_w = if let Some(cw_idx) = col.col_widths_index() {
                            cw[cw_idx]
                        } else {
                            col.default_fixed_width()
                        };
                        let effective = if let Some(idx) = cfg_idx {
                            app.column_config.effective_width(idx, auto_w)
                        } else {
                            auto_w
                        };
                        Constraint::Length(effective)
                    };
                    if i < vis_cols.len() - 1 {
                        vec![w, Constraint::Length(1)]
                    } else {
                        vec![w]
                    }
                })
                .collect::<Vec<_>>(),
        );

        let mut header_cells: Vec<Cell> = Vec::new();
        if has_regions {
            header_cells.push(Cell::from("").style(Style::default()));
            header_cells.push(Cell::from(sep_char).style(sep_style));
        }
        header_cells.extend(vis_cols.iter().enumerate().flat_map(|(i, col)| {
            let cell = Cell::from(col.label()).style(Style::default().add_modifier(Modifier::BOLD));
            if i < vis_cols.len() - 1 {
                vec![cell, Cell::from(sep_char).style(sep_style)]
            } else {
                vec![cell]
            }
        }));

        let panel_has_focus = app.panel_state.expanded
            && app.panel_state.focus == crate::panel::PanelFocus::PanelContent;
        let header_style = if panel_has_focus {
            theme.table.header_unfocused.to_style()
        } else {
            theme.table.header.to_style()
        };
        let header = Row::new(header_cells).style(header_style);

        let rows: Vec<Row> = visible
            .iter()
            .enumerate()
            .map(|(i, record)| {
                let filtered_idx = app.scroll_offset + i;
                let is_selected = filtered_idx == app.selected;
                let is_match = app.is_search_match(filtered_idx);
                let record_idx = app.filtered_indices[filtered_idx];
                let is_bookmarked = app.is_bookmarked(record_idx);
                let row_style = level_style(record.level, theme);

                let mut cells: Vec<Cell> = Vec::new();

                // Region gutter marker
                if has_regions {
                    let gutter = region_gutter_marker(record_idx, &app.regions);
                    cells.push(gutter);
                    cells.push(Cell::from(sep_char).style(sep_style));
                }

                cells.extend(
                    vis_cols
                        .iter()
                        .enumerate()
                        .flat_map(|(ci, col)| {
                            let cell = match col {
                                Column::Time => Cell::from(
                                    record.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                                ),
                                Column::Level => Cell::from(
                                    record.level.map(|l| format!("{}", l)).unwrap_or_default(),
                                ),
                                Column::Hostname => {
                                    Cell::from(record.hostname.as_deref().unwrap_or("").to_string())
                                }
                                Column::Container => Cell::from(
                                    record.container.as_deref().unwrap_or("").to_string(),
                                ),
                                Column::Context => {
                                    Cell::from(record.context.as_deref().unwrap_or("").to_string())
                                }
                                Column::Function => {
                                    Cell::from(record.function.as_deref().unwrap_or("").to_string())
                                }
                                Column::ProcessName => Cell::from(
                                    record.process_name.as_deref().unwrap_or("").to_string(),
                                ),
                                Column::Pid => Cell::from(
                                    record.pid.map(|p| p.to_string()).unwrap_or_default(),
                                ),
                                Column::Tid => Cell::from(
                                    record.tid.map(|t| t.to_string()).unwrap_or_default(),
                                ),
                                Column::Component => Cell::from(
                                    record.component_name.as_deref().unwrap_or("").to_string(),
                                ),
                                Column::Source => Cell::from(record.source.to_string()),
                                Column::Log => Cell::from(record.message.clone()),
                            };
                            if ci < vis_cols.len() - 1 {
                                vec![cell, Cell::from(sep_char).style(sep_style)]
                            } else {
                                vec![cell]
                            }
                        })
                        .collect::<Vec<_>>(),
                );

                // Determine highlight row background: last matching rule wins
                let highlight_bg: Option<Color> = if app.highlight_rules.is_empty() {
                    None
                } else {
                    let mut bg = None;
                    for rule in &app.highlight_rules {
                        if rule.regex.is_match(&record.message) {
                            bg = Some(rule.color);
                        }
                    }
                    bg
                };

                let mut row = Row::new(cells).style(row_style);
                if is_selected && is_match {
                    row = row.style(row_style.bg(theme.table.selected_search.bg_color()));
                } else if is_selected && is_bookmarked {
                    row = row.style(row_style.bg(theme.table.selected_highlight.bg_color()));
                } else if is_selected {
                    row = row.style(row_style.bg(theme.table.selected.bg_color()));
                } else if is_match {
                    row = row.style(row_style.bg(theme.table.search_match.bg_color()));
                } else if is_bookmarked {
                    row = row.style(row_style.bg(theme.table.bookmark.bg_color()));
                } else if let Some(bg) = highlight_bg {
                    row = row.style(Style::default().bg(bg).fg(Color::Black));
                }
                row
            })
            .collect();

        let table = Table::new(rows, widths).header(header).column_spacing(0);
        frame.render_widget(table, area);
    }
}

impl UiComponent for LogTableWidget {
    fn render(&self, _frame: &mut Frame, _area: Rect) {}

    fn enable_jk_navigation(&self) -> bool {
        true
    }

    fn on_up(&mut self) -> ComponentResult {
        ComponentResult::Consumed
    }

    fn on_down(&mut self) -> ComponentResult {
        ComponentResult::Consumed
    }

    fn on_page_up(&mut self) -> ComponentResult {
        ComponentResult::Consumed
    }

    fn on_page_down(&mut self) -> ComponentResult {
        ComponentResult::Consumed
    }
}
