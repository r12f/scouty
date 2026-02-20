//! Application state for the TUI.

use scouty::filter::eval;
use scouty::filter::expr::{self, Expr};
use scouty::loader::file::FileLoader;
use scouty::parser::factory::ParserFactory;
use scouty::record::LogRecord;
use scouty::session::LogSession;
use scouty::traits::LogLoader;

/// Input mode for the TUI.
#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Filter,
    Search,
    TimeJump,
    GotoLine,
    Help,
}

/// Main application state.
pub struct App {
    /// All log records loaded from the file.
    pub records: Vec<LogRecord>,
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
    /// Filter input buffer.
    pub filter_input: String,
    /// Active filter expression (compiled).
    pub filter_expr: Option<Expr>,
    /// Filter error message.
    pub filter_error: Option<String>,
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
    /// Column widths computed from data (Time, Level, ProcessName, Pid, Tid, Component).
    pub col_widths: [u16; 6],
}

impl App {
    /// Load log records from a file.
    pub fn load_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut loader = FileLoader::new(path, false);
        let _lines = loader.load()?;
        let info = loader.info();

        let group = ParserFactory::create_parser_group(info);

        let mut session = LogSession::new();
        session.add_loader(Box::new(FileLoader::new(path, false)), group);
        let _filtered = session.run()?;

