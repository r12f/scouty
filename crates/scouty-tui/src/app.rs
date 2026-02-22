//! Application state for the TUI.

use scouty::filter::eval;
use scouty::filter::expr::{self, Expr};
use scouty::loader::file::FileLoader;
use scouty::parser::factory::ParserFactory;
use scouty::record::LogRecord;
use scouty::traits::LogLoader;
use std::sync::Arc;

/// Input mode for the TUI.
#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Filter,
    Search,
    TimeJump,
    GotoLine,
    QuickExclude,
    QuickInclude,
    FieldFilter,
    FilterManager,
    ColumnSelector,
    CopyFormat,
    Help,
}

/// Column identifiers for the log table.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Column {
    Time,
    Level,
    Hostname,
    Container,
    ProcessName,
    Pid,
    Tid,
    Component,
    Function,
    Context,
    Source,
    Log,
}

impl Column {
    #[allow(dead_code)]
    pub const ALL: [Column; 12] = [
        Column::Time,
        Column::Level,
        Column::Hostname,
        Column::Container,
        Column::ProcessName,
        Column::Pid,
        Column::Tid,
        Column::Component,
        Column::Function,
        Column::Context,
        Column::Source,
        Column::Log,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Column::Time => "Time",
            Column::Level => "Level",
            Column::Hostname => "Hostname",
            Column::Container => "Container",
            Column::ProcessName => "ProcessName",
            Column::Pid => "Pid",
            Column::Tid => "Tid",
            Column::Component => "Component",
            Column::Function => "Function",
            Column::Context => "Context",
            Column::Source => "Source",
            Column::Log => "Log",
        }
    }
}

/// Column visibility configuration.
#[derive(Debug, Clone)]
pub struct ColumnConfig {
    /// (Column, visible) for each column.
    pub columns: Vec<(Column, bool)>,
    /// Cursor in the column selector dialog.
    pub cursor: usize,
}

impl Default for ColumnConfig {
    fn default() -> Self {
        Self {
            columns: vec![
                (Column::Time, true),
                (Column::Level, false),
                (Column::Hostname, false),
                (Column::Container, false),
                (Column::ProcessName, false),
                (Column::Pid, false),
                (Column::Tid, false),
                (Column::Component, false),
                (Column::Function, false),
                (Column::Context, false),
                (Column::Source, false),
                (Column::Log, true),
            ],
            cursor: 0,
        }
    }
}

impl ColumnConfig {
    #[allow(dead_code)]
    pub fn is_visible(&self, col: Column) -> bool {
        self.columns
            .iter()
            .find(|(c, _)| *c == col)
            .map(|(_, v)| *v)
            .unwrap_or(false)
    }

    pub fn visible_columns(&self) -> Vec<Column> {
        self.columns
            .iter()
            .filter(|(_, v)| *v)
            .map(|(c, _)| *c)
            .collect()
    }

    #[allow(dead_code)]
    pub fn toggle(&mut self, index: usize) {
        if index < self.columns.len() {
            // Don't allow hiding Log column
            if self.columns[index].0 == Column::Log {
                return;
            }
            self.columns[index].1 = !self.columns[index].1;
        }
    }
}

/// A single filter entry in the filter stack.
#[derive(Debug, Clone)]
pub struct FilterEntry {
    /// Human-readable label (e.g. "exclude: timeout", "level == ERROR").
    pub label: String,
    /// The compiled expression.
    pub expr: Expr,
    /// Whether this is an exclude (true) or include (false) filter.
    pub exclude: bool,
}

/// Kind of field filter entry.
#[derive(Clone, Debug, PartialEq)]
pub enum FieldEntryKind {
    /// Regular field: generates `field = "value"`.
    Field,
    /// Time before: generates `timestamp < "rfc3339"` (exclude) or `<= "rfc3339"` (include).
    TimeBefore { rfc3339: String },
    /// Time after: generates `timestamp > "rfc3339"` (exclude) or `>= "rfc3339"` (include).
    TimeAfter { rfc3339: String },
}

/// A single entry in the field filter dialog.
#[derive(Clone, Debug)]
pub struct FieldEntry {
    /// Display name shown in dialog.
    pub name: String,
    /// Display value shown in dialog.
    pub value: String,
    /// Whether this entry is selected.
    pub checked: bool,
    /// Kind of entry (determines filter generation).
    pub kind: FieldEntryKind,
}

#[derive(Clone)]
pub struct FieldFilterState {
    /// Available fields from the selected record.
    pub fields: Vec<FieldEntry>,
    /// Current cursor in the field list.
    pub cursor: usize,
    /// Whether we're in Exclude (true) or Include (false) mode.
    pub exclude: bool,
    /// Whether to combine with OR (true) or AND (false). Default: OR.
    pub logic_or: bool,
}

/// Main application state.
pub struct App {
    /// All log records loaded from the file.
    pub records: Vec<Arc<LogRecord>>,
    /// Total records before filtering.
    pub total_records: usize,
    /// Filtered indices into records.
    pub filtered_indices: Vec<usize>,
    /// Current scroll offset (index into filtered list).
    pub scroll_offset: usize,
    /// Index of selected record in filtered list.
    pub selected: usize,
    /// Number of visible rows in the log list (updated by render).
    pub visible_rows: usize,
    /// Whether the detail panel is open.
    pub detail_open: bool,
    /// Current input mode.
    pub input_mode: InputMode,
    /// Filter input buffer (for expression mode).
    pub filter_input: String,
    /// Filter error message.
    pub filter_error: Option<String>,
    /// Stack of active filters.
    pub filters: Vec<FilterEntry>,
    /// Quick exclude/include input buffer.
    pub quick_filter_input: String,
    /// Field filter dialog state.
    pub field_filter: Option<FieldFilterState>,
    /// Filter manager: selected index.
    pub filter_manager_cursor: usize,
    /// Search input buffer.
    pub search_input: String,
    /// Current search matches (indices into filtered list).
    pub search_matches: Vec<usize>,
    /// Current search match index.
    pub search_match_idx: Option<usize>,
    /// Time jump input buffer.
    pub time_input: String,
    /// Goto line input buffer.
    pub goto_input: String,
    /// Status message shown temporarily.
    pub status_message: Option<String>,
    /// Timestamp when status_message was set (for auto-clear).
    pub status_message_at: Option<std::time::Instant>,
    /// Column widths computed from data (Time, Level, ProcessName, Pid, Tid, Component).
    pub col_widths: [u16; 6],
    /// Column visibility configuration.
    pub column_config: ColumnConfig,
    /// Follow mode: auto-scroll to bottom.
    pub follow_mode: bool,
}

