//! Application state for the TUI.

use scouty::loader::file::FileLoader;
use scouty::parser::factory::ParserFactory;
use scouty::parser::group::ParserGroup;
use scouty::record::LogRecord;
use scouty::session::LogSession;
use scouty::traits::LogLoader;

/// Main application state.
pub struct App {
    /// All log records loaded from the file.
    pub records: Vec<LogRecord>,
    /// Current scroll offset (index of top visible record).
    pub scroll_offset: usize,
    /// Number of visible rows in the log list (updated by render).
    pub visible_rows: usize,
    /// Total record count.
    pub total: usize,
}

impl App {
    /// Load log records from a file.
    pub fn load_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut loader = FileLoader::new(path);
        let _lines = loader.load()?;
        let info = loader.info();

        let group = ParserFactory::create_parser_group(info);

        let mut session = LogSession::new();
        session.add_loader(Box::new(FileLoader::new(path)), group);
        let _filtered = session.run()?;

        let records: Vec<LogRecord> = session.store().iter().cloned().collect();
        let total = records.len();

        Ok(Self {
            records,
            scroll_offset: 0,
            visible_rows: 20,
            total,
        })
    }

    pub fn scroll_down(&mut self, n: usize) {
        let max = self.total.saturating_sub(self.visible_rows);
        self.scroll_offset = (self.scroll_offset + n).min(max);
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }

    pub fn page_down(&mut self) {
        self.scroll_down(self.visible_rows);
    }

    pub fn page_up(&mut self) {
        self.scroll_up(self.visible_rows);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.total.saturating_sub(self.visible_rows);
    }

    /// Get the visible slice of records.
    pub fn visible_records(&self) -> &[LogRecord] {
        let end = (self.scroll_offset + self.visible_rows).min(self.total);
        &self.records[self.scroll_offset..end]
    }
}
