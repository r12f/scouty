//! Application state for the TUI.

use scouty::loader::file::FileLoader;
use scouty::parser::factory::ParserFactory;
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
        let mut loader = FileLoader::new(path, false);
        let _lines = loader.load()?;
        let info = loader.info();

        let group = ParserFactory::create_parser_group(info);

        let mut session = LogSession::new();
        session.add_loader(Box::new(FileLoader::new(path, false)), group);
        let _filtered = session.run()?;

        let records: Vec<LogRecord> = session.store().records().to_vec();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_down() {
        let mut app = App {
            records: vec![],
            scroll_offset: 0,
            visible_rows: 10,
            total: 100,
        };
        app.scroll_down(5);
        assert_eq!(app.scroll_offset, 5);
    }

    #[test]
    fn test_scroll_down_clamped() {
        let mut app = App {
            records: vec![],
            scroll_offset: 85,
            visible_rows: 10,
            total: 100,
        };
        app.scroll_down(20);
        assert_eq!(app.scroll_offset, 90);
    }

    #[test]
    fn test_scroll_up() {
        let mut app = App {
            records: vec![],
            scroll_offset: 10,
            visible_rows: 10,
            total: 100,
        };
        app.scroll_up(5);
        assert_eq!(app.scroll_offset, 5);
    }

    #[test]
    fn test_scroll_up_clamped() {
        let mut app = App {
            records: vec![],
            scroll_offset: 3,
            visible_rows: 10,
            total: 100,
        };
        app.scroll_up(10);
        assert_eq!(app.scroll_offset, 0);
    }

    #[test]
    fn test_page_down() {
        let mut app = App {
            records: vec![],
            scroll_offset: 0,
            visible_rows: 20,
            total: 100,
        };
        app.page_down();
        assert_eq!(app.scroll_offset, 20);
    }

    #[test]
    fn test_scroll_to_top_and_bottom() {
        let mut app = App {
            records: vec![],
            scroll_offset: 50,
            visible_rows: 10,
            total: 100,
        };
        app.scroll_to_top();
        assert_eq!(app.scroll_offset, 0);
        app.scroll_to_bottom();
        assert_eq!(app.scroll_offset, 90);
    }

    #[test]
    fn test_empty_records() {
        let mut app = App {
            records: vec![],
            scroll_offset: 0,
            visible_rows: 10,
            total: 0,
        };
        app.scroll_down(1);
        assert_eq!(app.scroll_offset, 0);
        app.scroll_to_bottom();
        assert_eq!(app.scroll_offset, 0);
    }
}