impl App {
    /// Load log records from a file.
    pub fn load_files(paths: &[&str]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut store = scouty::store::LogStore::new();
        let mut record_id: u64 = 0;

        for path in paths {
            let mut loader = FileLoader::new(path, false);
            let lines = loader.load()?;
            let info = loader.info().clone();

            let group = ParserFactory::create_parser_group(&info);

            for line in lines.into_iter() {
                if let Some(mut record) = group.parse(&line, &info.id, &info.id, record_id) {
                    record.raw = line;
                    store.insert(record);
                    record_id += 1;
                }
            }
        }

        // Ensure out-of-order records are merged before iterating
        store.compact_ooo();
        let records: Vec<Arc<LogRecord>> = store.iter_arc().cloned().collect();
        let total_records = records.len();
        let filtered_indices: Vec<usize> = (0..records.len()).collect();

        let col_widths = Self::compute_col_widths(&records, &filtered_indices);

        Ok(Self {
            records,
            total_records,
            filtered_indices,
            scroll_offset: 0,
            selected: 0,
            visible_rows: 20,
            detail_open: false,
            input_mode: InputMode::Normal,
            filter_input: String::new(),
            filter_error: None,
            filters: Vec::new(),
            quick_filter_input: String::new(),
            field_filter: None,
            filter_manager_cursor: 0,
            search_input: String::new(),
            search_matches: vec![],
            search_match_idx: None,
            time_input: String::new(),
            goto_input: String::new(),
            status_message: None,
            status_message_at: None,
            col_widths,
            column_config: ColumnConfig::default(),
            follow_mode: false,
        })
    }

    /// Compute auto-fit column widths by sampling records.
    fn compute_col_widths(records: &[Arc<LogRecord>], indices: &[usize]) -> [u16; 6] {
        let mut widths: [u16; 6] = [4, 5, 11, 3, 3, 9];
        let max_widths: [u16; 6] = [23, 5, 20, 8, 8, 20];

        let sample_size = 1000.min(indices.len());
        let step = if sample_size == 0 {
            1
        } else {
            indices.len().max(1) / sample_size.max(1)
        }
        .max(1);

        for i in (0..indices.len()).step_by(step) {
            let r = &records[indices[i]];
            widths[0] = widths[0].max(19);
            if let Some(level) = r.level {
                widths[1] = widths[1].max(format!("{}", level).len() as u16);
            }
            if let Some(ref name) = r.process_name {
                widths[2] = widths[2].max((name.len() as u16).min(max_widths[2]));
            }
            if let Some(pid) = r.pid {
                widths[3] = widths[3].max((format!("{}", pid).len() as u16).min(max_widths[3]));
            }
            if let Some(tid) = r.tid {
                widths[4] = widths[4].max((format!("{}", tid).len() as u16).min(max_widths[4]));
            }
            if let Some(ref comp) = r.component_name {
                widths[5] = widths[5].max((comp.len() as u16).min(max_widths[5]));
            }
        }

        for i in 0..6 {
            widths[i] = widths[i].min(max_widths[i]);
        }
        widths
    }

