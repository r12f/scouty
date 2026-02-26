//! Detail panel widget — expanded view of the selected log record.

#[cfg(test)]
#[path = "detail_panel_widget_tests.rs"]
mod detail_panel_widget_tests;

use crate::app::App;
use crate::config::Theme;
use crate::ui::UiComponent;
use ratatui::layout::{Constraint, Layout, Rect};
use scouty::record::{ExpandedField, ExpandedValue};

use crate::ui::widgets::log_table_widget::level_style;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap};
use ratatui::Frame;
use std::collections::HashSet;

/// A flattened tree node for rendering/navigation.
#[derive(Debug, Clone)]
pub struct FlatNode {
    /// Indentation depth.
    pub depth: usize,
    /// Path key for collapse state (e.g. "0.Attributes.2").
    pub path_key: String,
    /// Display label.
    pub label: String,
    /// Display value (None for branch nodes).
    pub value: Option<String>,
    /// Whether this node is collapsible (has children).
    pub collapsible: bool,
    /// Whether this node is currently collapsed.
    pub collapsed: bool,
    /// Filter expression for quick filter (leaf nodes only).
    pub filter_expr: Option<String>,
}

/// Flatten expanded fields into a list of renderable nodes.
pub fn flatten_expanded(fields: &[ExpandedField], collapsed: &HashSet<String>) -> Vec<FlatNode> {
    let mut nodes = Vec::new();
    for (i, field) in fields.iter().enumerate() {
        let path = i.to_string();
        flatten_value(
            &field.label,
            &field.value,
            0,
            &path,
            collapsed,
            &mut nodes,
            &field.label,
        );
    }
    nodes
}

fn flatten_value(
    label: &str,
    value: &ExpandedValue,
    depth: usize,
    path: &str,
    collapsed: &HashSet<String>,
    nodes: &mut Vec<FlatNode>,
    filter_path: &str,
) {
    match value {
        ExpandedValue::Text(text) => {
            nodes.push(FlatNode {
                depth,
                path_key: path.to_string(),
                label: label.to_string(),
                value: Some(text.clone()),
                collapsible: false,
                collapsed: false,
                filter_expr: Some(format!(
                    "{} == \"{}\"",
                    filter_path,
                    text.replace('"', "\\\"")
                )),
            });
        }
        ExpandedValue::KeyValue(pairs) => {
            let is_collapsed = collapsed.contains(path);
            nodes.push(FlatNode {
                depth,
                path_key: path.to_string(),
                label: label.to_string(),
                value: None,
                collapsible: true,
                collapsed: is_collapsed,
                filter_expr: None,
            });
            if !is_collapsed {
                for (i, (key, child)) in pairs.iter().enumerate() {
                    let child_path = format!("{}.{}", path, i);
                    let child_filter = format!("{}.{}", filter_path, key);
                    flatten_value(
                        key,
                        child,
                        depth + 1,
                        &child_path,
                        collapsed,
                        nodes,
                        &child_filter,
                    );
                }
            }
        }
        ExpandedValue::List(items) => {
            let is_collapsed = collapsed.contains(path);
            nodes.push(FlatNode {
                depth,
                path_key: path.to_string(),
                label: label.to_string(),
                value: None,
                collapsible: true,
                collapsed: is_collapsed,
                filter_expr: None,
            });
            if !is_collapsed {
                for (i, child) in items.iter().enumerate() {
                    let child_path = format!("{}.{}", path, i);
                    let child_label = format!("[{}]", i);
                    let child_filter = format!("{}[{}]", filter_path, i);
                    flatten_value(
                        &child_label,
                        child,
                        depth + 1,
                        &child_path,
                        collapsed,
                        nodes,
                        &child_filter,
                    );
                }
            }
        }
    }
}

/// Count the number of field rows that would be displayed for a record,
/// without allocating the full field pairs vector.
pub(crate) fn field_count(record: &scouty::record::LogRecord) -> usize {
    let mut count: usize = 3;
    if record.hostname.is_some() {
        count += 1;
    }
    if record.container.is_some() {
        count += 1;
    }
    if record.context.is_some() {
        count += 1;
    }
    if record.function.is_some() {
        count += 1;
    }
    if record.component_name.is_some() {
        count += 1;
    }
    if record.process_name.is_some() {
        count += 1;
    }
    if record.pid.is_some() {
        count += 1;
    }
    if record.tid.is_some() {
        count += 1;
    }
    if let Some(meta) = record.metadata.as_ref() {
        count += meta.len();
    }
    count
}

