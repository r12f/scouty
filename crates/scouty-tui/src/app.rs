//! Application state for the TUI.

use crate::config::Theme;
use crate::text_input::TextInput;
use ratatui::style::Color;
use regex::Regex;
use scouty::filter::eval;
use scouty::filter::expr::{self, Expr};
use scouty::loader::file::FileLoader;
use scouty::loader::ssh::{is_ssh_url, SshLoader, SshUrl};
use scouty::parser::factory::ParserFactory;
use scouty::record::LogRecord;
use scouty::traits::LogLoader;
use std::sync::Arc;
use tracing::{instrument, warn};

/// Input mode for the TUI.
#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Filter,
    Search,
    JumpForward,
    JumpBackward,
    GotoLine,
    QuickExclude,
    QuickInclude,
    FieldFilter,
    FilterManager,
    ColumnSelector,
    CopyFormat,
    Help,
    Command,
    BookmarkManager,

    Highlight,
    HighlightManager,
    LevelFilter,
    SavePreset,
    LoadPreset,
    DensitySelector,
    SaveDialog,
    RegionManager,
}

/// Column identifiers for the log table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    /// Index into the col_widths array, if applicable.
    #[allow(dead_code)]
    pub fn col_widths_index(&self) -> Option<usize> {
        match self {
            Column::Time => Some(0),
            Column::Level => Some(1),
            Column::ProcessName => Some(2),
            Column::Pid => Some(3),
            Column::Tid => Some(4),
            Column::Component => Some(5),
            Column::Context => Some(6),
            Column::Function => Some(7),
            _ => None,
        }
    }

    /// Default fixed width for columns not tracked by `col_widths`.
    /// Returns 0 for `Log` (fill column) and columns that belong in `col_widths`.
    #[allow(dead_code)]
    pub fn default_fixed_width(&self) -> u16 {
        match self {
            Column::Hostname => 20,
            Column::Container => 15,
            Column::Source => 15,
            Column::Log => 0,
            // Columns tracked by col_widths should use col_widths_index() instead.
            _ => 0,
        }
    }

    /// Minimum width for this column (cannot shrink below this).
    #[allow(dead_code)]
    pub fn min_width(&self) -> u16 {
        match self {
            Column::Time => 19,
            Column::Level => 3,
            Column::Hostname => 4,
            Column::Container => 4,
            Column::ProcessName => 4,
            Column::Pid => 3,
            Column::Tid => 3,
            Column::Component => 4,
            Column::Function => 4,
            Column::Context => 4,
            Column::Source => 4,
            Column::Log => 0, // fill — not applicable
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
    /// Manual width overrides (parallel to `columns`). `None` = auto-computed.
    pub width_overrides: Vec<Option<u16>>,
}

impl Default for ColumnConfig {
    fn default() -> Self {
        let columns = vec![
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
        ];
        let width_overrides = vec![None; columns.len()];
        Self {
            columns,
            cursor: 0,
            width_overrides,
        }
    }
}

impl ColumnConfig {
    /// Get the effective width for a column at the given index.
    /// Returns the manual override if set, otherwise the auto-computed width.
    #[allow(dead_code)]
    pub fn effective_width(&self, index: usize, auto_width: u16) -> u16 {
        if index < self.width_overrides.len() {
            self.width_overrides[index].unwrap_or(auto_width)
        } else {
            auto_width
        }
    }

    /// Adjust width for column at index by delta. Respects min_width.
    /// Returns true if width changed.
    #[allow(dead_code)]
    pub fn adjust_width(&mut self, index: usize, delta: i16, auto_width: u16) -> bool {
        if index >= self.columns.len() {
            return false;
        }
        let (col, visible) = &self.columns[index];
        if *col == Column::Log || !visible {
            return false;
        }
        let current = self.effective_width(index, auto_width);
        let min = col.min_width();
        let new_width = ((current as i32) + (delta as i32))
            .max(min as i32)
            .min(u16::MAX as i32) as u16;
        if new_width != current {
            self.width_overrides[index] = Some(new_width);
            true
        } else {
            false
        }
    }

    /// Reset width override for column at index (back to auto-computed).
    #[allow(dead_code)]
    pub fn reset_width(&mut self, index: usize) {
        if index < self.width_overrides.len() {
            self.width_overrides[index] = None;
        }
    }

    /// Get the auto-computed width for a column, given col_widths array.
    #[allow(dead_code)]
    pub fn auto_width_for(&self, index: usize, col_widths: &[u16; 8]) -> u16 {
        if index >= self.columns.len() {
            return 0;
        }
        let col = &self.columns[index].0;
        if let Some(cw_idx) = col.col_widths_index() {
            col_widths[cw_idx]
        } else {
            col.default_fixed_width()
        }
    }

    /// Get the display width for a column (effective = override or auto).
    #[allow(dead_code)]
    pub fn display_width(&self, index: usize, col_widths: &[u16; 8]) -> u16 {
        let auto = self.auto_width_for(index, col_widths);
        self.effective_width(index, auto)
    }
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
    /// The original parseable expression string.
    pub expr_str: String,
    /// The compiled expression.
    pub expr: Expr,
    /// Whether this is an exclude (true) or include (false) filter.
    pub exclude: bool,
}

/// Kind of field filter entry.
#[derive(Clone, Debug, PartialEq)]
pub enum FieldEntryKind {
    /// Regular field: generates `field == "value"`.
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

/// Source for density chart data.
#[derive(Debug, Clone, PartialEq)]
pub enum DensitySource {
    /// All filtered records (default).
    All,
    /// Only records matching a specific level.
    Level(String),
    /// Only records matching a specific highlight rule (by pattern).
    Highlight(String),
}

/// A single highlight rule: regex pattern + assigned color.
#[derive(Debug, Clone)]
pub struct HighlightRule {
    /// The regex pattern string.
    pub pattern: String,
    /// Compiled regex.
    pub regex: Regex,
    /// Assigned foreground color.
    pub color: Color,
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
    /// Detail panel max height ratio (0.1 - 0.9).
    pub detail_panel_ratio: f64,
    /// Detail panel: tree cursor position (index into flattened tree).
    pub detail_tree_cursor: usize,
    /// Detail panel: set of collapsed node indices (path-based keys).
    pub detail_tree_collapsed: std::collections::HashSet<String>,
    /// Detail panel: horizontal scroll offset (columns).
    pub detail_horizontal_offset: usize,
    /// Panel system state.
    pub panel_state: crate::panel::PanelState,
    /// Current input mode.
    pub input_mode: InputMode,
    /// Filter input buffer (for expression mode).
    pub filter_input: TextInput,
    /// Filter error message.
    pub filter_error: Option<String>,
    /// Stack of active filters.
    pub filters: Vec<FilterEntry>,
    /// Quick exclude/include input buffer.
    pub quick_filter_input: TextInput,
    /// Field filter dialog state.
    pub field_filter: Option<FieldFilterState>,
    /// Filter manager: selected index.
    pub filter_manager_cursor: usize,
    /// Search input buffer.
    pub search_input: TextInput,
    /// Current search matches (indices into filtered list).
    pub search_matches: Vec<usize>,
    /// Cached compiled search regex (for incremental follow-mode search).
    pub search_regex: Option<regex::Regex>,
    /// Current search match index.
    pub search_match_idx: Option<usize>,
    /// Time jump input buffer.
    pub time_input: TextInput,
    /// Goto line input buffer.
    pub goto_input: TextInput,
    /// Status message shown temporarily.
    pub status_message: Option<String>,

    /// Dynamic shortcut hints collected from focus path (set by MainWindow before render).
    pub shortcut_hints_cache: Vec<(String, String)>,
    /// Timestamp when status_message was set (for auto-clear).
    pub status_message_at: Option<std::time::Instant>,
    /// Column widths computed from data (Time, Level, ProcessName, Pid, Tid, Component, Context, Function).
    pub col_widths: [u16; 8],
    /// Column visibility configuration.
    pub column_config: ColumnConfig,
    /// Follow mode: auto-scroll to bottom.
    pub follow_mode: bool,
    /// Count of new records added since user scrolled away from bottom.
    pub follow_new_count: usize,
    pub should_quit: bool,
    /// Copy format dialog cursor (0=Raw, 1=JSON, 2=YAML).
    pub copy_format_cursor: usize,
    /// Save dialog: path input.
    pub save_path_input: TextInput,
    /// Save dialog: format cursor (0=Raw, 1=JSON, 2=YAML).
    pub save_format_cursor: usize,
    /// Save dialog: current focus (Path or Format).
    pub save_dialog_focus: crate::ui::windows::save_dialog_window::Focus,
    /// Scroll offset for help window.
    pub help_scroll: u16,
    /// Save file input buffer.
    pub command_input: TextInput,
    /// Filter version counter (incremented on filter/data change, for density cache invalidation).
    pub filter_version: u64,
    /// Cached density chart data.
    pub density_cache: Option<DensityCache>,
    /// Highlight rules.
    pub highlight_rules: Vec<HighlightRule>,
    /// Highlight input buffer.
    pub highlight_input: TextInput,
    /// Highlight manager cursor.
    pub highlight_manager_cursor: usize,
    /// Bookmarked record IDs.
    pub bookmarks: std::collections::HashSet<u64>,
    /// Bookmark manager cursor.
    pub bookmark_manager_cursor: usize,
    /// Theme for UI colors.
    pub theme: Theme,
    /// Active level filter (None = ALL).
    pub level_filter: Option<LevelFilterPreset>,
    /// Level filter cursor for overlay navigation.
    pub level_filter_cursor: usize,
    /// Save preset name input.
    pub preset_name_input: TextInput,
    /// Available preset names for load dialog.
    pub preset_list: Vec<String>,
    /// Load preset cursor.
    pub preset_list_cursor: usize,
    /// Current density chart source.
    pub density_source: DensitySource,
    /// Density selector cursor.
    pub density_selector_cursor: usize,
    /// Detected regions from region processor.
    pub regions: scouty::region::store::RegionStore,
    /// Region processor for incremental follow-mode processing.
    pub region_processor: Option<scouty::region::processor::RegionProcessor>,
    /// Categorization processor (evaluates categories and tracks stats).
    pub category_processor: Option<scouty::category::CategoryProcessor>,
    /// Category panel cursor position.
    pub category_cursor: usize,
    /// Region manager cursor position.
    pub region_manager_cursor: usize,
    /// Region panel sort mode.
    pub region_panel_sort: crate::ui::widgets::region_panel_widget::RegionSortMode,
    /// Region panel type filter (None = show all).
    pub region_panel_type_filter: Option<String>,
}

/// Level filter presets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LevelFilterPreset {
    /// Show all levels.
    All,
    /// TRACE and above.
    TracePlus,
    /// DEBUG and above.
    DebugPlus,
    /// INFO and above.
    InfoPlus,
    /// NOTICE and above.
    NoticePlus,
    /// WARN and above.
    WarnPlus,
    /// ERROR and above.
    ErrorPlus,
    /// FATAL only.
    FatalOnly,
}

impl LevelFilterPreset {
    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "ALL",
            Self::TracePlus => "TRACE+",
            Self::DebugPlus => "DEBUG+",
            Self::InfoPlus => "INFO+",
            Self::NoticePlus => "NOTICE+",
            Self::WarnPlus => "WARN+",
            Self::ErrorPlus => "ERROR+",
            Self::FatalOnly => "FATAL",
        }
    }

    pub fn from_number(n: u8) -> Option<Self> {
        match n {
            1 => Some(Self::All),
            2 => Some(Self::TracePlus),
            3 => Some(Self::DebugPlus),
            4 => Some(Self::InfoPlus),
            5 => Some(Self::NoticePlus),
            6 => Some(Self::WarnPlus),
            7 => Some(Self::ErrorPlus),
            8 => Some(Self::FatalOnly),
            _ => None,
        }
    }

    pub fn as_number(&self) -> u8 {
        match self {
            Self::All => 1,
            Self::TracePlus => 2,
            Self::DebugPlus => 3,
            Self::InfoPlus => 4,
            Self::NoticePlus => 5,
            Self::WarnPlus => 6,
            Self::ErrorPlus => 7,
            Self::FatalOnly => 8,
        }
    }

    /// Returns the minimum log level ordinal that passes this filter.
    pub fn matches_level(&self, level: Option<&scouty::record::LogLevel>) -> bool {
        use scouty::record::LogLevel;
        match self {
            Self::All => true,
            Self::TracePlus => matches!(
                level,
                Some(
                    LogLevel::Trace
                        | LogLevel::Debug
                        | LogLevel::Info
                        | LogLevel::Notice
                        | LogLevel::Warn
                        | LogLevel::Error
                        | LogLevel::Fatal
                )
            ),
            Self::DebugPlus => matches!(
                level,
                Some(
                    LogLevel::Debug
                        | LogLevel::Info
                        | LogLevel::Notice
                        | LogLevel::Warn
                        | LogLevel::Error
                        | LogLevel::Fatal
                )
            ),
            Self::InfoPlus => matches!(
                level,
                Some(
                    LogLevel::Info
                        | LogLevel::Notice
                        | LogLevel::Warn
                        | LogLevel::Error
                        | LogLevel::Fatal
                )
            ),
            Self::NoticePlus => matches!(
                level,
                Some(LogLevel::Notice | LogLevel::Warn | LogLevel::Error | LogLevel::Fatal)
            ),
            Self::WarnPlus => matches!(
                level,
                Some(LogLevel::Warn | LogLevel::Error | LogLevel::Fatal)
            ),
            Self::ErrorPlus => matches!(level, Some(LogLevel::Error | LogLevel::Fatal)),
            Self::FatalOnly => matches!(level, Some(LogLevel::Fatal)),
        }
    }
}