    /// Total filtered record count.
    pub fn total(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Set a status message with auto-clear timestamp.
    pub fn set_status(&mut self, msg: String) {
        self.status_message = Some(msg);
        self.status_message_at = Some(std::time::Instant::now());
    }

    /// Clear status message if it has been displayed for >= 3 seconds.
    pub fn tick_status_clear(&mut self) {
        if let Some(at) = self.status_message_at {
            if at.elapsed() >= std::time::Duration::from_secs(3) {
                self.status_message = None;
                self.status_message_at = None;
            }
        }
    }

    /// Clear status message immediately (on keypress).
    pub fn clear_status(&mut self) {
        self.status_message = None;
        self.status_message_at = None;
    }

    // ── Filter application ──────────────────────────────────────

    /// Re-apply all active filters to compute filtered_indices.
    pub fn reapply_filters(&mut self) {
        self.filtered_indices = (0..self.records.len())
            .filter(|&i| {
                let record = &self.records[i];
                for f in &self.filters {
                    let matches = eval::eval(&f.expr, record);
                    if f.exclude && matches {
                        return false; // exclude filter matched → hide
                    }
                    if !f.exclude && !matches {
                        return false; // include filter didn't match → hide
                    }
                }
                true
            })
            .collect();

        self.col_widths = Self::compute_col_widths(&self.records, &self.filtered_indices);
        self.scroll_offset = 0;
        self.selected = 0;
        self.clear_search();
    }

    /// Apply filter expression from the `f` input mode.
    pub fn apply_filter(&mut self) {
        if self.filter_input.is_empty() {
            self.filter_error = None;
            return;
        }
        match expr::parse(&self.filter_input) {
            Ok(parsed_expr) => {
                self.filters.push(FilterEntry {
                    label: self.filter_input.clone(),
                    expr: parsed_expr,
                    exclude: false,
                });
                self.filter_error = None;
                self.filter_input.clear();
                self.reapply_filters();
            }
            Err(e) => {
                self.filter_error = Some(e);
            }
        }
    }

    /// Add a quick exclude filter (message contains text).
    pub fn apply_quick_exclude(&mut self) {
        let text = self.quick_filter_input.trim().to_string();
        if text.is_empty() {
            return;
        }
        let expr_str = format!("message contains \"{}\"", text.replace('"', "\\\""));
        match expr::parse(&expr_str) {
            Ok(parsed_expr) => {
                self.filters.push(FilterEntry {
                    label: format!("exclude: {}", text),
                    expr: parsed_expr,
                    exclude: true,
                });
                self.quick_filter_input.clear();
                self.reapply_filters();
            }
            Err(e) => {
                self.set_status(format!("Filter error: {}", e));
            }
        }
    }

    /// Add a quick include filter (message contains text).
    pub fn apply_quick_include(&mut self) {
        let text = self.quick_filter_input.trim().to_string();
        if text.is_empty() {
            return;
        }
        let expr_str = format!("message contains \"{}\"", text.replace('"', "\\\""));
        match expr::parse(&expr_str) {
            Ok(parsed_expr) => {
                self.filters.push(FilterEntry {
                    label: format!("include: {}", text),
                    expr: parsed_expr,
                    exclude: false,
                });
                self.quick_filter_input.clear();
                self.reapply_filters();
            }
            Err(e) => {
                self.set_status(format!("Filter error: {}", e));
            }
        }
    }

    /// Open field filter dialog based on selected record.
    /// `exclude` determines initial mode (Ctrl+- = true, Ctrl+= = false).
    pub fn open_field_filter(&mut self, exclude: bool) {
        if let Some(record) = self.selected_record().cloned() {
            let mut fields: Vec<FieldEntry> = Vec::new();

            // Time range options at the top
            let ts_rfc3339 = record.timestamp.to_rfc3339();
            let ts_display = record.timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string();
            fields.push(FieldEntry {
                name: format!("Before {}", ts_display),
                value: String::new(),
                checked: false,
                kind: FieldEntryKind::TimeBefore {
                    rfc3339: ts_rfc3339.clone(),
                },
            });
            fields.push(FieldEntry {
                name: format!("After {}", ts_display),
                value: String::new(),
                checked: false,
                kind: FieldEntryKind::TimeAfter {
                    rfc3339: ts_rfc3339.clone(),
                },
            });

            // Helper to push a regular field entry
            let mut push_field = |name: &str, val: String| {
                fields.push(FieldEntry {
                    name: name.to_string(),
                    value: val,
                    checked: false,
                    kind: FieldEntryKind::Field,
                });
            };

            // ALL fields from LogRecord — use RFC3339 for timestamp
            push_field("timestamp", ts_rfc3339);
            if let Some(level) = record.level {
                push_field("level", format!("{}", level));
            }
            push_field("source", record.source.to_string());
            if let Some(ref name) = record.hostname {
                push_field("hostname", name.clone());
            }
            if let Some(ref name) = record.container {
                push_field("container", name.clone());
            }
            if let Some(ref ctx) = record.context {
                push_field("context", ctx.clone());
            }
            if let Some(ref func) = record.function {
                push_field("function", func.clone());
            }
            if let Some(ref name) = record.process_name {
                push_field("process_name", name.clone());
            }
            if let Some(pid) = record.pid {
                push_field("pid", pid.to_string());
            }
            if let Some(tid) = record.tid {
                push_field("tid", tid.to_string());
            }
            if let Some(ref comp) = record.component_name {
                push_field("component", comp.clone());
            }
            push_field("message", record.message.clone());

            // Include metadata fields
            if let Some(ref meta) = record.metadata {
                for (k, v) in meta {
                    fields.push(FieldEntry {
                        name: k.clone(),
                        value: v.clone(),
                        checked: false,
                        kind: FieldEntryKind::Field,
                    });
                }
            }

            self.field_filter = Some(FieldFilterState {
                fields,
                cursor: 0,
                exclude,
                logic_or: true, // default OR
            });
            self.input_mode = InputMode::FieldFilter;
        } else {
            self.set_status("No record selected".to_string());
        }
    }

    /// Apply the field filter dialog selections.
    pub fn apply_field_filter(&mut self) {
        let state = match self.field_filter.clone() {
            Some(s) => s,
            None => return,
        };
        let mut time_parts: Vec<String> = Vec::new();
        let mut field_parts: Vec<String> = Vec::new();

        for entry in &state.fields {
            if !entry.checked {
                continue;
            }

            match &entry.kind {
                FieldEntryKind::TimeBefore { rfc3339 } => {
                    if state.exclude {
                        time_parts.push(format!("timestamp < \"{}\"", rfc3339));
                    } else {
                        time_parts.push(format!("timestamp <= \"{}\"", rfc3339));
                    }
                }
                FieldEntryKind::TimeAfter { rfc3339 } => {
                    if state.exclude {
                        time_parts.push(format!("timestamp > \"{}\"", rfc3339));
                    } else {
                        time_parts.push(format!("timestamp >= \"{}\"", rfc3339));
                    }
                }
                FieldEntryKind::Field => {
                    field_parts.push(format!(
                        "{} = \"{}\"",
                        entry.name,
                        entry.value.replace('"', "\\\"")
                    ));
                }
            }
        }

        // Time filters are always emitted as separate FilterEntry items (AND with stack)
        // Field filters use the user-selected joiner (OR/AND)
        let has_time = !time_parts.is_empty();
        let has_fields = !field_parts.is_empty();

        if !has_time && !has_fields {
            self.set_status("No fields selected".to_string());
            return;
        }

        let mut ok = true;

        // Emit each time filter as a separate entry
        for part in &time_parts {
            match expr::parse(part) {
                Ok(parsed_expr) => {
                    let label = if state.exclude {
                        format!("exclude: {}", part)
                    } else {
                        format!("include: {}", part)
                    };
                    self.filters.push(FilterEntry {
                        label,
                        expr: parsed_expr,
                        exclude: state.exclude,
                    });
                }
                Err(e) => {
                    self.set_status(format!("Filter error: {}", e));
                    ok = false;
                }
            }
        }

        // Emit field filters as a combined expression
        if has_fields {
            let joiner = if state.logic_or { " OR " } else { " AND " };
            let expr_str = field_parts.join(joiner);
            let label = if state.exclude {
                format!("exclude: {}", expr_str)
            } else {
                format!("include: {}", expr_str)
            };

            match expr::parse(&expr_str) {
                Ok(parsed_expr) => {
                    self.filters.push(FilterEntry {
                        label,
                        expr: parsed_expr,
                        exclude: state.exclude,
                    });
                }
                Err(e) => {
                    self.set_status(format!("Filter error: {}", e));
                    ok = false;
                }
            }
        }

        if ok {
            self.field_filter = None;
            self.input_mode = InputMode::Normal;
            self.reapply_filters();
        }
    }

    /// Remove a filter by index.
    pub fn remove_filter(&mut self, index: usize) {
        if index < self.filters.len() {
            self.filters.remove(index);
            self.reapply_filters();
        }
    }

    /// Clear all filters.
    pub fn clear_filters(&mut self) {
        self.filters.clear();
        self.reapply_filters();
    }

    // ── Search ──────────────────────────────────────────────────

    /// Execute regex search across filtered records.
    pub fn execute_search(&mut self) {
        if self.search_input.is_empty() {
            self.clear_search();
            return;
        }
        let pattern = match regex::RegexBuilder::new(&self.search_input)
            .case_insensitive(true)
            .build()
        {
            Ok(re) => re,
            Err(e) => {
                self.search_matches.clear();
                self.search_match_idx = None;
                self.set_status(format!("Invalid regex: {}", e));
                return;
            }
        };

        self.search_matches = self
            .filtered_indices
            .iter()
            .enumerate()
            .filter(|(_, &ri)| {
                pattern.is_match(&self.records[ri].message)
                    || pattern.is_match(&self.records[ri].raw)
            })
            .map(|(i, _)| i)
            .collect();

        if self.search_matches.is_empty() {
            self.search_match_idx = None;
            self.set_status("No matches found".to_string());
        } else {
            let idx = self
                .search_matches
                .iter()
                .position(|&m| m >= self.selected)
                .unwrap_or(0);
            self.search_match_idx = Some(idx);
            self.jump_to_search_match();
        }
    }

    pub fn next_search_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        if let Some(idx) = self.search_match_idx {
            self.search_match_idx = Some((idx + 1) % self.search_matches.len());
            self.jump_to_search_match();
        }
    }