/// Build field key-value pairs for the right pane.
fn build_field_pairs(record: &scouty::record::LogRecord) -> Vec<(&'static str, String)> {
    let mut pairs = vec![
        (
            "Timestamp",
            record.timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
        ),
        (
            "Level",
            record
                .level
                .map(|l| l.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ),
        ("Source", record.source.to_string()),
    ];

    let optional_fields: Vec<(&str, Option<String>)> = vec![
        ("Hostname", record.hostname.clone()),
        ("Container", record.container.clone()),
        ("Context", record.context.clone()),
        ("Function", record.function.clone()),
        ("Component", record.component_name.clone()),
        ("Process", record.process_name.clone()),
        ("PID", record.pid.map(|p| p.to_string())),
        ("TID", record.tid.map(|t| t.to_string())),
    ];

    for (label, value) in optional_fields {
        if let Some(val) = value {
            pairs.push((label, val));
        }
    }

    if record.metadata.as_ref().is_some_and(|m| !m.is_empty()) {
        for (k, v) in record.metadata.as_ref().unwrap() {
            pairs.push(("Meta", format!("{} = {}", k, v)));
        }
    }

    pairs
}

/// Build Line spans from field pairs (for single-column fallback).
fn build_field_lines(record: &scouty::record::LogRecord, theme: &Theme) -> Vec<Line<'static>> {
    let label_style = theme.detail_panel.field_name.to_style();
    build_field_pairs(record)
        .into_iter()
        .map(|(key, val)| {
            let padded_key = format!("{:<11}", format!("{}:", key));
            Line::from(vec![Span::styled(padded_key, label_style), Span::raw(val)])
        })
        .collect()
}

pub struct DetailPanelWidget;

/// Minimum total width to show split layout.
const MIN_SPLIT_WIDTH: u16 = 40;
/// Max value display length before truncation.
const MAX_VALUE_LEN: usize = 60;

impl DetailPanelWidget {
    pub fn render_with_app(&self, frame: &mut Frame, area: Rect, app: &App) {
        let theme = &app.theme;
        let block = Block::default()
            .title(" Detail ")
            .borders(Borders::TOP)
            .border_style(theme.detail_panel.border.to_style());

        let Some(record) = app.selected_record() else {
            let empty = Paragraph::new("No record selected").block(block);
            frame.render_widget(empty, area);
            return;
        };

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.width < MIN_SPLIT_WIDTH {
            self.render_single_column(frame, inner, record, theme);
        } else {
            self.render_split(frame, inner, record, app);
        }
    }