/// Cached density chart — avoids O(N) recomputation on every frame.
#[derive(Clone)]
pub struct DensityCache {
    /// Pre-rendered braille string.
    pub braille_text: String,
    /// Number of buckets used for cursor position lookup.
    pub num_buckets: usize,
    /// Min timestamp in filtered data.
    pub min_ts: chrono::DateTime<chrono::Utc>,
    /// Max timestamp in filtered data.
    pub max_ts: chrono::DateTime<chrono::Utc>,
    /// Filter version when cache was built.
    pub filter_version: u64,
    /// Chart width when cache was built.
    pub chart_width: usize,
    /// Density source when cache was built.
    pub density_source: DensitySource,
}

impl App {
    /// Load log records from file paths.
    /// Load log records from file paths and/or SSH URLs.
    #[instrument(skip(paths), fields(file_count = paths.len()))]
    pub fn load_files(
        paths: &[&str],
        ssh_connect_timeout: u32,
        ssh_keepalive_interval: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        tracing::info!(file_count = paths.len(), "loading files");
        let mut store = scouty::store::LogStore::new();
        let mut record_id: u64 = 0;

        for path in paths {
            tracing::info!(%path, "loading file");
            if is_ssh_url(path) {
                let url = SshUrl::parse(path).map_err(|e| {
                    Box::<dyn std::error::Error>::from(format!("Invalid SSH URL '{}': {}", path, e))
                })?;
                let mut loader = SshLoader::new(url, ssh_connect_timeout, ssh_keepalive_interval);
                let lines = loader.load()?;
                let info = loader.info().clone();
                Self::ingest_lines(&mut store, lines, &info, &mut record_id);
            } else {
                let mut loader = FileLoader::new(path, false);
                let lines = loader.load()?;
                let info = loader.info().clone();
                Self::ingest_lines(&mut store, lines, &info, &mut record_id);
            }
        }

        Self::from_store(store)
    }

    /// Load log records from pre-read stdin lines.
    #[instrument(skip(lines), fields(line_count = lines.len()))]
    pub fn load_stdin(lines: Vec<String>) -> Result<Self, Box<dyn std::error::Error>> {
        use scouty::loader::stdin::StdinLoader;

        let loader = StdinLoader::new();
        let mut info = loader.info().clone();
        info.sample_lines = lines.iter().take(10).cloned().collect();

        let mut store = scouty::store::LogStore::new();
        let mut record_id: u64 = 0;
        Self::ingest_lines(&mut store, lines, &info, &mut record_id);

        Self::from_store(store)
    }

    /// Parse lines with auto-detected parser and insert into the store.
    fn ingest_lines(
        store: &mut scouty::store::LogStore,
        lines: Vec<String>,
        info: &scouty::traits::LoaderInfo,
        record_id: &mut u64,
    ) {
        let group = ParserFactory::create_parser_group(info);
        for line in lines {
            if let Some(mut record) = group.parse(&line, &info.id, &info.id, *record_id) {
                record.raw = line;
                store.insert(record);
                *record_id += 1;
            }
        }
    }

    /// Build an `App` from a populated `LogStore`.
    fn from_store(mut store: scouty::store::LogStore) -> Result<Self, Box<dyn std::error::Error>> {
        store.compact_ooo();
        let records: Vec<Arc<LogRecord>> = store.iter_arc().cloned().collect();
        let total_records = records.len();
        tracing::info!(total_records, "files loaded, records parsed");
        let filtered_indices: Vec<usize> = (0..records.len()).collect();
        let col_widths = Self::compute_col_widths(&records, &filtered_indices);

        Ok(Self {
            records,
            total_records,
            filtered_indices,
            scroll_offset: 0,
            selected: 0,
            visible_rows: 20,
            detail_panel_ratio: 0.3,
            detail_tree_cursor: 0,
            detail_tree_collapsed: std::collections::HashSet::new(),
            detail_horizontal_offset: 0,
            panel_state: crate::panel::PanelState::default(),
            input_mode: InputMode::Normal,
            filter_input: TextInput::new(),
            filter_error: None,
            filters: Vec::new(),
            quick_filter_input: TextInput::new(),
            field_filter: None,
            filter_manager_cursor: 0,
            search_input: TextInput::new(),
            search_matches: vec![],
            search_regex: None,
            search_match_idx: None,
            time_input: TextInput::new(),
            goto_input: TextInput::new(),
            status_message: None,
            shortcut_hints_cache: Vec::new(),
            status_message_at: None,
            col_widths,
            column_config: ColumnConfig::default(),
            follow_mode: false,
            follow_new_count: 0,
            should_quit: false,
            copy_format_cursor: 0,
            save_path_input: TextInput::with_text("./scouty-export.log"),
            save_format_cursor: 0,
            save_dialog_focus: crate::ui::windows::save_dialog_window::Focus::Path,
            help_scroll: 0,
            command_input: TextInput::new(),
            filter_version: 0,
            density_cache: None,
            highlight_rules: Vec::new(),
            highlight_input: TextInput::new(),
            highlight_manager_cursor: 0,
            bookmarks: std::collections::HashSet::new(),
            bookmark_manager_cursor: 0,
            theme: Theme::default(),
            level_filter: None,
            level_filter_cursor: 0,
            preset_name_input: TextInput::new(),
            preset_list: Vec::new(),
            preset_list_cursor: 0,
            density_source: DensitySource::All,
            density_selector_cursor: 0,
            regions: scouty::region::store::RegionStore::default(),
            region_processor: None,
            category_processor: None,
            category_cursor: 0,
            region_manager_cursor: 0,
            region_panel_sort: crate::ui::widgets::region_panel_widget::RegionSortMode::StartTime,
            region_panel_type_filter: None,
        })
    }

    /// Compute auto-fit column widths by sampling records.
    fn compute_col_widths(records: &[Arc<LogRecord>], indices: &[usize]) -> [u16; 8] {
        let mut widths: [u16; 8] = [4, 5, 11, 3, 3, 9, 7, 8];
        let max_widths: [u16; 8] = [23, 5, 20, 8, 8, 30, 40, 30];

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
            if let Some(ref ctx) = r.context {
                widths[6] = widths[6].max((ctx.len() as u16).min(max_widths[6]));
            }
            if let Some(ref func) = r.function {
                widths[7] = widths[7].max((func.len() as u16).min(max_widths[7]));
            }
        }