    pub fn prev_search_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        if let Some(idx) = self.search_match_idx {
            let prev = if idx == 0 {
                self.search_matches.len() - 1
            } else {
                idx - 1
            };
            self.search_match_idx = Some(prev);
            self.jump_to_search_match();
        }
    }

    fn jump_to_search_match(&mut self) {
        if let Some(idx) = self.search_match_idx {
            let target = self.search_matches[idx];
            self.selected = target;
            self.ensure_selected_visible();
            self.set_status(format!("Match {}/{}", idx + 1, self.search_matches.len()));
        }
    }

    fn clear_search(&mut self) {
        self.search_matches.clear();
        self.search_match_idx = None;
    }

    // ── Navigation ──────────────────────────────────────────────

    pub fn jump_to_time(&mut self) {
        use chrono::NaiveTime;
        let input = self.time_input.trim();
        if input.is_empty() {
            return;
        }

        if let Ok(time) = NaiveTime::parse_from_str(input, "%H:%M:%S") {
            for (fi, &ri) in self.filtered_indices.iter().enumerate() {
                if self.records[ri].timestamp.time() >= time {
                    self.selected = fi;
                    self.ensure_selected_visible();
                    self.set_status(format!("Jumped to {}", time));
                    return;
                }
            }
            self.set_status("No record at or after that time".to_string());
            return;
        }

        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S") {
            let dt_utc = dt.and_utc();
            for (fi, &ri) in self.filtered_indices.iter().enumerate() {
                if self.records[ri].timestamp >= dt_utc {
                    self.selected = fi;
                    self.ensure_selected_visible();
                    self.set_status(format!("Jumped to {}", dt_utc));
                    return;
                }
            }
            self.set_status("No record at or after that time".to_string());
            return;
        }

        self.status_message =
            Some("Invalid time format (use HH:MM:SS or YYYY-MM-DD HH:MM:SS)".to_string());
    }

    pub fn goto_line(&mut self) {
        let input = self.goto_input.trim();
        if input.is_empty() {
            return;
        }
        match input.parse::<usize>() {
            Ok(line) if line >= 1 && line <= self.total() => {
                self.selected = line - 1;
                self.ensure_selected_visible();
                self.set_status(format!("Line {}", line));
            }
            Ok(line) if line > self.total() => {
                self.scroll_to_bottom();
                self.set_status(format!("Line {} (clamped to {})", line, self.total()));
            }
            _ => {
                self.set_status("Invalid line number".to_string());
            }
        }
    }

    pub fn select_down(&mut self, n: usize) {
        let max = self.total().saturating_sub(1);
        self.selected = (self.selected + n).min(max);
        self.ensure_selected_visible();
    }

    pub fn select_up(&mut self, n: usize) {
        self.selected = self.selected.saturating_sub(n);
        self.ensure_selected_visible();
        if n > 0 && self.selected < self.total().saturating_sub(1) {
            self.exit_follow();
        }
    }

    pub fn page_down(&mut self) {
        self.select_down((self.visible_rows / 2).max(1));
    }

    pub fn page_up(&mut self) {
        self.select_up((self.visible_rows / 2).max(1));
    }

    pub fn scroll_to_top(&mut self) {
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        if self.total() > 0 {
            self.selected = self.total() - 1;
        }
        self.scroll_offset = self.total().saturating_sub(self.visible_rows);
    }

    pub fn toggle_detail(&mut self) {
        self.detail_open = !self.detail_open;
    }

    /// Toggle follow mode.
    pub fn toggle_follow(&mut self) {
        self.follow_mode = !self.follow_mode;
        if self.follow_mode {
            self.scroll_to_bottom();
        }
    }

    /// Exit follow mode (called on manual scroll up).
    pub fn exit_follow(&mut self) {
        self.follow_mode = false;
    }

    fn ensure_selected_visible(&mut self) {
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + self.visible_rows {
            self.scroll_offset = self.selected.saturating_sub(self.visible_rows - 1);
        }
    }

    pub fn visible_records(&self) -> Vec<&LogRecord> {
        let end = (self.scroll_offset + self.visible_rows).min(self.total());
        self.filtered_indices[self.scroll_offset..end]
            .iter()
            .map(|&i| self.records[i].as_ref())
            .collect()
    }

    pub fn selected_record(&self) -> Option<&LogRecord> {
        self.filtered_indices
            .get(self.selected)
            .map(|&i| self.records[i].as_ref())
    }

    pub fn is_search_match(&self, filtered_idx: usize) -> bool {
        self.search_matches.contains(&filtered_idx)
    }

    /// Copy the selected record's raw text to clipboard via OSC 52.
    pub fn copy_raw(&mut self) -> Option<String> {
        if let Some(record) = self.selected_record() {
            let text = record.raw.clone();
            self.set_status("Copied raw log to clipboard".to_string());
            Some(text)
        } else {
            self.set_status("No record selected".to_string());
            None
        }
    }

    /// Copy the selected record in the given format to clipboard via OSC 52.
    pub fn copy_as_format(&mut self, format: CopyFormat) -> Option<String> {
        if let Some(record) = self.selected_record() {
            let label = match format {
                CopyFormat::Raw => "raw",
                CopyFormat::Json => "JSON",
                CopyFormat::Yaml => "YAML",
            };
            let (text, ok) = match format {
                CopyFormat::Raw => (record.raw.clone(), true),
                CopyFormat::Json => match serde_json::to_string_pretty(record) {
                    Ok(json) => (json, true),
                    Err(_) => (record.raw.clone(), false),
                },
                CopyFormat::Yaml => match serde_yaml::to_string(record) {
                    Ok(yaml) => (yaml, true),
                    Err(_) => (record.raw.clone(), false),
                },
            };
            self.status_message = Some(if ok {
                format!("Copied as {} to clipboard", label)
            } else {
                format!("{} serialization failed; copied raw log instead", label)
            });
            self.input_mode = InputMode::Normal;
            Some(text)
        } else {
            self.set_status("No record selected".to_string());
            None
        }
    }
}

