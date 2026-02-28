//! Follow mode — watches a file for new data and yields new lines incrementally.

#[cfg(test)]
#[path = "follow_tests.rs"]
mod follow_tests;

use scouty::parser::factory::ParserFactory;
use scouty::record::LogRecord;
use scouty::traits::LoaderInfo;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Get the current size of a file in bytes.
pub fn file_size(path: &Path) -> std::io::Result<u64> {
    Ok(std::fs::metadata(path)?.len())
}

/// File watcher that tracks read position and yields new parsed records on poll.
pub struct FileFollower {
    path: PathBuf,
    offset: u64,
    /// Partial line buffer (when file doesn't end with newline yet).
    partial: String,
    /// Loader info for parser creation.
    info: LoaderInfo,
    /// Next record ID to assign.
    next_record_id: u64,
}

impl FileFollower {
    /// Create a new follower starting from the given byte offset.
    /// `start_record_id` is the next record ID to assign to new records.
    pub fn new(
        path: impl Into<PathBuf>,
        start_offset: u64,
        info: LoaderInfo,
        start_record_id: u64,
    ) -> Self {
        Self {
            path: path.into(),
            offset: start_offset,
            partial: String::new(),
            info,
            next_record_id: start_record_id,
        }
    }

    /// Poll for new complete lines, parse them into LogRecords.
    /// Returns empty vec if no new data.
    pub fn poll(&mut self) -> std::io::Result<Vec<Arc<LogRecord>>> {
        let lines = self.poll_lines()?;
        if lines.is_empty() {
            return Ok(Vec::new());
        }

        let group = ParserFactory::create_parser_group(&self.info);
        let mut records = Vec::new();

        for line in lines {
            if let Some(mut record) =
                group.parse(&line, &self.info.id, &self.info.id, self.next_record_id)
            {
                record.raw = line;
                records.push(Arc::new(record));
                self.next_record_id += 1;
            }
        }

        Ok(records)
    }

    /// Poll for new raw lines (without parsing).
    fn poll_lines(&mut self) -> std::io::Result<Vec<String>> {
        let metadata = std::fs::metadata(&self.path)?;
        let current_size = metadata.len();

        // File truncated — reset to beginning
        if current_size < self.offset {
            tracing::info!(
                path = %self.path.display(),
                old_offset = self.offset,
                new_size = current_size,
                "File truncated, resetting"
            );
            self.offset = 0;
            self.partial.clear();
        }

        // No new data
        if current_size == self.offset {
            return Ok(Vec::new());
        }

        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(self.offset))?;
        let mut reader = BufReader::new(file);

        let mut lines = Vec::new();
        let mut buf = std::mem::take(&mut self.partial);

        loop {
            let mut line_buf = String::new();
            let bytes_read = reader.read_line(&mut line_buf)?;
            if bytes_read == 0 {
                break;
            }
            self.offset += bytes_read as u64;
            buf.push_str(&line_buf);

            if buf.ends_with('\n') {
                let trimmed = buf.trim_end_matches('\n').trim_end_matches('\r');
                if !trimmed.is_empty() {
                    lines.push(trimmed.to_string());
                }
                buf.clear();
            }
        }

        // Leftover partial line
        if !buf.is_empty() {
            self.partial = buf;
        }

        Ok(lines)
    }

    /// The path being followed.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Current byte offset.
    pub fn offset(&self) -> u64 {
        self.offset
    }
}