        let records: Vec<LogRecord> = session.store().iter().cloned().collect();
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
            filter_expr: None,
            filter_error: None,
            search_input: String::new(),
            search_matches: vec![],
            search_match_idx: None,
            time_input: String::new(),
            goto_input: String::new(),
            status_message: None,
            col_widths,
        })
    }

    /// Compute auto-fit column widths by sampling records.
    /// Returns [Time, Level, ProcessName, Pid, Tid, Component] widths.
    /// The Log column will fill remaining space.
    fn compute_col_widths(records: &[LogRecord], indices: &[usize]) -> [u16; 6] {
        // Minimum widths from headers
        let mut widths: [u16; 6] = [
            4,  // "Time"
            5,  // "Level"
            11, // "ProcessName"
            3,  // "Pid"
            3,  // "Tid"
            9,  // "Component"
        ];

        // Maximum widths to prevent any single column from being too wide
        let max_widths: [u16; 6] = [23, 5, 20, 8, 8, 20];

        // Sample up to 1000 records evenly distributed
        let sample_size = 1000.min(indices.len());
        let step = if sample_size == 0 {
            1
        } else {
            indices.len().max(1) / sample_size.max(1)
        }
        .max(1);

        for i in (0..indices.len()).step_by(step) {
            let r = &records[indices[i]];

            // Time: fixed format "YYYY-MM-DD HH:MM:SS"
            widths[0] = widths[0].max(19);

            // Level
            if let Some(level) = r.level {
                let len = format!("{}", level).len() as u16;
                widths[1] = widths[1].max(len);
            }

            // ProcessName
            if let Some(ref name) = r.process_name {
                widths[2] = widths[2].max((name.len() as u16).min(max_widths[2]));
            }

            // Pid
            if let Some(pid) = r.pid {
                let len = format!("{}", pid).len() as u16;
                widths[3] = widths[3].max(len.min(max_widths[3]));
            }

            // Tid
            if let Some(tid) = r.tid {
                let len = format!("{}", tid).len() as u16;
                widths[4] = widths[4].max(len.min(max_widths[4]));
            }

            // Component
            if let Some(ref comp) = r.component_name {
                widths[5] = widths[5].max((comp.len() as u16).min(max_widths[5]));
            }
        }

        // Clamp all to max
        for i in 0..6 {
            widths[i] = widths[i].min(max_widths[i]);
        }

        widths
    }

    /// Total filtered record count.
    pub fn total(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Apply filter expression to records.
    pub fn apply_filter(&mut self) {
        if self.filter_input.is_empty() {
            self.filter_expr = None;
            self.filter_error = None;
            self.filtered_indices = (0..self.records.len()).collect();
        } else {
            match expr::parse(&self.filter_input) {
                Ok(expr) => {
                    self.filtered_indices = self
                        .records
                        .iter()
                        .enumerate()
                        .filter(|(_, r)| eval::eval(&expr, r))
                        .map(|(i, _)| i)
                        .collect();
                    self.filter_expr = Some(expr);
                    self.filter_error = None;
                }
                Err(e) => {
                    self.filter_error = Some(e);
                    return;
                }
            }
        }
        self.col_widths = Self::compute_col_widths(&self.records, &self.filtered_indices);
        self.scroll_offset = 0;
        self.selected = 0;
        self.clear_search();
    }

    /// Execute regex search across filtered records.
    pub fn execute_search(&mut self) {
        if self.search_input.is_empty() {
            self.clear_search();
            return;
        }
        // Build case-insensitive regex
        let pattern = match regex::RegexBuilder::new(&self.search_input)
            .case_insensitive(true)
            .build()
        {
            Ok(re) => re,
            Err(e) => {
                self.search_matches.clear();
                self.search_match_idx = None;
                self.status_message = Some(format!("Invalid regex: {}", e));
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
            self.status_message = Some("No matches found".to_string());
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

    /// Jump to next search match.
    pub fn next_search_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        if let Some(idx) = self.search_match_idx {
            let next = (idx + 1) % self.search_matches.len();
            self.search_match_idx = Some(next);
            self.jump_to_search_match();
        }
    }

    /// Jump to previous search match.
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
            let total = self.search_matches.len();
            self.status_message = Some(format!("Match {}/{}", idx + 1, total));
        }
    }

    fn clear_search(&mut self) {
        self.search_matches.clear();
        self.search_match_idx = None;
    }

    /// Jump to time (format: HH:MM:SS or YYYY-MM-DD HH:MM:SS).
    pub fn jump_to_time(&mut self) {
        use chrono::NaiveTime;

        let input = self.time_input.trim();
        if input.is_empty() {
            return;
        }

        if let Ok(time) = NaiveTime::parse_from_str(input, "%H:%M:%S") {
            for (fi, &ri) in self.filtered_indices.iter().enumerate() {
                let record_time = self.records[ri].timestamp.time();
                if record_time >= time {
                    self.selected = fi;
                    self.ensure_selected_visible();
                    self.status_message = Some(format!("Jumped to {}", time));
                    return;
                }
            }
            self.status_message = Some("No record at or after that time".to_string());
            return;
        }

        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S") {
            let dt_utc = dt.and_utc();
            for (fi, &ri) in self.filtered_indices.iter().enumerate() {
                if self.records[ri].timestamp >= dt_utc {
                    self.selected = fi;
                    self.ensure_selected_visible();
                    self.status_message = Some(format!("Jumped to {}", dt_utc));
                    return;
                }
            }
            self.status_message = Some("No record at or after that time".to_string());
            return;
        }

        self.status_message =
            Some("Invalid time format (use HH:MM:SS or YYYY-MM-DD HH:MM:SS)".to_string());
    }

    /// Jump to a specific line number (1-indexed).
    pub fn goto_line(&mut self) {
        let input = self.goto_input.trim();
        if input.is_empty() {
            return;
        }
        match input.parse::<usize>() {
            Ok(line) if line >= 1 && line <= self.total() => {
                self.selected = line - 1;
                self.ensure_selected_visible();
                self.status_message = Some(format!("Line {}", line));
            }
            Ok(line) if line > self.total() => {
                self.scroll_to_bottom();
                self.status_message = Some(format!("Line {} (clamped to {})", line, self.total()));
            }
            _ => {
                self.status_message = Some("Invalid line number".to_string());
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
    }

    pub fn page_down(&mut self) {
        let half = self.visible_rows / 2;
        self.select_down(half.max(1));
    }

    pub fn page_up(&mut self) {
        let half = self.visible_rows / 2;
        self.select_up(half.max(1));
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

    fn ensure_selected_visible(&mut self) {
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + self.visible_rows {
            self.scroll_offset = self.selected.saturating_sub(self.visible_rows - 1);
        }
    }

    /// Get the visible slice of filtered indices.
    pub fn visible_records(&self) -> Vec<&LogRecord> {
        let end = (self.scroll_offset + self.visible_rows).min(self.total());
        self.filtered_indices[self.scroll_offset..end]
            .iter()
            .map(|&i| &self.records[i])
            .collect()
    }

    /// Get the selected record.
    pub fn selected_record(&self) -> Option<&LogRecord> {
        self.filtered_indices
            .get(self.selected)
            .map(|&i| &self.records[i])
    }

    /// Check if a filtered index is a search match.
    pub fn is_search_match(&self, filtered_idx: usize) -> bool {
        self.search_matches.contains(&filtered_idx)
    }
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
            raw: message.to_string(),
            metadata: None,
            loader_id: "test".into(),
        }
    }

    fn make_app(n: usize) -> App {
        let records: Vec<LogRecord> = (0..n)
            .map(|i| make_record(i as u64, Some(LogLevel::Info), &format!("msg {}", i)))
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
            filter_expr: None,
            filter_error: None,
            search_input: String::new(),
            search_matches: vec![],
            search_match_idx: None,
            time_input: String::new(),
            goto_input: String::new(),
            status_message: None,
            col_widths: [19, 5, 11, 3, 3, 9],
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
        app.page_down(); // half screen = 5
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
        app.records[5].message = "error happened".to_string();
        app.records[15].message = "another error".to_string();
        app.search_input = "error".to_string();
        app.execute_search();
        assert_eq!(app.search_matches.len(), 2);
        assert_eq!(app.search_matches, vec![5, 15]);
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
        app.records[3].message = "ERROR: connection timeout".to_string();
        app.records[7].message = "error: disk full".to_string();
        app.records[12].message = "Warning: low memory".to_string();

        // Regex search: case-insensitive by default
        app.search_input = "error".to_string();
        app.execute_search();
        assert_eq!(app.search_matches.len(), 2);
        assert_eq!(app.search_matches, vec![3, 7]);

        // Regex pattern
        app.search_input = r"error.*(?:timeout|full)".to_string();
        app.execute_search();
        assert_eq!(app.search_matches.len(), 2);

        // Invalid regex
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
    fn test_input_modes() {
        let app = make_app(10);
        assert_eq!(app.input_mode, InputMode::Normal);
    }

    #[test]
    fn test_goto_line() {
        let mut app = make_app(100);
        app.goto_input = "50".to_string();
        app.goto_line();
        assert_eq!(app.selected, 49); // 1-indexed

        app.goto_input = "999".to_string();
        app.goto_line();
        assert_eq!(app.selected, 99); // clamped

        app.goto_input = "abc".to_string();
        app.goto_line();
        assert!(app.status_message.as_ref().unwrap().contains("Invalid"));
    }

    #[test]
    fn test_col_widths() {
        let app = make_app(10);
        // Time should be at least 19 (YYYY-MM-DD HH:MM:SS)
        assert!(app.col_widths[0] >= 19);
        // Level should be at least 5
        assert!(app.col_widths[1] >= 5);
    }
}