/// Copy format options.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CopyFormat {
    Raw,
    Json,
    Yaml,
}

/// Write text to system clipboard via OSC 52 escape sequence.
/// Works in most modern terminals including over SSH.
pub fn osc52_copy(text: &str) {
    use base64::Engine;
    use std::io::Write;
    let encoded = base64::engine::general_purpose::STANDARD.encode(text);
    // OSC 52 ; c ; <base64> ST
    print!("\x1b]52;c;{}\x07", encoded);
    let _ = std::io::stdout().flush();
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use scouty::record::{LogLevel, LogRecord};

    fn make_record(id: u64, level: Option<LogLevel>, message: &str) -> LogRecord {
        LogRecord {
            id,
            timestamp: Utc::now(),
            level,
            source: "test".into(),
            pid: None,
            tid: None,
            component_name: None,
            process_name: None,
            message: message.to_string(),
            hostname: None,
            container: None,
            context: None,
            function: None,
            raw: message.to_string(),
            metadata: None,
            loader_id: "test".into(),
        }
    }

    fn make_app(n: usize) -> App {
        let records: Vec<Arc<LogRecord>> = (0..n)
            .map(|i| {
                Arc::new(make_record(
                    i as u64,
                    Some(LogLevel::Info),
                    &format!("msg {}", i),
                ))
            })
            .collect();
        let filtered_indices = (0..n).collect();
        App {
            records,
            total_records: n,
            filtered_indices,
            scroll_offset: 0,
            selected: 0,
            visible_rows: 10,
            detail_open: false,
            input_mode: InputMode::Normal,
            filter_input: String::new(),
            filter_error: None,
            filters: Vec::new(),
            quick_filter_input: String::new(),
            field_filter: None,
            filter_manager_cursor: 0,
            search_input: String::new(),
            search_matches: vec![],
            search_match_idx: None,
            time_input: String::new(),
            goto_input: String::new(),
            status_message: None,
            status_message_at: None,
            col_widths: [19, 5, 11, 3, 3, 9],
            column_config: ColumnConfig::default(),
            follow_mode: false,
        }
    }

    fn make_app_with_levels(messages: &[(&str, Option<LogLevel>)]) -> App {
        let records: Vec<Arc<LogRecord>> = messages
            .iter()
            .enumerate()
            .map(|(i, (msg, level))| Arc::new(make_record(i as u64, *level, msg)))
            .collect();
        let n = records.len();
        let filtered_indices = (0..n).collect();
        App {
            records,
            total_records: n,
            filtered_indices,
            scroll_offset: 0,
            selected: 0,
            visible_rows: 10,
            detail_open: false,
            input_mode: InputMode::Normal,
            filter_input: String::new(),
            filter_error: None,
            filters: Vec::new(),
            quick_filter_input: String::new(),
            field_filter: None,
            filter_manager_cursor: 0,
            search_input: String::new(),
            search_matches: vec![],
            search_match_idx: None,
            time_input: String::new(),
            goto_input: String::new(),
            status_message: None,
            status_message_at: None,
            col_widths: [19, 5, 11, 3, 3, 9],
            column_config: ColumnConfig::default(),
            follow_mode: false,
        }
    }

    #[test]
    fn test_select_down_up() {
        let mut app = make_app(100);
        app.select_down(5);
        assert_eq!(app.selected, 5);
        app.select_up(3);
        assert_eq!(app.selected, 2);
    }

    #[test]
    fn test_select_clamped() {
        let mut app = make_app(10);
        app.select_down(50);
        assert_eq!(app.selected, 9);
        app.select_up(50);
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_page_down_up() {
        let mut app = make_app(100);
        app.page_down();
        assert_eq!(app.selected, 5);
        app.page_up();
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_scroll_to_top_bottom() {
        let mut app = make_app(100);
        app.scroll_to_bottom();
        assert_eq!(app.selected, 99);
        assert_eq!(app.scroll_offset, 90);
        app.scroll_to_top();
        assert_eq!(app.selected, 0);
        assert_eq!(app.scroll_offset, 0);
    }

    #[test]
    fn test_toggle_detail() {
        let mut app = make_app(10);
        assert!(!app.detail_open);
        app.toggle_detail();
        assert!(app.detail_open);
        app.toggle_detail();
        assert!(!app.detail_open);
    }

    #[test]
    fn test_empty_records() {
        let mut app = make_app(0);
        app.select_down(1);
        assert_eq!(app.selected, 0);
        app.scroll_to_bottom();
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_search() {
        let mut app = make_app(20);
        Arc::get_mut(&mut app.records[5]).unwrap().message = "error happened".to_string();
        Arc::get_mut(&mut app.records[15]).unwrap().message = "another error".to_string();
        app.search_input = "error".to_string();
        app.execute_search();
        assert_eq!(app.search_matches.len(), 2);
        assert_eq!(app.selected, 5);

        app.next_search_match();
        assert_eq!(app.selected, 15);

        app.next_search_match();
        assert_eq!(app.selected, 5);

        app.prev_search_match();
        assert_eq!(app.selected, 15);
    }

    #[test]
    fn test_search_no_match() {
        let mut app = make_app(10);
        app.search_input = "zzzznotfound".to_string();
        app.execute_search();
        assert!(app.search_matches.is_empty());
        assert!(app.status_message.is_some());
    }

    #[test]
    fn test_search_regex() {
        let mut app = make_app(20);
        Arc::get_mut(&mut app.records[3]).unwrap().message =
            "ERROR: connection timeout".to_string();
        Arc::get_mut(&mut app.records[7]).unwrap().message = "error: disk full".to_string();

        app.search_input = r"error.*(?:timeout|full)".to_string();
        app.execute_search();
        assert_eq!(app.search_matches.len(), 2);

        app.search_input = "[invalid".to_string();
        app.execute_search();
        assert!(app.search_matches.is_empty());
        assert!(app
            .status_message
            .as_ref()
            .unwrap()
            .contains("Invalid regex"));
    }

    #[test]
    fn test_ensure_selected_visible() {
        let mut app = make_app(100);
        app.visible_rows = 10;
        app.selected = 50;
        app.ensure_selected_visible();
        assert!(app.scroll_offset <= 50);
        assert!(app.scroll_offset + app.visible_rows > 50);
    }

    #[test]
    fn test_goto_line() {
        let mut app = make_app(100);
        app.goto_input = "50".to_string();
        app.goto_line();
        assert_eq!(app.selected, 49);
    }

    #[test]
    fn test_col_widths() {
        let app = make_app(10);
        assert!(app.col_widths[0] >= 19);
    }

    // ── Filter tests ────────────────────────────────────────────

    #[test]
    fn test_quick_exclude() {
        let mut app = make_app_with_levels(&[
            ("timeout error", Some(LogLevel::Error)),
            ("success", Some(LogLevel::Info)),
            ("timeout again", Some(LogLevel::Warn)),
            ("all good", Some(LogLevel::Info)),
        ]);

        app.quick_filter_input = "timeout".to_string();
        app.apply_quick_exclude();

        assert_eq!(app.filters.len(), 1);
        assert!(app.filters[0].exclude);
        assert_eq!(app.filtered_indices.len(), 2); // "success" and "all good"
    }

    #[test]
    fn test_quick_include() {
        let mut app = make_app_with_levels(&[
            ("timeout error", Some(LogLevel::Error)),
            ("success", Some(LogLevel::Info)),
            ("timeout again", Some(LogLevel::Warn)),
            ("all good", Some(LogLevel::Info)),
        ]);

        app.quick_filter_input = "timeout".to_string();
        app.apply_quick_include();

        assert_eq!(app.filters.len(), 1);
        assert!(!app.filters[0].exclude);
        assert_eq!(app.filtered_indices.len(), 2); // "timeout error" and "timeout again"
    }

    #[test]
    fn test_multiple_filters() {
        let mut app = make_app_with_levels(&[
            ("timeout error", Some(LogLevel::Error)),
            ("success msg", Some(LogLevel::Info)),
            ("timeout warning", Some(LogLevel::Warn)),
            ("disk error", Some(LogLevel::Error)),
            ("all good", Some(LogLevel::Info)),
        ]);

        // Exclude "timeout"
        app.quick_filter_input = "timeout".to_string();
        app.apply_quick_exclude();
        assert_eq!(app.filtered_indices.len(), 3);

        // Also exclude "disk"
        app.quick_filter_input = "disk".to_string();
        app.apply_quick_exclude();
        assert_eq!(app.filtered_indices.len(), 2);
        assert_eq!(app.filters.len(), 2);
    }

    #[test]
    fn test_remove_filter() {
        let mut app = make_app_with_levels(&[
            ("timeout error", Some(LogLevel::Error)),
            ("success", Some(LogLevel::Info)),
            ("timeout again", Some(LogLevel::Warn)),
        ]);

        app.quick_filter_input = "timeout".to_string();
        app.apply_quick_exclude();
        assert_eq!(app.filtered_indices.len(), 1);

        app.remove_filter(0);
        assert_eq!(app.filters.len(), 0);
        assert_eq!(app.filtered_indices.len(), 3);
    }

    #[test]
    fn test_clear_filters() {
        let mut app = make_app_with_levels(&[
            ("a", Some(LogLevel::Error)),
            ("b", Some(LogLevel::Info)),
            ("c", Some(LogLevel::Warn)),
        ]);

        app.quick_filter_input = "a".to_string();
        app.apply_quick_exclude();
        app.quick_filter_input = "b".to_string();
        app.apply_quick_exclude();
        assert_eq!(app.filtered_indices.len(), 1);

        app.clear_filters();
        assert_eq!(app.filters.len(), 0);
        assert_eq!(app.filtered_indices.len(), 3);
    }

    #[test]
    fn test_filter_expression() {
        let mut app = make_app_with_levels(&[
            ("error msg", Some(LogLevel::Error)),
            ("info msg", Some(LogLevel::Info)),
            ("warn msg", Some(LogLevel::Warn)),
        ]);

        app.filter_input = r#"level = "ERROR""#.to_string();
        app.apply_filter();
        // The filter parser requires string values in quotes
        assert_eq!(app.filters.len(), 1);
        // "error msg" has level Error, should match
        assert_eq!(
            app.filtered_indices.len(),
            1,
            "Expected 1 filtered record, got {}. filter_error: {:?}",
            app.filtered_indices.len(),
            app.filter_error
        );
    }

    #[test]
    fn test_field_filter_opens() {
        let mut app = make_app_with_levels(&[("error msg", Some(LogLevel::Error))]);
        Arc::get_mut(&mut app.records[0]).unwrap().process_name = Some("myapp".to_string());
        Arc::get_mut(&mut app.records[0]).unwrap().pid = Some(1234);

        app.open_field_filter(true);
        assert_eq!(app.input_mode, InputMode::FieldFilter);
        let ff = app.field_filter.as_ref().unwrap();
        assert!(ff.fields.len() >= 4); // timestamp, level, source, process_name, pid, message
        assert!(ff.fields.iter().any(|e| e.name == "level"));
        assert!(ff.fields.iter().any(|e| e.name == "process_name"));
        assert!(ff.fields.iter().any(|e| e.name == "source"));
        assert!(ff.fields.iter().any(|e| e.name == "timestamp"));
        assert!(ff.fields.iter().any(|e| e.name == "message"));
        assert!(ff.fields.iter().any(|e| e.name == "pid"));
        assert!(ff.logic_or); // default OR
    }
}

#[cfg(test)]
mod field_filter_v2_tests {
    use super::*;
    use chrono::Utc;
    use scouty::record::{LogLevel, LogRecord};

    fn make_record_full(id: u64, msg: &str, level: LogLevel) -> LogRecord {
        LogRecord {
            id,
            timestamp: Utc::now(),
            level: Some(level),
            source: "syslog".into(),
            pid: Some(1000 + id as u32),
            tid: Some(2000 + id as u32),
            component_name: Some("comp".into()),
            process_name: Some("myapp".into()),
            hostname: None,
            container: None,
            context: None,
            function: None,
            message: msg.to_string(),
            raw: msg.to_string(),
            metadata: None,
            loader_id: "test".into(),
        }
    }

    fn make_app_full(records: Vec<LogRecord>) -> App {
        let n = records.len();
        let records: Vec<Arc<LogRecord>> = records.into_iter().map(Arc::new).collect();
        let filtered_indices = (0..n).collect();
        App {
            records,
            total_records: n,
            filtered_indices,
            scroll_offset: 0,
            selected: 0,
            visible_rows: 10,
            detail_open: false,
            input_mode: InputMode::Normal,
            filter_input: String::new(),
            filter_error: None,
            filters: Vec::new(),
            quick_filter_input: String::new(),
            field_filter: None,
            filter_manager_cursor: 0,
            search_input: String::new(),
            search_matches: vec![],
            search_match_idx: None,
            time_input: String::new(),
            goto_input: String::new(),
            status_message: None,
            status_message_at: None,
            col_widths: [19, 5, 11, 3, 3, 9],
            column_config: ColumnConfig::default(),
            follow_mode: false,
        }
    }

    #[test]
    fn test_field_filter_all_fields() {
        let records = vec![make_record_full(0, "test msg", LogLevel::Error)];
        let mut app = make_app_full(records);

        app.open_field_filter(true);
        let ff = app.field_filter.as_ref().unwrap();
        // Should have: 2 time options + timestamp, level, source, process_name, pid, tid, component, message
        assert_eq!(ff.fields.len(), 10);
        assert!(ff.exclude);
        assert!(ff.logic_or);
    }

    #[test]
    fn test_field_filter_include_mode() {
        let records = vec![make_record_full(0, "test msg", LogLevel::Info)];
        let mut app = make_app_full(records);

        app.open_field_filter(false);
        let ff = app.field_filter.as_ref().unwrap();
        assert!(!ff.exclude);
    }

    #[test]
    fn test_field_filter_or_logic() {
        let records = vec![
            make_record_full(0, "err msg", LogLevel::Error),
            make_record_full(1, "info msg", LogLevel::Info),
            make_record_full(2, "warn msg", LogLevel::Warn),
        ];
        let mut app = make_app_full(records);

        app.open_field_filter(false); // include mode
        let ff = app.field_filter.as_mut().unwrap();
        assert!(ff.logic_or);

        // Check level field (index 1) and message field
        let level_idx = ff.fields.iter().position(|e| e.name == "level").unwrap();
        ff.fields[level_idx].checked = true; // check level = ERROR

        // Apply — should include only record with level=ERROR
        app.apply_field_filter();
        assert_eq!(app.filtered_indices.len(), 1);
    }

    #[test]
    fn test_field_filter_and_logic() {
        let records = vec![
            make_record_full(0, "err msg", LogLevel::Error),
            make_record_full(1, "info msg", LogLevel::Info),
        ];
        let mut app = make_app_full(records);

        app.open_field_filter(false); // include
        let ff = app.field_filter.as_mut().unwrap();
        ff.logic_or = false; // AND

        // Check both level and pid
        let level_idx = ff.fields.iter().position(|e| e.name == "level").unwrap();
        let pid_idx = ff.fields.iter().position(|e| e.name == "pid").unwrap();
        ff.fields[level_idx].checked = true;
        ff.fields[pid_idx].checked = true;

        app.apply_field_filter();
        // Only record 0 has level=ERROR AND pid=1000
        assert_eq!(app.filtered_indices.len(), 1);
    }

    #[test]
    fn test_field_filter_metadata_fields() {
        let mut record = make_record_full(0, "test", LogLevel::Info);
        let mut meta = std::collections::HashMap::new();
        meta.insert("env".to_string(), "prod".to_string());
        meta.insert("region".to_string(), "us-west".to_string());
        record.metadata = Some(meta);

        let mut app = make_app_full(vec![record]);
        app.open_field_filter(true);
        let ff = app.field_filter.as_ref().unwrap();
        // 2 time options + 8 standard fields + 2 metadata
        assert_eq!(ff.fields.len(), 12);
        assert!(ff.fields.iter().any(|e| e.name == "env"));
        assert!(ff.fields.iter().any(|e| e.name == "region"));
    }

    #[test]
    fn test_help_mode_toggle() {
        let records = vec![
            make_record_full(0, "a", LogLevel::Info),
            make_record_full(1, "b", LogLevel::Info),
            make_record_full(2, "c", LogLevel::Info),
        ];
        let mut app = make_app_full(records);
        assert_eq!(app.input_mode, InputMode::Normal);
        app.input_mode = InputMode::Help;
        assert_eq!(app.input_mode, InputMode::Help);
        app.input_mode = InputMode::Normal;
        assert_eq!(app.input_mode, InputMode::Normal);
    }
}

#[cfg(test)]
mod column_follow_tests {
    use super::*;
    use chrono::Utc;
    use scouty::record::{LogLevel, LogRecord};

    fn make_record(id: u64, message: &str) -> LogRecord {
        LogRecord {
            id,
            timestamp: Utc::now(),
            level: Some(LogLevel::Info),
            source: "test".into(),
            pid: Some(100),
            tid: Some(200),
            component_name: Some("comp".into()),
            process_name: Some("proc".into()),
            hostname: None,
            container: None,
            context: None,
            function: None,
            message: message.to_string(),
            raw: message.to_string(),
            metadata: None,
            loader_id: "test".into(),
        }
    }

    fn make_app_cf(n: usize) -> App {
        let records: Vec<Arc<LogRecord>> = (0..n)
            .map(|i| Arc::new(make_record(i as u64, &format!("msg {}", i))))
            .collect();
        let filtered_indices = (0..n).collect();
        App {
            records,
            total_records: n,
            filtered_indices,
            scroll_offset: 0,
            selected: 0,
            visible_rows: 10,
            detail_open: false,
            input_mode: InputMode::Normal,
            filter_input: String::new(),
            filter_error: None,
            filters: Vec::new(),
            quick_filter_input: String::new(),
            field_filter: None,
            filter_manager_cursor: 0,
            search_input: String::new(),
            search_matches: vec![],
            search_match_idx: None,
            time_input: String::new(),
            goto_input: String::new(),
            status_message: None,
            status_message_at: None,
            col_widths: [19, 5, 11, 3, 3, 9],
            column_config: ColumnConfig::default(),
            follow_mode: false,
        }
    }

    // ── Column config tests ──────────────────────────────────

    #[test]
    fn test_default_column_config() {
        let config = ColumnConfig::default();
        assert!(config.is_visible(Column::Time));
        assert!(!config.is_visible(Column::Level)); // hidden by default
        assert!(!config.is_visible(Column::ProcessName)); // hidden by default
        assert!(config.is_visible(Column::Log));
        assert!(!config.is_visible(Column::Source)); // hidden by default
    }

    #[test]
    fn test_toggle_column() {
        let mut config = ColumnConfig::default();
        // Find ProcessName index — hidden by default
        let idx = config
            .columns
            .iter()
            .position(|(c, _)| *c == Column::ProcessName)
            .unwrap();
        assert!(!config.is_visible(Column::ProcessName));
        config.toggle(idx);
        assert!(config.is_visible(Column::ProcessName));
        config.toggle(idx);
        assert!(!config.is_visible(Column::ProcessName));
    }

    #[test]
    fn test_cannot_toggle_log() {
        let mut config = ColumnConfig::default();
        let idx = config
            .columns
            .iter()
            .position(|(c, _)| *c == Column::Log)
            .unwrap();
        assert!(config.is_visible(Column::Log));
        config.toggle(idx);
        assert!(config.is_visible(Column::Log)); // still visible
    }

    #[test]
    fn test_visible_columns() {
        let mut config = ColumnConfig::default();
        let default_visible = config.visible_columns();
        assert_eq!(default_visible.len(), 2); // Time + Log only

        // Show Level
        let idx = config
            .columns
            .iter()
            .position(|(c, _)| *c == Column::Level)
            .unwrap();
        config.toggle(idx);
        let visible = config.visible_columns();
        assert_eq!(visible.len(), 3);
        assert!(visible.contains(&Column::Level));
    }

    #[test]
    fn test_show_source_column() {
        let mut config = ColumnConfig::default();
        let idx = config
            .columns
            .iter()
            .position(|(c, _)| *c == Column::Source)
            .unwrap();
        config.toggle(idx);
        assert!(config.is_visible(Column::Source));
        assert_eq!(config.visible_columns().len(), 3); // Time + Log + Source
    }

    // ── Follow mode tests ────────────────────────────────────

    #[test]
    fn test_follow_mode_toggle() {
        let mut app = make_app_cf(100);
        assert!(!app.follow_mode);
        app.toggle_follow();
        assert!(app.follow_mode);
        assert_eq!(app.selected, 99); // scrolled to bottom
        app.toggle_follow();
        assert!(!app.follow_mode);
    }

    #[test]
    fn test_follow_mode_exits_on_scroll_up() {
        let mut app = make_app_cf(100);
        app.toggle_follow();
        assert!(app.follow_mode);
        app.select_up(5);
        assert!(!app.follow_mode);
    }

    #[test]
    fn test_follow_mode_exits_on_page_up() {
        let mut app = make_app_cf(100);
        app.toggle_follow();
        assert!(app.follow_mode);
        app.page_up();
        assert!(!app.follow_mode);
    }

    #[test]
    fn test_follow_mode_persists_on_down() {
        let mut app = make_app_cf(100);
        app.toggle_follow();
        assert!(app.follow_mode);
        // Already at bottom, select_down shouldn't exit follow
        app.select_down(1);
        assert!(app.follow_mode);
    }
}

#[cfg(test)]
mod copy_tests {
    use super::*;
    use chrono::Utc;
    use scouty::record::{LogLevel, LogRecord};

    fn make_record(id: u64, msg: &str) -> LogRecord {
        LogRecord {
            id,
            timestamp: Utc::now(),
            level: Some(LogLevel::Info),
            source: "test".into(),
            pid: Some(1234),
            tid: None,
            component_name: None,
            process_name: Some("app".into()),
            hostname: None,
            container: None,
            context: None,
            function: None,
            message: msg.to_string(),
            raw: msg.to_string(),
            metadata: None,
            loader_id: "test".into(),
        }
    }

    fn make_app_copy(records: Vec<LogRecord>) -> App {
        let n = records.len();
        let records: Vec<Arc<LogRecord>> = records.into_iter().map(Arc::new).collect();
        App {
            records,
            total_records: n,
            filtered_indices: (0..n).collect(),
            scroll_offset: 0,
            selected: 0,
            visible_rows: 10,
            detail_open: false,
            input_mode: InputMode::Normal,
            filter_input: String::new(),
            filter_error: None,
            filters: Vec::new(),
            quick_filter_input: String::new(),
            field_filter: None,
            filter_manager_cursor: 0,
            search_input: String::new(),
            search_matches: vec![],
            search_match_idx: None,
            time_input: String::new(),
            goto_input: String::new(),
            status_message: None,
            status_message_at: None,
            col_widths: [19, 5, 11, 3, 3, 9],
            column_config: ColumnConfig::default(),
            follow_mode: false,
        }
    }

    #[test]
    fn test_copy_raw() {
        let mut app = make_app_copy(vec![make_record(0, "hello world")]);
        let result = app.copy_raw();
        assert_eq!(result, Some("hello world".to_string()));
        assert!(app.status_message.unwrap().contains("raw"));
    }

    #[test]
    fn test_copy_raw_empty() {
        let mut app = make_app_copy(vec![]);
        let result = app.copy_raw();
        assert_eq!(result, None);
    }

    #[test]
    fn test_copy_as_json() {
        let mut app = make_app_copy(vec![make_record(0, "test msg")]);
        let result = app.copy_as_format(CopyFormat::Json);
        assert!(result.is_some());
        let json = result.unwrap();
        assert!(json.contains("\"message\""));
        assert!(json.contains("test msg"));
        assert_eq!(app.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_copy_as_yaml() {
        let mut app = make_app_copy(vec![make_record(0, "test msg")]);
        let result = app.copy_as_format(CopyFormat::Yaml);
        assert!(result.is_some());
        let yaml = result.unwrap();
        assert!(yaml.contains("message"));
        assert!(yaml.contains("test msg"));
    }

    #[test]
    fn test_osc52_does_not_panic() {
        // Just ensure it doesn't panic; actual clipboard is terminal-dependent
        osc52_copy("test data");
    }

    // ── Hostname & Container column tests ────────────────────

    #[test]
    fn test_hostname_container_columns_default_hidden() {
        let config = ColumnConfig::default();
        assert!(!config.is_visible(Column::Hostname));
        assert!(!config.is_visible(Column::Container));
    }

    #[test]
    fn test_hostname_container_columns_togglable() {
        let mut config = ColumnConfig::default();
        let host_idx = config
            .columns
            .iter()
            .position(|(c, _)| *c == Column::Hostname)
            .unwrap();
        let ctr_idx = config
            .columns
            .iter()
            .position(|(c, _)| *c == Column::Container)
            .unwrap();
        config.toggle(host_idx);
        config.toggle(ctr_idx);
        assert!(config.is_visible(Column::Hostname));
        assert!(config.is_visible(Column::Container));
        assert_eq!(config.visible_columns().len(), 4); // Time + Log + Hostname + Container
    }

    #[test]
    fn test_hostname_container_in_column_selector() {
        let config = ColumnConfig::default();
        let labels: Vec<&str> = config.columns.iter().map(|(c, _)| c.label()).collect();
        assert!(labels.contains(&"Hostname"));
        assert!(labels.contains(&"Container"));
    }
}