        for i in 0..8 {
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

    // ── Density chart cache ─────────────────────────────────────

    /// Get or rebuild the density cache. Returns None if no data.
    pub fn get_density_cache(&mut self, chart_width: usize) -> Option<&DensityCache> {
        if chart_width == 0 {
            return None;
        }
        let num_buckets = (chart_width * 2).min(200);
        if num_buckets == 0 {
            return None;
        }
        let needs_rebuild = match &self.density_cache {
            Some(c) => {
                c.filter_version != self.filter_version
                    || c.chart_width != chart_width
                    || c.density_source != self.density_source
            }
            None => true,
        };

        if needs_rebuild {
            if self.filtered_indices.is_empty() {
                self.density_cache = None;
                return None;
            }

            let source_indices = self.density_source_indices();
            if source_indices.is_empty() {
                self.density_cache = None;
                return None;
            }

            let (buckets, min_ts, max_ts) = crate::density::compute_density_indexed(
                &self.records,
                &source_indices,
                num_buckets,
            );

            let (braille_text, _) = crate::density::render_braille(&buckets, None);

            self.density_cache = Some(DensityCache {
                braille_text,
                num_buckets,
                min_ts,
                max_ts,
                filter_version: self.filter_version,
                chart_width,
                density_source: self.density_source.clone(),
            });
        }

        self.density_cache.as_ref()
    }

    /// Compute cursor char index in braille text from current selection — O(1).
    pub fn cursor_char_in_density(&self) -> Option<usize> {
        let cache = self.density_cache.as_ref()?;
        let record = self.selected_record()?;
        let cursor_ts = record.timestamp;
        if cache.min_ts == cache.max_ts {
            return Some(0);
        }
        let range_ms = (cache.max_ts - cache.min_ts).num_milliseconds() as f64;
        let offset_ms = (cursor_ts - cache.min_ts).num_milliseconds() as f64;
        let idx = ((offset_ms / range_ms) * (cache.num_buckets as f64 - 1.0)) as usize;
        Some(idx.min(cache.num_buckets - 1) / 2)
    }

    /// Compute filtered indices for the current density source.
    fn density_source_indices(&self) -> Vec<usize> {
        match &self.density_source {
            DensitySource::All => self.filtered_indices.clone(),
            DensitySource::Level(level) => match scouty::record::LogLevel::from_str_loose(level) {
                Some(ref lvl) => self
                    .filtered_indices
                    .iter()
                    .copied()
                    .filter(|&i| self.records[i].level.as_ref() == Some(lvl))
                    .collect(),
                None => Vec::new(),
            },
            DensitySource::Highlight(pattern) => {
                if let Some(rule) = self.highlight_rules.iter().find(|r| r.pattern == *pattern) {
                    let regex = rule.regex.clone();
                    self.filtered_indices
                        .iter()
                        .copied()
                        .filter(|&i| regex.is_match(&self.records[i].raw))
                        .collect()
                } else {
                    Vec::new()
                }
            }
        }
    }

    /// Cycle density source: All -> ERROR -> WARN -> highlights -> All.
    pub fn cycle_density_source(&mut self) {
        let sources = self.density_source_options();
        let current_idx = sources
            .iter()
            .position(|s| *s == self.density_source)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % sources.len();
        self.density_source = sources[next_idx].clone();
        self.density_cache = None;
        self.set_status(format!("Density: {}", self.density_source_label()));
    }

    /// All available density source options.
    pub fn density_source_options(&self) -> Vec<DensitySource> {
        let mut options = vec![
            DensitySource::All,
            DensitySource::Level("FATAL".to_string()),
            DensitySource::Level("ERROR".to_string()),
            DensitySource::Level("WARN".to_string()),
            DensitySource::Level("INFO".to_string()),
        ];
        for rule in &self.highlight_rules {
            options.push(DensitySource::Highlight(rule.pattern.clone()));
        }
        options
    }

    /// Label for current density source.
    pub fn density_source_label(&self) -> String {
        match &self.density_source {
            DensitySource::All => "All".to_string(),
            DensitySource::Level(l) => l.clone(),
            DensitySource::Highlight(p) => format!("\"{}\"", p),
        }
    }

    // ── Filter application ──────────────────────────────────────

    /// Re-apply all active filters to compute filtered_indices.
    #[instrument(skip(self))]
    pub fn reapply_filters(&mut self) {
        // Remember the record index at the current cursor position
        let prev_record_idx = self.filtered_indices.get(self.selected).copied();

        self.filtered_indices = (0..self.records.len())
            .filter(|&i| {
                let record = &self.records[i];
                // Apply level filter first
                if let Some(ref lf) = self.level_filter {
                    if !lf.matches_level(record.level.as_ref()) {
                        return false;
                    }
                }
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

        // Preserve cursor: find the previous record or nearest preceding visible record
        let new_selected = if let Some(prev_idx) = prev_record_idx {
            if self.filtered_indices.is_empty() {
                0
            } else {
                // Use partition_point (binary search) since filtered_indices is sorted
                let pos = self.filtered_indices.partition_point(|&ri| ri <= prev_idx);
                if pos > 0 && self.filtered_indices[pos - 1] == prev_idx {
                    // Exact match — record is still visible
                    pos - 1
                } else if pos > 0 {
                    // Nearest preceding visible record
                    pos - 1
                } else {
                    // No preceding records — fall back to first row
                    0
                }
            }
        } else {
            0
        };

        self.selected = new_selected;
        // Clamp scroll_offset to keep selected row visible without guessing terminal size
        if self.scroll_offset > self.selected {
            self.scroll_offset = self.selected;
        }
        self.clear_search();
        self.filter_version += 1;
    }

    /// Apply filter expression from the `f` input mode.
    #[instrument(skip(self))]
    pub fn apply_filter(&mut self) {
        if self.filter_input.is_empty() {
            self.filter_error = None;
            return;
        }
        let filter_text = self.filter_input.value().to_string();
        tracing::info!(filter = %filter_text, "applying filter");
        match expr::parse(&filter_text) {
            Ok(parsed_expr) => {
                self.filters.push(FilterEntry {
                    label: self.filter_input.value().to_string(),
                    expr_str: self.filter_input.value().to_string(),
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
    #[instrument(skip(self))]
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
                    expr_str: expr_str.clone(),
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
    #[instrument(skip(self))]
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
                    expr_str: expr_str.clone(),
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
    /// Apply a level filter preset. None = clear level filter (show ALL).
    pub fn apply_level_filter(&mut self, preset: LevelFilterPreset) {
        if preset == LevelFilterPreset::All {
            self.level_filter = None;
        } else {
            self.level_filter = Some(preset);
        }
        self.reapply_filters();
        self.set_status(format!("Level filter: {}", preset.label()));
    }

    /// Save current filters as a named preset.
    pub fn save_filter_preset(&mut self, name: &str) {
        use crate::config::filter_preset::{save_preset, FilterPreset, FilterPresetEntry};

        let preset = FilterPreset {
            filters: self
                .filters
                .iter()
                .map(|f| FilterPresetEntry {
                    expr: f.expr_str.clone(),
                    exclude: f.exclude,
                })
                .collect(),
            level_filter: self.level_filter.map(|l| l.label().to_string()),
        };

        match save_preset(name, &preset) {
            Ok(_) => self.set_status(format!("Preset '{}' saved", name)),
            Err(e) => self.set_status(format!("Error saving preset: {}", e)),
        }
    }

    /// Load a filter preset by name, replacing current filters.
    pub fn load_filter_preset(&mut self, name: &str) {
        use crate::config::filter_preset::load_preset;

        let preset = match load_preset(name) {
            Ok(p) => p,
            Err(e) => {
                self.set_status(format!("Error loading preset: {}", e));
                return;
            }
        };

        // Clear existing filters
        self.filters.clear();

        // Load expression filters
        for entry in &preset.filters {
            match expr::parse(&entry.expr) {
                Ok(parsed) => {
                    self.filters.push(FilterEntry {
                        label: entry.expr.clone(),
                        expr_str: entry.expr.clone(),
                        expr: parsed,
                        exclude: entry.exclude,
                    });
                }
                Err(e) => {
                    self.set_status(format!(
                        "Warning: filter '{}' failed to parse: {}",
                        entry.expr, e
                    ));
                }
            }
        }

        // Load level filter
        if let Some(ref level_str) = preset.level_filter {
            match level_str.as_str() {
                "ALL" => self.level_filter = None,
                "TRACE+" => self.level_filter = Some(LevelFilterPreset::TracePlus),
                "DEBUG+" => self.level_filter = Some(LevelFilterPreset::DebugPlus),
                "INFO+" => self.level_filter = Some(LevelFilterPreset::InfoPlus),
                "NOTICE+" => self.level_filter = Some(LevelFilterPreset::NoticePlus),
                "WARN+" => self.level_filter = Some(LevelFilterPreset::WarnPlus),
                "ERROR+" => self.level_filter = Some(LevelFilterPreset::ErrorPlus),
                "FATAL" => self.level_filter = Some(LevelFilterPreset::FatalOnly),
                _ => {}
            }
        } else {
            self.level_filter = None;
        }

        self.reapply_filters();
        self.set_status(format!(
            "Preset '{}' loaded ({} filters)",
            name,
            self.filters.len()
        ));
    }

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
                        "{} == \"{}\"",
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
                        expr_str: part.to_string(),
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
                        expr_str: expr_str.clone(),
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
    #[instrument(skip(self))]
    pub fn clear_filters(&mut self) {
        tracing::info!("clearing all filters");
        self.filters.clear();
        self.reapply_filters();
    }

    // ── Search ──────────────────────────────────────────────────

    /// Execute regex search across filtered records.
    #[instrument(skip(self))]
    pub fn execute_search(&mut self) {
        if self.search_input.is_empty() {
            self.clear_search();
            return;
        }
        tracing::info!(pattern = %self.search_input.value(), "executing search");
        let pattern = match regex::RegexBuilder::new(self.search_input.value())
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

        self.search_regex = Some(pattern.clone());
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
        self.search_regex = None;
    }

    // ── Navigation ──────────────────────────────────────────────

    /// Toggle bookmark on the currently selected record.
    #[instrument(skip(self))]
    pub fn toggle_bookmark(&mut self) {
        if let Some(&ri) = self.filtered_indices.get(self.selected) {
            let id = self.records[ri].id;
            if !self.bookmarks.remove(&id) {
                self.bookmarks.insert(id);
                tracing::debug!(
                    record_id = id,
                    total = self.bookmarks.len(),
                    "bookmark added"
                );
                self.set_status(format!("Bookmark added (total: {})", self.bookmarks.len()));
            } else {
                tracing::debug!(
                    record_id = id,
                    total = self.bookmarks.len(),
                    "bookmark removed"
                );
                self.set_status(format!(
                    "Bookmark removed (total: {})",
                    self.bookmarks.len()
                ));
            }
        }
    }

    /// Jump to the next bookmarked record (cyclic).
    pub fn jump_next_bookmark(&mut self) {
        if self.bookmarks.is_empty() || self.filtered_indices.is_empty() {
            self.set_status("No bookmarks".to_string());
            return;
        }
        let len = self.filtered_indices.len();
        let start = self.selected + 1;
        for offset in 0..len {
            let fi = (start + offset) % len;
            let ri = self.filtered_indices[fi];
            if self.bookmarks.contains(&self.records[ri].id) {
                self.selected = fi;
                self.ensure_selected_visible();
                self.set_status(format!("Bookmark {}", fi + 1));
                return;
            }
        }
    }

    /// Jump to the previous bookmarked record (cyclic).
    pub fn jump_prev_bookmark(&mut self) {
        if self.bookmarks.is_empty() || self.filtered_indices.is_empty() {
            self.set_status("No bookmarks".to_string());
            return;
        }
        let len = self.filtered_indices.len();
        let start = if self.selected == 0 {
            len - 1
        } else {
            self.selected - 1
        };
        for offset in 0..len {
            let fi = (start + len - offset) % len;
            let ri = self.filtered_indices[fi];
            if self.bookmarks.contains(&self.records[ri].id) {
                self.selected = fi;
                self.ensure_selected_visible();
                self.set_status(format!("Bookmark {}", fi + 1));
                return;
            }
        }
    }

    /// Get sorted list of bookmarked filtered indices for the manager.
    pub fn bookmarked_filtered_indices(&self) -> Vec<usize> {
        self.filtered_indices
            .iter()
            .enumerate()
            .filter(|(_, &ri)| self.bookmarks.contains(&self.records[ri].id))
            .map(|(fi, _)| fi)
            .collect()
    }

    /// Check if a record index is bookmarked.
    pub fn is_bookmarked(&self, record_idx: usize) -> bool {
        self.bookmarks.contains(&self.records[record_idx].id)
    }

    /// Parse a relative duration string like "5m", "30s", "2h", "1d".
    /// Returns the duration in seconds, or None if invalid.
    /// Parse a relative duration string and return the total in **milliseconds**.
    ///
    /// Supported suffixes: `ms` (milliseconds), `s` (seconds), `m` (minutes),
    /// `h` (hours), `d` (days). Compound forms like `1h30m`, `2m30s`, `500ms`
    /// are all valid.
    fn parse_relative_duration(input: &str) -> Option<i64> {
        let input = input.trim();
        if input.is_empty() {
            return None;
        }

        let mut total: i64 = 0;
        let mut num_buf = String::new();
        let mut found_any = false;
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];
            if ch.is_ascii_digit() {
                num_buf.push(ch);
                i += 1;
            } else {
                if num_buf.is_empty() {
                    return None;
                }
                let value: i64 = num_buf.parse().ok()?;
                num_buf.clear();

                // Check for two-char suffix "ms"
                if ch == 'm' && i + 1 < chars.len() && chars[i + 1] == 's' {
                    total = total.checked_add(value)?;
                    found_any = true;
                    i += 2;
                } else {
                    let ms = match ch {
                        's' => value.checked_mul(1_000)?,
                        'm' => value.checked_mul(60_000)?,
                        'h' => value.checked_mul(3_600_000)?,
                        'd' => value.checked_mul(86_400_000)?,
                        _ => return None,
                    };
                    total = total.checked_add(ms)?;
                    found_any = true;
                    i += 1;
                }
            }
        }

        // Handle trailing number without suffix (not valid)
        if !num_buf.is_empty() || !found_any {
            return None;
        }

        if total <= 0 {
            return None;
        }

        Some(total)
    }

    /// Format seconds as a human-readable relative duration.
    fn format_duration_secs(secs: i64) -> String {
        let abs = secs.unsigned_abs();
        if abs >= 86400 && abs.is_multiple_of(86400) {
            format!("{}d", abs / 86400)
        } else if abs >= 3600 && abs.is_multiple_of(3600) {
            format!("{}h", abs / 3600)
        } else if abs >= 60 && abs.is_multiple_of(60) {
            format!("{}m", abs / 60)
        } else {
            format!("{}s", abs)
        }
    }

    /// Jump forward (`forward=true`) or backward (`forward=false`) by relative duration.
    /// Returns `true` if the jump succeeded, `false` on invalid input.
    pub fn jump_relative(&mut self, forward: bool) -> bool {
        let input = self.time_input.trim().to_string();
        if input.is_empty() {
            return false;
        }

        let ms = match Self::parse_relative_duration(&input) {
            Some(v) => v,
            None => {
                self.set_status("Invalid duration (use Nms, Ns, Nm, Nh, Nd)".to_string());
                return false;
            }
        };

        if self.filtered_indices.is_empty() {
            self.set_status("No records".to_string());
            return false;
        }

        let current_ri = self.filtered_indices[self.selected];
        let current_ts = self.records[current_ri].timestamp;
        let delta = chrono::Duration::milliseconds(if forward { ms } else { -ms });
        let target_ts = current_ts + delta;

        // Binary search filtered_indices for the closest row to target_ts
        let fi_len = self.filtered_indices.len();
        let mut lo: usize = 0;
        let mut hi: usize = fi_len;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let ri = self.filtered_indices[mid];
            if self.records[ri].timestamp < target_ts {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }

        // lo is the first index with timestamp >= target_ts
        // Pick the closest between lo and lo-1
        let best = if lo == 0 {
            0
        } else if lo >= fi_len {
            fi_len - 1
        } else {
            let ri_lo = self.filtered_indices[lo];
            let ri_prev = self.filtered_indices[lo - 1];
            let diff_lo = (self.records[ri_lo].timestamp - target_ts)
                .num_milliseconds()
                .unsigned_abs();
            let diff_prev = (target_ts - self.records[ri_prev].timestamp)
                .num_milliseconds()
                .unsigned_abs();
            if diff_prev <= diff_lo {
                lo - 1
            } else {
                lo
            }
        };

        let actual_ri = self.filtered_indices[best];
        let actual_ts = self.records[actual_ri].timestamp;
        let actual_diff = (actual_ts - current_ts).num_seconds();
        let direction = if forward { "→+" } else { "→-" };
        let actual_str = Self::format_duration_secs(actual_diff);

        self.selected = best;
        self.ensure_selected_visible();
        self.set_status(format!(
            "Jumped {} (actual {}{})",
            input, direction, actual_str
        ));
        true
    }

    #[instrument(skip(self))]
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
        self.follow_new_count = 0;
    }

    pub fn toggle_detail(&mut self) {
        self.panel_state
            .toggle_expand(crate::panel::PanelId::Detail);
        tracing::debug!(
            detail_open = self
                .panel_state
                .is_panel_open(crate::panel::PanelId::Detail),
            "toggled detail panel (focus unchanged)"
        );
    }

    /// Get the flattened tree nodes for the current record.
    fn detail_flat_nodes(&self) -> Vec<crate::ui::widgets::detail_panel_widget::FlatNode> {
        if let Some(record) = self.selected_record() {
            if let Some(expanded) = record.expanded.as_ref() {
                return crate::ui::widgets::detail_panel_widget::flatten_expanded(
                    expanded,
                    &self.detail_tree_collapsed,
                );
            }
        }
        Vec::new()
    }

    /// Move tree cursor down.
    pub fn detail_tree_move_down(&mut self) {
        let nodes = self.detail_flat_nodes();
        if self.detail_tree_cursor + 1 < nodes.len() {
            self.detail_tree_cursor += 1;
        }
    }

    /// Move tree cursor up.
    pub fn detail_tree_move_up(&mut self) {
        if self.detail_tree_cursor > 0 {
            self.detail_tree_cursor -= 1;
        }
    }

    /// Toggle expand/collapse on current tree node.
    pub fn detail_tree_toggle(&mut self) {
        let nodes = self.detail_flat_nodes();
        if let Some(node) = nodes.get(self.detail_tree_cursor) {
            if node.collapsible {
                let key = node.path_key.clone();
                if node.collapsed {
                    self.detail_tree_collapsed.remove(&key);
                } else {
                    self.detail_tree_collapsed.insert(key);
                }
            }
        }
    }

    /// Collapse current node or move to parent.
    pub fn detail_tree_collapse_or_parent(&mut self) {
        let nodes = self.detail_flat_nodes();
        if let Some(node) = nodes.get(self.detail_tree_cursor) {
            if node.collapsible && !node.collapsed {
                // Collapse this node
                self.detail_tree_collapsed.insert(node.path_key.clone());
            } else {
                // Move to parent: find nearest node with lower depth
                let current_depth = node.depth;
                if current_depth > 0 {
                    for i in (0..self.detail_tree_cursor).rev() {
                        if nodes[i].depth < current_depth {
                            self.detail_tree_cursor = i;
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Collapse all tree nodes.
    pub fn detail_tree_collapse_all(&mut self) {
        // First expand all, then flatten to get all collapsible paths.
        self.detail_tree_collapsed.clear();
        let all_nodes = self.detail_flat_nodes();
        for node in &all_nodes {
            if node.collapsible {
                self.detail_tree_collapsed.insert(node.path_key.clone());
            }
        }
        // Clamp cursor
        let new_nodes = self.detail_flat_nodes();
        if self.detail_tree_cursor >= new_nodes.len() {
            self.detail_tree_cursor = new_nodes.len().saturating_sub(1);
        }
    }

    /// Expand all tree nodes.
    pub fn detail_tree_expand_all(&mut self) {
        self.detail_tree_collapsed.clear();
    }

    /// Scroll detail panel right by a few columns.
    pub fn detail_scroll_right(&mut self) {
        self.detail_horizontal_offset = self.detail_horizontal_offset.saturating_add(4);
    }

    /// Scroll detail panel left by a few columns.
    pub fn detail_scroll_left(&mut self) {
        self.detail_horizontal_offset = self.detail_horizontal_offset.saturating_sub(4);
    }

    /// Create a quick filter from the current tree leaf node.
    pub fn detail_tree_quick_filter(&mut self) {
        let nodes = self.detail_flat_nodes();
        if let Some(node) = nodes.get(self.detail_tree_cursor) {
            if let Some(ref expr_str) = node.filter_expr {
                match expr::parse(expr_str) {
                    Ok(parsed_expr) => {
                        self.filters.push(FilterEntry {
                            label: expr_str.clone(),
                            expr_str: expr_str.clone(),
                            expr: parsed_expr,
                            exclude: false,
                        });
                        self.reapply_filters();
                        self.set_status(format!("Filter added: {}", expr_str));
                    }
                    Err(e) => {
                        self.set_status(format!("Filter error: {}", e));
                    }
                }
            }
        }
    }

    /// Toggle follow mode.
    /// Disable follow mode (one-way). Cannot re-enable from TUI.
    pub fn toggle_follow(&mut self) {
        if self.follow_mode {
            self.follow_mode = false;
            self.follow_new_count = 0;
        }
        // No-op if already disabled — cannot re-enable from TUI
    }

    /// Clear all records (for follow mode reload after truncation/rotation).
    pub fn clear_records(&mut self) {
        self.records.clear();
        self.total_records = 0;
        self.filtered_indices.clear();
        self.selected = 0;
        self.scroll_offset = 0;
        self.follow_new_count = 0;
        self.density_cache = None;
        // Reset category stats
        if let Some(ref mut proc) = self.category_processor {
            for cat in &mut proc.store.categories {
                cat.count = 0;
                cat.density.fill(0);
            }
        }
    }

    /// Append new records from follow mode. Incrementally updates
    /// filtered_indices and auto-scrolls if follow_mode is active.
    pub fn append_records(&mut self, new_records: Vec<Arc<LogRecord>>) {
        if new_records.is_empty() {
            return;
        }

        let was_at_bottom = self.follow_mode
            && (self.filtered_indices.is_empty()
                || self.selected + 1 >= self.filtered_indices.len());

        let base_idx = self.records.len();
        let count = new_records.len();
        self.records.extend(new_records);
        self.total_records = self.records.len();

        // Incrementally filter new records (don't re-filter all)
        let filtered_before = self.filtered_indices.len();
        for i in base_idx..base_idx + count {
            let record = &self.records[i];

            // Level filter
            if let Some(ref lf) = self.level_filter {
                if !lf.matches_level(record.level.as_ref()) {
                    continue;
                }
            }

            // Include/exclude filters
            let mut pass = true;
            for f in &self.filters {
                let matches = eval::eval(&f.expr, record);
                if f.exclude && matches {
                    pass = false;
                    break;
                }
                if !f.exclude && !matches {
                    pass = false;
                    break;
                }
            }

            if pass {
                self.filtered_indices.push(i);
            }
        }

        // Recompute column widths
        self.col_widths = Self::compute_col_widths(&self.records, &self.filtered_indices);

        // Incrementally update search matches for new filtered records
        if let Some(ref regex) = self.search_regex {
            for fi in filtered_before..self.filtered_indices.len() {
                let ri = self.filtered_indices[fi];
                let record = &self.records[ri];
                if regex.is_match(&record.message) || regex.is_match(&record.raw) {
                    self.search_matches.push(fi);
                }
            }
        }

        // Auto-scroll if in follow mode and was at bottom
        if was_at_bottom && !self.filtered_indices.is_empty() {
            self.selected = self.filtered_indices.len() - 1;
            self.ensure_selected_visible();
            self.follow_new_count = 0;
        } else if self.follow_mode && !self.filtered_indices.is_empty() {
            // User has scrolled up — track how many records are below cursor
            self.follow_new_count = self
                .filtered_indices
                .len()
                .saturating_sub(self.selected + 1);
        }

        tracing::debug!(
            new = count,
            total = self.total_records,
            filtered = self.filtered_indices.len(),
            "records appended"
        );
    }

    /// Called on manual scroll up — does NOT disable follow mode,
    /// just stops auto-scrolling. New records continue to load.
    pub fn exit_follow(&mut self) {
        // Don't disable follow_mode — that's done by F key only.
        // Auto-scroll is controlled by cursor position, not a separate flag.
    }

    pub fn ensure_selected_visible(&mut self) {
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + self.visible_rows {
            self.scroll_offset = self.selected.saturating_sub(self.visible_rows - 1);
        }
    }

    /// Jump to a record by its original index in the records array.
    /// Finds the corresponding filtered index and selects it.
    pub fn jump_to_record_index(&mut self, record_idx: usize) {
        if let Some(filtered_idx) = self.filtered_indices.iter().position(|&i| i == record_idx) {
            self.selected = filtered_idx;
            self.ensure_selected_visible();
        }
    }

    /// Add a filter expression string.
    pub fn add_filter_expr(&mut self, expr: &str) {
        self.filter_input = crate::text_input::TextInput::with_text(expr);
        self.apply_filter();
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

    /// Execute a command entered in command mode.
    pub fn execute_command(&mut self) {
        let input = self.command_input.trim().to_string();
        if input.is_empty() {
            return;
        }

        if input == "q" {
            self.should_quit = true;
        } else {
            self.set_status(format!("Unknown command: :{}", input));
        }
    }

    /// Add a highlight rule with the given regex pattern.
    /// Returns Ok(()) on success or Err with the regex error message.
    pub fn add_highlight_rule(&mut self, pattern: &str) -> Result<(), String> {
        if pattern.is_empty() {
            return Err("Empty pattern".to_string());
        }
        match Regex::new(pattern) {
            Ok(regex) => {
                let palette = &self.theme.highlight_palette;
                let palette_len = palette.len().max(1);
                let used_colors: std::collections::HashSet<usize> = self
                    .highlight_rules
                    .iter()
                    .filter_map(|r| palette.iter().position(|c| c.0 == r.color))
                    .collect();
                let color_idx = (0..palette_len)
                    .find(|i| !used_colors.contains(i))
                    .unwrap_or(self.highlight_rules.len() % palette_len);
                let color = palette.get(color_idx).map(|c| c.0).unwrap_or(Color::Red);
                self.highlight_rules.push(HighlightRule {
                    pattern: pattern.to_string(),
                    regex,
                    color,
                });
                self.set_status(format!("Highlight added: {}", pattern));
                Ok(())
            }
            Err(e) => Err(format!("Invalid regex: {}", e)),
        }
    }

    /// Remove highlight rule at index.
    pub fn remove_highlight_rule(&mut self, idx: usize) {
        if idx < self.highlight_rules.len() {
            let removed = self.highlight_rules.remove(idx);
            self.set_status(format!("Removed highlight: {}", removed.pattern));
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CopyFormat {
    Raw,
    Json,
    Yaml,
}

/// Export format for saving logs to file.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
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
            expanded: None,
        }
    }

    fn make_record_with_ts(id: u64, ts: chrono::DateTime<Utc>) -> LogRecord {
        LogRecord {
            id,
            timestamp: ts,
            level: Some(LogLevel::Info),
            source: "test".into(),
            pid: None,
            tid: None,
            component_name: None,
            process_name: None,
            message: format!("msg {}", id),
            hostname: None,
            container: None,
            context: None,
            function: None,
            raw: format!("msg {}", id),
            metadata: None,
            loader_id: "test".into(),
            expanded: None,
        }
    }

    fn make_app_with_timestamps(n: usize) -> App {
        let base = Utc::now();
        let records: Vec<Arc<LogRecord>> = (0..n)
            .map(|i| {
                Arc::new(make_record_with_ts(
                    i as u64,
                    base + chrono::Duration::seconds(i as i64),
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
            detail_panel_ratio: 0.3,
            detail_tree_cursor: 0,
            detail_tree_collapsed: std::collections::HashSet::new(),
            detail_horizontal_offset: 0,
            panel_state: crate::panel::PanelState::default(),
            input_mode: InputMode::Normal,
            filter_input: TextInput::new(),
            filter_error: None,
            filters: Vec::new(),
            quick_filter_input: TextInput::new(),
            field_filter: None,
            filter_manager_cursor: 0,
            search_input: TextInput::new(),
            search_matches: vec![],
            search_regex: None,
            search_match_idx: None,
            time_input: TextInput::new(),
            goto_input: TextInput::new(),
            status_message: None,
            shortcut_hints_cache: Vec::new(),
            status_message_at: None,
            col_widths: [19, 5, 11, 3, 3, 9, 7, 8],
            column_config: ColumnConfig::default(),
            follow_mode: false,
            follow_new_count: 0,
            should_quit: false,
            copy_format_cursor: 0,
            save_path_input: TextInput::with_text("./scouty-export.log"),
            save_format_cursor: 0,
            save_dialog_focus: crate::ui::windows::save_dialog_window::Focus::Path,
            help_scroll: 0,
            command_input: TextInput::new(),
            filter_version: 0,
            density_cache: None,
            highlight_rules: Vec::new(),
            highlight_input: TextInput::new(),
            highlight_manager_cursor: 0,
            bookmarks: std::collections::HashSet::new(),
            bookmark_manager_cursor: 0,
            theme: Theme::default(),
            level_filter: None,
            level_filter_cursor: 0,
            preset_name_input: TextInput::new(),
            preset_list: Vec::new(),
            preset_list_cursor: 0,
            density_source: DensitySource::All,
            density_selector_cursor: 0,
            regions: scouty::region::store::RegionStore::default(),
            region_processor: None,
            category_processor: None,
            category_cursor: 0,
            region_manager_cursor: 0,
            region_panel_sort: crate::ui::widgets::region_panel_widget::RegionSortMode::StartTime,
            region_panel_type_filter: None,
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
            detail_panel_ratio: 0.3,
            detail_tree_cursor: 0,
            detail_tree_collapsed: std::collections::HashSet::new(),
            detail_horizontal_offset: 0,
            panel_state: crate::panel::PanelState::default(),
            input_mode: InputMode::Normal,
            filter_input: TextInput::new(),
            filter_error: None,
            filters: Vec::new(),
            quick_filter_input: TextInput::new(),
            field_filter: None,
            filter_manager_cursor: 0,
            search_input: TextInput::new(),
            search_matches: vec![],
            search_regex: None,
            search_match_idx: None,
            time_input: TextInput::new(),
            goto_input: TextInput::new(),
            status_message: None,
            shortcut_hints_cache: Vec::new(),
            status_message_at: None,
            col_widths: [19, 5, 11, 3, 3, 9, 7, 8],
            column_config: ColumnConfig::default(),
            follow_mode: false,
            follow_new_count: 0,
            should_quit: false,
            copy_format_cursor: 0,
            save_path_input: TextInput::with_text("./scouty-export.log"),
            save_format_cursor: 0,
            save_dialog_focus: crate::ui::windows::save_dialog_window::Focus::Path,
            help_scroll: 0,
            command_input: TextInput::new(),
            filter_version: 0,
            density_cache: None,
            highlight_rules: Vec::new(),
            highlight_input: TextInput::new(),
            highlight_manager_cursor: 0,
            bookmarks: std::collections::HashSet::new(),
            bookmark_manager_cursor: 0,
            theme: Theme::default(),
            level_filter: None,
            level_filter_cursor: 0,
            preset_name_input: TextInput::new(),
            preset_list: Vec::new(),
            preset_list_cursor: 0,
            density_source: DensitySource::All,
            density_selector_cursor: 0,
            regions: scouty::region::store::RegionStore::default(),
            region_processor: None,
            category_processor: None,
            category_cursor: 0,
            region_manager_cursor: 0,
            region_panel_sort: crate::ui::widgets::region_panel_widget::RegionSortMode::StartTime,
            region_panel_type_filter: None,
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
            detail_panel_ratio: 0.3,
            detail_tree_cursor: 0,
            detail_tree_collapsed: std::collections::HashSet::new(),
            detail_horizontal_offset: 0,
            panel_state: crate::panel::PanelState::default(),
            input_mode: InputMode::Normal,
            filter_input: TextInput::new(),
            filter_error: None,
            filters: Vec::new(),
            quick_filter_input: TextInput::new(),
            field_filter: None,
            filter_manager_cursor: 0,
            search_input: TextInput::new(),
            search_matches: vec![],
            search_regex: None,
            search_match_idx: None,
            time_input: TextInput::new(),
            goto_input: TextInput::new(),
            status_message: None,
            shortcut_hints_cache: Vec::new(),
            status_message_at: None,
            col_widths: [19, 5, 11, 3, 3, 9, 7, 8],
            column_config: ColumnConfig::default(),
            follow_mode: false,
            follow_new_count: 0,
            should_quit: false,
            copy_format_cursor: 0,
            save_path_input: TextInput::with_text("./scouty-export.log"),
            save_format_cursor: 0,
            save_dialog_focus: crate::ui::windows::save_dialog_window::Focus::Path,
            help_scroll: 0,
            command_input: TextInput::new(),
            filter_version: 0,
            density_cache: None,
            highlight_rules: Vec::new(),
            highlight_input: TextInput::new(),
            highlight_manager_cursor: 0,
            bookmarks: std::collections::HashSet::new(),
            bookmark_manager_cursor: 0,
            theme: Theme::default(),
            level_filter: None,
            level_filter_cursor: 0,
            preset_name_input: TextInput::new(),
            preset_list: Vec::new(),
            preset_list_cursor: 0,
            density_source: DensitySource::All,
            density_selector_cursor: 0,
            regions: scouty::region::store::RegionStore::default(),
            region_processor: None,
            category_processor: None,
            category_cursor: 0,
            region_manager_cursor: 0,
            region_panel_sort: crate::ui::widgets::region_panel_widget::RegionSortMode::StartTime,
            region_panel_type_filter: None,
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
        assert!(!app.panel_state.is_panel_open(crate::panel::PanelId::Detail));
        app.toggle_detail();
        assert!(app.panel_state.is_panel_open(crate::panel::PanelId::Detail));
        app.toggle_detail();
        assert!(!app.panel_state.is_panel_open(crate::panel::PanelId::Detail));
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
        app.search_input.set("error");
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
        app.search_input.set("zzzznotfound");
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

        app.search_input.set(r"error.*(?:timeout|full)");
        app.execute_search();
        assert_eq!(app.search_matches.len(), 2);

        app.search_input.set("[invalid");
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
        app.goto_input.set("50");
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

        app.quick_filter_input.set("timeout");
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

        app.quick_filter_input.set("timeout");
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
        app.quick_filter_input.set("timeout");
        app.apply_quick_exclude();
        assert_eq!(app.filtered_indices.len(), 3);

        // Also exclude "disk"
        app.quick_filter_input.set("disk");
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

        app.quick_filter_input.set("timeout");
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

        app.quick_filter_input.set("a");
        app.apply_quick_exclude();
        app.quick_filter_input.set("b");
        app.apply_quick_exclude();
        assert_eq!(app.filtered_indices.len(), 1);

        app.clear_filters();
        assert_eq!(app.filters.len(), 0);
        assert_eq!(app.filtered_indices.len(), 3);
    }

    #[test]
    fn test_filter_preserves_cursor_position() {
        let mut app = make_app_with_levels(&[
            ("a", Some(LogLevel::Error)), // idx 0
            ("b", Some(LogLevel::Info)),  // idx 1
            ("c", Some(LogLevel::Warn)),  // idx 2
            ("d", Some(LogLevel::Error)), // idx 3
            ("e", Some(LogLevel::Info)),  // idx 4
        ]);

        // Move cursor to record "c" (filtered index 2, record index 2)
        app.selected = 2;

        // Exclude messages containing "b" and "e" → remaining: a(0), c(2), d(3)
        app.quick_filter_input.set("b");
        app.apply_quick_exclude();
        app.quick_filter_input.set("e");
        app.apply_quick_exclude();
        // Cursor was on record "c" (idx 2), still visible → stays at filtered index 1
        assert_eq!(app.filtered_indices, vec![0, 2, 3]);
        assert_eq!(
            app.selected, 1,
            "cursor should stay on record 'c' at filtered index 1"
        );

        // Now exclude "c" too → remaining: a(0), d(3)
        // Record "c" filtered out, nearest preceding visible is "a" → filtered index 0
        app.quick_filter_input.set("c");
        app.apply_quick_exclude();
        assert_eq!(app.filtered_indices, vec![0, 3]);
        assert_eq!(
            app.selected, 0,
            "cursor should move to nearest preceding record 'a'"
        );
    }

    #[test]
    fn test_filter_preserves_cursor_no_preceding() {
        let mut app = make_app_with_levels(&[
            ("a", Some(LogLevel::Error)), // idx 0
            ("b", Some(LogLevel::Info)),  // idx 1
            ("c", Some(LogLevel::Warn)),  // idx 2
        ]);

        // Cursor on first record "a" (idx 0)
        app.selected = 0;

        // Exclude "a" → remaining: b(1), c(2)
        // No preceding records before idx 0 → cursor goes to first row (0)
        app.quick_filter_input.set("a");
        app.apply_quick_exclude();
        assert_eq!(app.filtered_indices, vec![1, 2]);
        assert_eq!(
            app.selected, 0,
            "cursor should go to first row when no preceding records exist"
        );
    }

    #[test]
    fn test_filter_expression() {
        let mut app = make_app_with_levels(&[
            ("error msg", Some(LogLevel::Error)),
            ("info msg", Some(LogLevel::Info)),
            ("warn msg", Some(LogLevel::Warn)),
        ]);

        app.filter_input.set(r#"level == "ERROR""#);
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

    // ── Density cache tests ─────────────────────────────────────

    #[test]
    fn test_density_cache_built_on_first_call() {
        let mut app = make_app(50);
        assert!(app.density_cache.is_none());
        let result = app.get_density_cache(40);
        assert!(result.is_some());
        assert!(app.density_cache.is_some());
        let cache = app.density_cache.as_ref().unwrap();
        assert_eq!(cache.chart_width, 40);
        assert_eq!(cache.filter_version, 0);
        assert!(!cache.braille_text.is_empty());
    }

    #[test]
    fn test_density_cache_reused_on_same_params() {
        let mut app = make_app(50);
        app.get_density_cache(40);
        let v1 = app.density_cache.as_ref().unwrap().filter_version;
        app.get_density_cache(40);
        let v2 = app.density_cache.as_ref().unwrap().filter_version;
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_density_cache_invalidated_on_filter_change() {
        let mut app = make_app(50);
        app.get_density_cache(40);
        let text1 = app.density_cache.as_ref().unwrap().braille_text.clone();
        app.filter_version += 1;
        app.get_density_cache(40);
        assert_eq!(app.density_cache.as_ref().unwrap().filter_version, 1);
        assert_eq!(app.density_cache.as_ref().unwrap().braille_text, text1);
    }

    #[test]
    fn test_density_cache_invalidated_on_width_change() {
        let mut app = make_app(50);
        app.get_density_cache(40);
        assert_eq!(app.density_cache.as_ref().unwrap().chart_width, 40);
        app.get_density_cache(60);
        assert_eq!(app.density_cache.as_ref().unwrap().chart_width, 60);
    }

    #[test]
    fn test_density_cache_none_for_empty() {
        let mut app = make_app(0);
        assert!(app.get_density_cache(40).is_none());
    }

    #[test]
    fn test_density_cache_zero_width() {
        let mut app = make_app(10);
        assert!(app.get_density_cache(0).is_none());
    }

    #[test]
    fn test_cursor_char_in_density_at_start() {
        let mut app = make_app_with_timestamps(100);
        app.selected = 0;
        app.get_density_cache(40);
        let idx = app.cursor_char_in_density();
        assert_eq!(idx, Some(0));
    }

    #[test]
    fn test_cursor_char_in_density_at_end() {
        let mut app = make_app_with_timestamps(100);
        app.selected = app.filtered_indices.len() - 1;
        app.get_density_cache(40);
        let idx = app.cursor_char_in_density();
        assert!(idx.is_some());
        let max_char = app
            .density_cache
            .as_ref()
            .unwrap()
            .braille_text
            .chars()
            .count()
            .saturating_sub(1);
        assert_eq!(idx.unwrap(), max_char);
    }

    #[test]
    fn test_cursor_char_in_density_at_middle() {
        let mut app = make_app_with_timestamps(100);
        app.selected = 50;
        app.get_density_cache(40);
        let idx = app.cursor_char_in_density().unwrap();
        let max_char = app
            .density_cache
            .as_ref()
            .unwrap()
            .braille_text
            .chars()
            .count()
            - 1;
        assert!(
            idx > 0 && idx < max_char,
            "Expected middle, got {}/{}",
            idx,
            max_char
        );
    }

    #[test]
    fn test_detail_scroll_right_increases_offset() {
        let mut app = make_app(5);
        assert_eq!(app.detail_horizontal_offset, 0);
        app.detail_scroll_right();
        assert_eq!(app.detail_horizontal_offset, 4);
        app.detail_scroll_right();
        assert_eq!(app.detail_horizontal_offset, 8);
    }

    #[test]
    fn test_detail_scroll_left_decreases_offset() {
        let mut app = make_app(5);
        app.detail_horizontal_offset = 8;
        app.detail_scroll_left();
        assert_eq!(app.detail_horizontal_offset, 4);
        app.detail_scroll_left();
        assert_eq!(app.detail_horizontal_offset, 0);
    }

    #[test]
    fn test_detail_scroll_left_does_not_underflow() {
        let mut app = make_app(5);
        assert_eq!(app.detail_horizontal_offset, 0);
        app.detail_scroll_left();
        assert_eq!(app.detail_horizontal_offset, 0);
    }

    #[test]
    fn test_detail_scroll_left_partial_saturate() {
        let mut app = make_app(5);
        app.detail_horizontal_offset = 2;
        app.detail_scroll_left();
        assert_eq!(app.detail_horizontal_offset, 0);
    }

    #[test]
    fn test_highlight_color_no_duplicate_after_delete() {
        let mut app = make_app(0);
        // Add two highlight rules — they should get different colors.
        app.add_highlight_rule("aaa").unwrap();
        app.add_highlight_rule("bbb").unwrap();
        let color0 = app.highlight_rules[0].color;
        let color1 = app.highlight_rules[1].color;
        assert_ne!(color0, color1, "initial rules should have different colors");

        // Remove the first rule, then add a new one.
        app.remove_highlight_rule(0);
        assert_eq!(app.highlight_rules.len(), 1);
        app.add_highlight_rule("ccc").unwrap();

        // The new rule must NOT collide with the remaining rule.
        let remaining_color = app.highlight_rules[0].color;
        let new_color = app.highlight_rules[1].color;
        assert_ne!(
            remaining_color, new_color,
            "new highlight should not reuse the color of the remaining rule"
        );
    }

    #[test]
    fn test_highlight_color_reuses_freed_slot() {
        let mut app = make_app(0);
        app.add_highlight_rule("aaa").unwrap();
        let first_color = app.highlight_rules[0].color;

        app.add_highlight_rule("bbb").unwrap();
        // Remove the first rule to free its palette slot.
        app.remove_highlight_rule(0);
        app.add_highlight_rule("ccc").unwrap();

        // The freed color (first_color) should be reused since it's the lowest free index.
        let reused_color = app.highlight_rules[1].color;
        assert_eq!(
            first_color, reused_color,
            "should reuse the freed palette slot"
        );
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
            expanded: None,
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
            detail_panel_ratio: 0.3,
            detail_tree_cursor: 0,
            detail_tree_collapsed: std::collections::HashSet::new(),
            detail_horizontal_offset: 0,
            panel_state: crate::panel::PanelState::default(),
            input_mode: InputMode::Normal,
            filter_input: TextInput::new(),
            filter_error: None,
            filters: Vec::new(),
            quick_filter_input: TextInput::new(),
            field_filter: None,
            filter_manager_cursor: 0,
            search_input: TextInput::new(),
            search_matches: vec![],
            search_regex: None,
            search_match_idx: None,
            time_input: TextInput::new(),
            goto_input: TextInput::new(),
            status_message: None,
            shortcut_hints_cache: Vec::new(),
            status_message_at: None,
            col_widths: [19, 5, 11, 3, 3, 9, 7, 8],
            column_config: ColumnConfig::default(),
            follow_mode: false,
            follow_new_count: 0,
            should_quit: false,
            copy_format_cursor: 0,
            save_path_input: TextInput::with_text("./scouty-export.log"),
            save_format_cursor: 0,
            save_dialog_focus: crate::ui::windows::save_dialog_window::Focus::Path,
            help_scroll: 0,
            command_input: TextInput::new(),
            filter_version: 0,
            density_cache: None,
            highlight_rules: Vec::new(),
            highlight_input: TextInput::new(),
            highlight_manager_cursor: 0,
            bookmarks: std::collections::HashSet::new(),
            bookmark_manager_cursor: 0,
            theme: Theme::default(),
            level_filter: None,
            level_filter_cursor: 0,
            preset_name_input: TextInput::new(),
            preset_list: Vec::new(),
            preset_list_cursor: 0,
            density_source: DensitySource::All,
            density_selector_cursor: 0,
            regions: scouty::region::store::RegionStore::default(),
            region_processor: None,
            category_processor: None,
            category_cursor: 0,
            region_manager_cursor: 0,
            region_panel_sort: crate::ui::widgets::region_panel_widget::RegionSortMode::StartTime,
            region_panel_type_filter: None,
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
            expanded: None,
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
            detail_panel_ratio: 0.3,
            detail_tree_cursor: 0,
            detail_tree_collapsed: std::collections::HashSet::new(),
            detail_horizontal_offset: 0,
            panel_state: crate::panel::PanelState::default(),
            input_mode: InputMode::Normal,
            filter_input: TextInput::new(),
            filter_error: None,
            filters: Vec::new(),
            quick_filter_input: TextInput::new(),
            field_filter: None,
            filter_manager_cursor: 0,
            search_input: TextInput::new(),
            search_matches: vec![],
            search_regex: None,
            search_match_idx: None,
            time_input: TextInput::new(),
            goto_input: TextInput::new(),
            status_message: None,
            shortcut_hints_cache: Vec::new(),
            status_message_at: None,
            col_widths: [19, 5, 11, 3, 3, 9, 7, 8],
            column_config: ColumnConfig::default(),
            follow_mode: false,
            follow_new_count: 0,
            should_quit: false,
            copy_format_cursor: 0,
            save_path_input: TextInput::with_text("./scouty-export.log"),
            save_format_cursor: 0,
            save_dialog_focus: crate::ui::windows::save_dialog_window::Focus::Path,
            help_scroll: 0,
            command_input: TextInput::new(),
            filter_version: 0,
            density_cache: None,
            highlight_rules: Vec::new(),
            highlight_input: TextInput::new(),
            highlight_manager_cursor: 0,
            bookmarks: std::collections::HashSet::new(),
            bookmark_manager_cursor: 0,
            theme: Theme::default(),
            level_filter: None,
            level_filter_cursor: 0,
            preset_name_input: TextInput::new(),
            preset_list: Vec::new(),
            preset_list_cursor: 0,
            density_source: DensitySource::All,
            density_selector_cursor: 0,
            regions: scouty::region::store::RegionStore::default(),
            region_processor: None,
            category_processor: None,
            category_cursor: 0,
            region_manager_cursor: 0,
            region_panel_sort: crate::ui::widgets::region_panel_widget::RegionSortMode::StartTime,
            region_panel_type_filter: None,
        }
    }

    // ── Column config tests ──────────────────────────────────

    #[test]
    fn test_default_column_config() {
        let config = ColumnConfig::default();
        assert!(config.is_visible(Column::Time));
        assert!(!config.is_visible(Column::Level)); // hidden by default
        assert!(!config.is_visible(Column::ProcessName)); // hidden by default
        assert!(!config.is_visible(Column::Pid)); // hidden by default
        assert!(!config.is_visible(Column::Tid)); // hidden by default
        assert!(!config.is_visible(Column::Component)); // hidden by default
        assert!(config.is_visible(Column::Log));
        assert!(!config.is_visible(Column::Source)); // hidden by default
    }

    #[test]
    fn test_toggle_column() {
        let mut config = ColumnConfig::default();
        // Find Hostname index — hidden by default
        let idx = config
            .columns
            .iter()
            .position(|(c, _)| *c == Column::Hostname)
            .unwrap();
        assert!(!config.is_visible(Column::Hostname));
        config.toggle(idx);
        assert!(config.is_visible(Column::Hostname));
        config.toggle(idx);
        assert!(!config.is_visible(Column::Hostname));
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
        assert_eq!(default_visible.len(), 2); // Time+Log

        // Show Hostname (hidden by default)
        let idx = config
            .columns
            .iter()
            .position(|(c, _)| *c == Column::Hostname)
            .unwrap();
        config.toggle(idx);
        let visible = config.visible_columns();
        assert_eq!(visible.len(), 3);
        assert!(visible.contains(&Column::Hostname));
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
        assert_eq!(config.visible_columns().len(), 3); // 2 defaults + Source
    }

    // ── Follow mode tests ────────────────────────────────────

    #[test]
    fn test_follow_mode_toggle() {
        let mut app = make_app_cf(100);
        assert!(!app.follow_mode);
        // Enable follow (simulating CLI --follow)
        app.follow_mode = true;
        app.scroll_to_bottom();
        assert!(app.follow_mode);
        assert_eq!(app.selected, 99);
        // F key disables
        app.toggle_follow();
        assert!(!app.follow_mode);
        // F key again is no-op (cannot re-enable)
        app.toggle_follow();
        assert!(!app.follow_mode);
    }

    #[test]
    fn test_follow_mode_exits_on_scroll_up() {
        let mut app = make_app_cf(100);
        app.follow_mode = true;
        app.scroll_to_bottom();
        assert!(app.follow_mode);
        app.select_up(5);
        // Scrolling up does NOT disable follow — just pauses auto-scroll
        assert!(app.follow_mode);
    }

    #[test]
    fn test_follow_mode_exits_on_page_up() {
        let mut app = make_app_cf(100);
        app.follow_mode = true;
        app.scroll_to_bottom();
        assert!(app.follow_mode);
        app.page_up();
        // Page up does NOT disable follow
        assert!(app.follow_mode);
    }

    #[test]
    fn test_follow_mode_persists_on_down() {
        let mut app = make_app_cf(100);
        app.follow_mode = true;
        app.scroll_to_bottom();
        assert!(app.follow_mode);
        // Already at bottom, select_down shouldn't exit follow
        app.select_down(1);
        assert!(app.follow_mode);
    }
}

#[cfg(test)]
mod follow_append_tests {
    use super::*;
    use chrono::Utc;
    use scouty::record::{LogLevel, LogRecord};
    use std::sync::Arc;

    fn make_record(id: u64) -> Arc<LogRecord> {
        Arc::new(LogRecord {
            id,
            timestamp: Utc::now(),
            level: Some(LogLevel::Info),
            message: format!("msg{}", id),
            raw: format!("raw{}", id),
            source: "test".into(),
            loader_id: "test".into(),
            pid: None,
            tid: None,
            component_name: None,
            process_name: None,
            hostname: None,
            container: None,
            context: None,
            function: None,
            metadata: None,
            expanded: None,
        })
    }

    fn make_follow_app(n: usize) -> App {
        let records: Vec<Arc<LogRecord>> = (0..n).map(|i| make_record(i as u64)).collect();
        let total = records.len();
        let filtered: Vec<usize> = (0..total).collect();
        let col_widths = App::compute_col_widths(&records, &filtered);
        App {
            records,
            total_records: total,
            filtered_indices: filtered,
            selected: total.saturating_sub(1),
            scroll_offset: 0,
            visible_rows: 20,
            follow_mode: true,
            follow_new_count: 0,
            col_widths,
            ..App::load_stdin(Vec::new()).unwrap()
        }
    }

    #[test]
    fn test_append_auto_scrolls_at_bottom() {
        let mut app = make_follow_app(5);
        assert_eq!(app.selected, 4);
        app.append_records(vec![make_record(5), make_record(6)]);
        assert_eq!(app.selected, 6); // auto-scrolled to new last
        assert_eq!(app.follow_new_count, 0);
    }

    #[test]
    fn test_append_tracks_new_count_when_scrolled_up() {
        let mut app = make_follow_app(10);
        app.selected = 5; // scrolled up from bottom
        app.append_records(vec![make_record(10), make_record(11)]);
        // Should not auto-scroll
        assert_eq!(app.selected, 5);
        // Should track new records below
        assert!(app.follow_new_count > 0);
    }

    #[test]
    fn test_scroll_to_bottom_resets_new_count() {
        let mut app = make_follow_app(10);
        app.selected = 3;
        app.follow_new_count = 5;
        app.scroll_to_bottom();
        assert_eq!(app.follow_new_count, 0);
    }

    #[test]
    fn test_toggle_clears_new_count() {
        let mut app = make_follow_app(5);
        app.follow_new_count = 10;
        app.toggle_follow(); // disables
        assert!(!app.follow_mode);
        assert_eq!(app.follow_new_count, 0);
    }

    #[test]
    fn test_append_updates_search_matches() {
        let mut app = make_follow_app(5);
        app.follow_mode = true;
        app.scroll_to_bottom();

        // Set up a search regex matching "msg"
        app.search_regex = Some(regex::Regex::new("msg").unwrap());

        // Append new records
        app.append_records(vec![make_record(5), make_record(6)]);

        // Search matches should include the new records
        assert!(!app.search_matches.is_empty());
    }

    #[test]
    fn test_append_filters_new_records() {
        let mut app = make_follow_app(5);
        app.follow_mode = true;
        app.scroll_to_bottom();

        // Add an exclude filter for records containing "msg3"
        let expr = scouty::filter::expr::parse("message contains \"msg3\"").unwrap();
        app.filters.push(crate::app::FilterEntry {
            label: "msg3".to_string(),
            expr_str: "message contains \"msg3\"".to_string(),
            expr,
            exclude: true,
        });

        let before = app.filtered_indices.len();
        // Append a record that would be excluded
        let mut r = (*make_record(10)).clone();
        r.message = "msg3 should be excluded".to_string();
        app.append_records(vec![Arc::new(r)]);

        // Filtered count should not increase (record excluded)
        assert_eq!(app.filtered_indices.len(), before);

        // Append a record that passes
        app.append_records(vec![make_record(11)]);
        assert_eq!(app.filtered_indices.len(), before + 1);
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
            expanded: None,
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
            detail_panel_ratio: 0.3,
            detail_tree_cursor: 0,
            detail_tree_collapsed: std::collections::HashSet::new(),
            detail_horizontal_offset: 0,
            panel_state: crate::panel::PanelState::default(),
            input_mode: InputMode::Normal,
            filter_input: TextInput::new(),
            filter_error: None,
            filters: Vec::new(),
            quick_filter_input: TextInput::new(),
            field_filter: None,
            filter_manager_cursor: 0,
            search_input: TextInput::new(),
            search_matches: vec![],
            search_regex: None,
            search_match_idx: None,
            time_input: TextInput::new(),
            goto_input: TextInput::new(),
            status_message: None,
            shortcut_hints_cache: Vec::new(),
            status_message_at: None,
            col_widths: [19, 5, 11, 3, 3, 9, 7, 8],
            column_config: ColumnConfig::default(),
            follow_mode: false,
            follow_new_count: 0,
            should_quit: false,
            copy_format_cursor: 0,
            save_path_input: TextInput::with_text("./scouty-export.log"),
            save_format_cursor: 0,
            save_dialog_focus: crate::ui::windows::save_dialog_window::Focus::Path,
            help_scroll: 0,
            command_input: TextInput::new(),
            filter_version: 0,
            density_cache: None,
            highlight_rules: Vec::new(),
            highlight_input: TextInput::new(),
            highlight_manager_cursor: 0,
            bookmarks: std::collections::HashSet::new(),
            bookmark_manager_cursor: 0,
            theme: Theme::default(),
            level_filter: None,
            level_filter_cursor: 0,
            preset_name_input: TextInput::new(),
            preset_list: Vec::new(),
            preset_list_cursor: 0,
            density_source: DensitySource::All,
            density_selector_cursor: 0,
            regions: scouty::region::store::RegionStore::default(),
            region_processor: None,
            category_processor: None,
            category_cursor: 0,
            region_manager_cursor: 0,
            region_panel_sort: crate::ui::widgets::region_panel_widget::RegionSortMode::StartTime,
            region_panel_type_filter: None,
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
        assert_eq!(config.visible_columns().len(), 4); // 2 defaults + Hostname + Container
    }

    #[test]
    fn test_hostname_container_in_column_selector() {
        let config = ColumnConfig::default();
        let labels: Vec<&str> = config.columns.iter().map(|(c, _)| c.label()).collect();
        assert!(labels.contains(&"Hostname"));
        assert!(labels.contains(&"Container"));
    }
}

#[cfg(test)]
mod time_jump_tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use scouty::record::{LogLevel, LogRecord};
    use std::sync::Arc;

    fn make_record_with_ts(id: u64, ts: chrono::DateTime<Utc>) -> LogRecord {
        LogRecord {
            id,
            timestamp: ts,
            level: Some(LogLevel::Info),
            source: "test".into(),
            pid: None,
            tid: None,
            component_name: None,
            process_name: None,
            message: format!("msg {}", id),
            hostname: None,
            container: None,
            context: None,
            function: None,
            raw: format!("msg {}", id),
            metadata: None,
            loader_id: "test".into(),
            expanded: None,
        }
    }

    fn make_jump_app(selected: usize, time_input: &str) -> App {
        let base = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let records: Vec<Arc<LogRecord>> = (0..3)
            .map(|i| {
                Arc::new(make_record_with_ts(
                    i as u64,
                    base + chrono::Duration::minutes(i as i64 * 5),
                ))
            })
            .collect();
        App {
            records,
            total_records: 3,
            filtered_indices: vec![0, 1, 2],
            scroll_offset: 0,
            selected,
            visible_rows: 10,
            detail_panel_ratio: 0.3,
            detail_tree_cursor: 0,
            detail_tree_collapsed: std::collections::HashSet::new(),
            detail_horizontal_offset: 0,
            panel_state: crate::panel::PanelState::default(),
            input_mode: InputMode::Normal,
            filter_input: TextInput::new(),
            filter_error: None,
            filters: Vec::new(),
            quick_filter_input: TextInput::new(),
            field_filter: None,
            filter_manager_cursor: 0,
            search_input: TextInput::new(),
            search_matches: vec![],
            search_regex: None,
            search_match_idx: None,
            time_input: TextInput::with_text(time_input),
            goto_input: TextInput::new(),
            status_message: None,
            shortcut_hints_cache: Vec::new(),
            status_message_at: None,
            col_widths: [0; 8],
            column_config: ColumnConfig::default(),
            follow_mode: false,
            follow_new_count: 0,
            should_quit: false,
            copy_format_cursor: 0,
            save_path_input: TextInput::with_text("./scouty-export.log"),
            save_format_cursor: 0,
            save_dialog_focus: crate::ui::windows::save_dialog_window::Focus::Path,
            help_scroll: 0,
            command_input: TextInput::new(),
            filter_version: 0,
            density_cache: None,
            highlight_input: TextInput::new(),
            highlight_manager_cursor: 0,
            highlight_rules: Vec::new(),
            bookmarks: std::collections::HashSet::new(),
            bookmark_manager_cursor: 0,
            theme: Theme::default(),
            level_filter: None,
            level_filter_cursor: 0,
            preset_name_input: TextInput::new(),
            preset_list: Vec::new(),
            preset_list_cursor: 0,
            density_source: DensitySource::All,
            density_selector_cursor: 0,
            regions: scouty::region::store::RegionStore::default(),
            region_processor: None,
            category_processor: None,
            category_cursor: 0,
            region_manager_cursor: 0,
            region_panel_sort: crate::ui::widgets::region_panel_widget::RegionSortMode::StartTime,
            region_panel_type_filter: None,
        }
    }

    #[test]
    fn test_parse_relative_duration() {
        assert_eq!(App::parse_relative_duration("5s"), Some(5_000));
        assert_eq!(App::parse_relative_duration("5m"), Some(300_000));
        assert_eq!(App::parse_relative_duration("2h"), Some(7_200_000));
        assert_eq!(App::parse_relative_duration("1d"), Some(86_400_000));
        assert_eq!(App::parse_relative_duration(""), None);
        assert_eq!(App::parse_relative_duration("abc"), None);
        assert_eq!(App::parse_relative_duration("5x"), None);
        assert_eq!(App::parse_relative_duration("0s"), None);
        // Combined formats
        assert_eq!(App::parse_relative_duration("1h30m"), Some(5_400_000));
        assert_eq!(App::parse_relative_duration("2m30s"), Some(150_000));
        assert_eq!(App::parse_relative_duration("1d2h30m"), Some(95_400_000));
        assert_eq!(App::parse_relative_duration("1h0m"), Some(3_600_000));
        // Millisecond formats
        assert_eq!(App::parse_relative_duration("500ms"), Some(500));
        assert_eq!(App::parse_relative_duration("100ms"), Some(100));
        assert_eq!(App::parse_relative_duration("1s500ms"), Some(1_500));
        assert_eq!(App::parse_relative_duration("0ms"), None);
        // Trailing number without suffix is invalid
        assert_eq!(App::parse_relative_duration("30"), None);
    }

    #[test]
    fn test_format_duration_secs() {
        assert_eq!(App::format_duration_secs(30), "30s");
        assert_eq!(App::format_duration_secs(60), "1m");
        assert_eq!(App::format_duration_secs(300), "5m");
        assert_eq!(App::format_duration_secs(3600), "1h");
        assert_eq!(App::format_duration_secs(86400), "1d");
        assert_eq!(App::format_duration_secs(90), "90s");
        assert_eq!(App::format_duration_secs(-300), "5m");
    }

    #[test]
    fn test_jump_relative_forward() {
        let mut app = make_jump_app(0, "5m");

        app.jump_relative(true);
        assert_eq!(app.selected, 1);

        app.time_input.set("5m");
        app.jump_relative(true);
        assert_eq!(app.selected, 2);
    }

    #[test]
    fn test_jump_relative_backward() {
        let mut app = make_jump_app(2, "5m");

        app.jump_relative(false);
        assert_eq!(app.selected, 1);
    }
}

#[cfg(test)]
mod command_tests {
    use super::*;

    fn make_cmd_record(id: u64, msg: &str) -> LogRecord {
        LogRecord {
            id,
            timestamp: chrono::Utc::now(),
            level: Some(scouty::record::LogLevel::Info),
            source: "test".into(),
            pid: None,
            tid: None,
            component_name: None,
            process_name: None,
            message: msg.to_string(),
            hostname: None,
            container: None,
            context: None,
            function: None,
            raw: msg.to_string(),
            metadata: None,
            loader_id: "test".into(),
            expanded: None,
        }
    }

    fn make_command_app() -> App {
        let records: Vec<Arc<LogRecord>> = (0..2)
            .map(|i| Arc::new(make_cmd_record(i, &format!("line{}", i))))
            .collect();
        let filtered_indices = vec![0, 1];
        App {
            records,
            total_records: 2,
            filtered_indices,
            scroll_offset: 0,
            selected: 0,
            visible_rows: 10,
            detail_panel_ratio: 0.3,
            detail_tree_cursor: 0,
            detail_tree_collapsed: std::collections::HashSet::new(),
            detail_horizontal_offset: 0,
            panel_state: crate::panel::PanelState::default(),
            input_mode: InputMode::Normal,
            filter_input: TextInput::new(),
            filter_error: None,
            filters: Vec::new(),
            quick_filter_input: TextInput::new(),
            field_filter: None,
            filter_manager_cursor: 0,
            search_input: TextInput::new(),
            search_matches: vec![],
            search_regex: None,
            search_match_idx: None,
            time_input: TextInput::new(),
            goto_input: TextInput::new(),
            status_message: None,
            shortcut_hints_cache: Vec::new(),
            status_message_at: None,
            col_widths: [19, 5, 11, 3, 3, 9, 7, 8],
            column_config: ColumnConfig::default(),
            follow_mode: false,
            follow_new_count: 0,
            should_quit: false,
            copy_format_cursor: 0,
            save_path_input: TextInput::with_text("./scouty-export.log"),
            save_format_cursor: 0,
            save_dialog_focus: crate::ui::windows::save_dialog_window::Focus::Path,
            help_scroll: 0,
            command_input: TextInput::new(),
            filter_version: 0,
            density_cache: None,
            highlight_rules: Vec::new(),
            highlight_input: TextInput::new(),
            highlight_manager_cursor: 0,
            bookmarks: std::collections::HashSet::new(),
            bookmark_manager_cursor: 0,
            theme: Theme::default(),
            level_filter: None,
            level_filter_cursor: 0,
            preset_name_input: TextInput::new(),
            preset_list: Vec::new(),
            preset_list_cursor: 0,
            density_source: DensitySource::All,
            density_selector_cursor: 0,
            regions: scouty::region::store::RegionStore::default(),
            region_processor: None,
            category_processor: None,
            category_cursor: 0,
            region_manager_cursor: 0,
            region_panel_sort: crate::ui::widgets::region_panel_widget::RegionSortMode::StartTime,
            region_panel_type_filter: None,
        }
    }
    #[test]
    fn test_command_w_removed() {
        let mut app = make_command_app();
        app.command_input.set("w");
        app.execute_command();
        let msg = app.status_message.as_ref().unwrap();
        assert!(msg.contains("Unknown command"));
    }

    #[test]
    fn test_command_q_sets_should_quit() {
        let mut app = make_command_app();
        app.command_input.set("q");
        app.execute_command();
        assert!(app.should_quit);
    }

    #[test]
    fn test_command_unknown() {
        let mut app = make_command_app();
        app.command_input.set("foobar");
        app.execute_command();
        let msg = app.status_message.as_ref().unwrap();
        assert!(msg.contains("Unknown command"));
    }

    #[test]
    fn test_command_empty() {
        let mut app = make_command_app();
        app.command_input.set("");
        app.execute_command();
        // No status change for empty command
        assert!(!app.should_quit);
    }
}

#[cfg(test)]
mod column_config_tests {
    use super::*;

    #[test]
    fn test_default_width_overrides_matches_columns_len() {
        let cfg = ColumnConfig::default();
        assert_eq!(cfg.width_overrides.len(), cfg.columns.len());
    }

    #[test]
    fn test_effective_width_returns_auto_when_no_override() {
        let cfg = ColumnConfig::default();
        assert_eq!(cfg.effective_width(0, 19), 19);
    }

    #[test]
    fn test_effective_width_returns_override_when_set() {
        let mut cfg = ColumnConfig::default();
        cfg.width_overrides[0] = Some(25);
        assert_eq!(cfg.effective_width(0, 19), 25);
    }

    #[test]
    fn test_effective_width_out_of_bounds_returns_auto() {
        let cfg = ColumnConfig::default();
        assert_eq!(cfg.effective_width(999, 42), 42);
    }

    #[test]
    fn test_adjust_width_increases() {
        let mut cfg = ColumnConfig::default();
        // Column::Time at index 0, auto_width=19
        let changed = cfg.adjust_width(0, 5, 19);
        assert!(changed);
        assert_eq!(cfg.width_overrides[0], Some(24));
    }

    #[test]
    fn test_adjust_width_respects_min() {
        let mut cfg = ColumnConfig::default();
        // Column::Time min_width=19, try to shrink below
        let changed = cfg.adjust_width(0, -100, 19);
        assert!(!changed); // 19 is already min, can't go lower
    }

    #[test]
    fn test_adjust_width_no_overflow_large_values() {
        let mut cfg = ColumnConfig::default();
        cfg.width_overrides[0] = Some(u16::MAX);
        // Should not panic or overflow
        let changed = cfg.adjust_width(0, 100, u16::MAX);
        assert!(!changed); // already at max
        assert_eq!(cfg.width_overrides[0], Some(u16::MAX));
    }

    #[test]
    fn test_adjust_width_skips_log_column() {
        let mut cfg = ColumnConfig::default();
        // Log is last column (index 11)
        let changed = cfg.adjust_width(11, 5, 0);
        assert!(!changed);
    }

    #[test]
    fn test_adjust_width_skips_hidden_column() {
        let mut cfg = ColumnConfig::default();
        // Level at index 1 is hidden by default
        let changed = cfg.adjust_width(1, 5, 10);
        assert!(!changed);
    }

    #[test]
    fn test_reset_width() {
        let mut cfg = ColumnConfig::default();
        cfg.width_overrides[0] = Some(30);
        cfg.reset_width(0);
        assert_eq!(cfg.width_overrides[0], None);
    }

    #[test]
    fn test_reset_width_out_of_bounds_no_panic() {
        let mut cfg = ColumnConfig::default();
        cfg.reset_width(999); // should not panic
    }

    #[test]
    fn test_auto_width_for_col_widths_column() {
        let cfg = ColumnConfig::default();
        let cw = [19, 5, 10, 6, 6, 12, 8, 15];
        // Time is index 0 in columns, col_widths_index() = Some(0) -> cw[0] = 19
        assert_eq!(cfg.auto_width_for(0, &cw), 19);
    }

    #[test]
    fn test_auto_width_for_fixed_column() {
        let cfg = ColumnConfig::default();
        let cw = [19, 5, 10, 6, 6, 12, 8, 15];
        // Hostname is index 2, default_fixed_width() = 20
        assert_eq!(cfg.auto_width_for(2, &cw), 20);
    }

    #[test]
    fn test_auto_width_for_log_returns_zero() {
        let cfg = ColumnConfig::default();
        let cw = [19, 5, 10, 6, 6, 12, 8, 15];
        // Log is index 11
        assert_eq!(cfg.auto_width_for(11, &cw), 0);
    }

    #[test]
    fn test_default_fixed_width_log_is_zero() {
        assert_eq!(Column::Log.default_fixed_width(), 0);
    }

    #[test]
    fn test_default_fixed_width_hostname() {
        assert_eq!(Column::Hostname.default_fixed_width(), 20);
    }

    #[test]
    fn test_display_width() {
        let mut cfg = ColumnConfig::default();
        let cw = [19, 5, 10, 6, 6, 12, 8, 15];
        // No override: auto
        assert_eq!(cfg.display_width(0, &cw), 19);
        // With override
        cfg.width_overrides[0] = Some(30);
        assert_eq!(cfg.display_width(0, &cw), 30);
    }
}