    fn render_split(
        &self,
        frame: &mut Frame,
        area: Rect,
        record: &scouty::record::LogRecord,
        app: &App,
    ) {
        let theme = &app.theme;
        let chunks = Layout::horizontal([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);

        // Left pane: tree or raw text
        let has_expanded = record.expanded.as_ref().is_some_and(|e| !e.is_empty());

        let left_title = if has_expanded {
            " Expanded "
        } else {
            " Log Content "
        };
        let left_border_style = if app.detail_tree_focus && has_expanded {
            theme
                .detail_panel
                .border
                .to_style()
                .add_modifier(Modifier::BOLD)
        } else {
            theme.detail_panel.border.to_style()
        };
        let left_block = Block::default()
            .title(left_title)
            .borders(Borders::RIGHT)
            .border_style(left_border_style);

        if has_expanded {
            let expanded = record.expanded.as_ref().unwrap();
            let flat = flatten_expanded(expanded, &app.detail_tree_collapsed);
            let inner_left = left_block.inner(chunks[0]);
            frame.render_widget(left_block, chunks[0]);
            self.render_tree(
                frame,
                inner_left,
                &flat,
                app.detail_tree_cursor,
                app.detail_tree_focus,
                theme,
            );
        } else {
            // Show message field (not raw line — raw duplicates fields already in right pane).
            // Fall back to raw only if message is empty.
            let content = if record.message.is_empty() {
                record.raw.clone()
            } else {
                record.message.clone()
            };
            let raw_text = Paragraph::new(content)
                .block(left_block)
                .wrap(Wrap { trim: false });
            frame.render_widget(raw_text, chunks[0]);
        }

        // Right pane: fields table
        let pairs = build_field_pairs(record);
        let label_style = theme.detail_panel.field_name.to_style();
        let rows: Vec<Row> = pairs
            .into_iter()
            .map(|(key, val)| {
                let val_cell = if key == "Level" {
                    Cell::from(Span::styled(val, level_style(record.level, theme)))
                } else {
                    Cell::from(val)
                };
                Row::new(vec![Cell::from(Span::styled(key, label_style)), val_cell])
            })
            .collect();
        let right_block = Block::default()
            .title(" Fields ")
            .border_style(theme.detail_panel.border.to_style());
        let table = Table::new(rows, [Constraint::Length(11), Constraint::Fill(1)])
            .column_spacing(1)
            .block(right_block);
        frame.render_widget(table, chunks[1]);
    }

    fn render_tree(
        &self,
        frame: &mut Frame,
        area: Rect,
        nodes: &[FlatNode],
        cursor: usize,
        focused: bool,
        theme: &Theme,
    ) {
        let visible_rows = area.height as usize;
        if nodes.is_empty() || visible_rows == 0 {
            return;
        }

        // Scroll to keep cursor visible
        let scroll_offset = if cursor >= visible_rows {
            cursor - visible_rows + 1
        } else {
            0
        };

        let mut lines = Vec::with_capacity(visible_rows);
        let width = area.width as usize;

        for (i, node) in nodes
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(visible_rows)
        {
            let indent = "  ".repeat(node.depth);
            let is_selected = i == cursor && focused;

            let indicator = if node.collapsible {
                if node.collapsed {
                    "▶ "
                } else {
                    "▼ "
                }
            } else {
                "  "
            };

            let line_text = if let Some(ref val) = node.value {
                let truncated = if val.len() > MAX_VALUE_LEN {
                    format!("{}…", &val[..MAX_VALUE_LEN])
                } else {
                    val.clone()
                };
                format!("{}{}{}: {}", indent, indicator, node.label, truncated)
            } else {
                format!("{}{}{}", indent, indicator, node.label)
            };

            // Pad/truncate to width
            let display = if line_text.len() > width {
                format!("{}…", &line_text[..width.saturating_sub(1)])
            } else {
                line_text
            };

            let style = if is_selected {
                theme
                    .detail_panel
                    .field_name
                    .to_style()
                    .add_modifier(Modifier::REVERSED)
            } else if node.collapsible {
                theme.detail_panel.section_header.to_style()
            } else {
                theme.detail_panel.field_value.to_style()
            };

            lines.push(Line::styled(display, style));
        }

        let para = Paragraph::new(lines);
        frame.render_widget(para, area);
    }

    fn render_single_column(
        &self,
        frame: &mut Frame,
        area: Rect,
        record: &scouty::record::LogRecord,
        theme: &Theme,
    ) {
        let mut lines = build_field_lines(record, theme);
        lines.push(Line::from(""));

        // Show log content section
        let has_expanded = record.expanded.as_ref().is_some_and(|e| !e.is_empty());
        if has_expanded {
            lines.push(Line::styled(
                "Expanded:",
                theme.detail_panel.section_header.to_style(),
            ));
            // Show a flat summary of expanded fields
            if let Some(expanded) = &record.expanded {
                for field in expanded {
                    let summary = match &field.value {
                        scouty::record::ExpandedValue::Text(t) => {
                            format!("  {}: {}", field.label, t)
                        }
                        _ => format!("  {} (…)", field.label),
                    };
                    lines.push(Line::from(summary));
                }
            }
        } else {
            lines.push(Line::styled(
                "Log Content:",
                theme.detail_panel.section_header.to_style(),
            ));
            let content = if record.message.is_empty() {
                &record.raw
            } else {
                &record.message
            };
            lines.push(Line::from(content.clone()));
        }

        let detail = Paragraph::new(lines).wrap(Wrap { trim: false });
        frame.render_widget(detail, area);
    }
}

impl UiComponent for DetailPanelWidget {
    fn render(&self, _frame: &mut Frame, _area: Rect) {}

    fn enable_jk_navigation(&self) -> bool {
        false
    }
}

impl crate::panel::Panel for DetailPanelWidget {
    fn name(&self) -> &str {
        "Detail"
    }

    fn shortcut(&self) -> Option<char> {
        // Enter key opens the Detail panel (handled as KeyCode::Enter in main.rs)
        Some('\r')
    }

    fn default_height(&self) -> crate::panel::PanelHeight {
        crate::panel::PanelHeight::FitContent
    }

    fn is_available(&self) -> bool {
        // Detail panel is always available when there are records
        true
    }

    fn on_log_cursor_changed(&mut self, _index: usize) {
        // Detail panel content auto-updates via App::selected_record(),
        // so no explicit action is needed here.
    }
}
