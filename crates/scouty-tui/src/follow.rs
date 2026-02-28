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

/// Get the inode number of a file (Unix only, returns 0 on other platforms).
#[cfg(unix)]
fn file_inode(path: &Path) -> std::io::Result<u64> {
    use std::os::unix::fs::MetadataExt;
    Ok(std::fs::metadata(path)?.ino())
}

#[cfg(not(unix))]
fn file_inode(_path: &Path) -> std::io::Result<u64> {
    Ok(0)
}

/// Result of a single poll cycle.
pub enum PollResult {
    /// No new data.
    NoChange,
    /// New records parsed from appended bytes.
    NewRecords(Vec<Arc<LogRecord>>),
    /// File was truncated — caller should clear and reload.
    Truncated,
    /// File was rotated (inode changed) — caller should clear and reload.
    Rotated,
    /// File was deleted — caller should show warning.
    Deleted,
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
    /// Inode at last successful read (for rotation detection).
    inode: u64,
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
        let path = path.into();
        let inode = file_inode(&path).unwrap_or(0);
        Self {
            path,
            offset: start_offset,
            partial: String::new(),
            info,
            next_record_id: start_record_id,
            inode,
        }
    }

    /// Poll for new data, handling truncation, rotation, and deletion.
    pub fn poll(&mut self) -> PollResult {
        // Check for inode change (rotation) — must happen before metadata check
        // because a rotated file may be smaller, which would look like truncation.
        let current_inode = file_inode(&self.path).unwrap_or(0);
        if current_inode != 0 && self.inode != 0 && current_inode != self.inode {
            tracing::info!(
                path = %self.path.display(),
                old_inode = self.inode,
                new_inode = current_inode,
                "File rotated (inode changed)"
            );
            self.inode = current_inode;
            self.offset = 0;
            self.partial.clear();
            return PollResult::Rotated;
        }

        let metadata = match std::fs::metadata(&self.path) {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::warn!(path = %self.path.display(), "File deleted");
                return PollResult::Deleted;
            }
            Err(e) => {
                // Transient error (permission, I/O) — don't stop following
                tracing::warn!(%e, path = %self.path.display(), "Transient error reading file metadata");
                return PollResult::NoChange;
            }
        };
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
            return PollResult::Truncated;
        }

        // No new data
        if current_size == self.offset {
            return PollResult::NoChange;
        }

        // Read new bytes
        let lines = match self.read_new_lines() {
            Ok(lines) => lines,
            Err(e) => {
                tracing::warn!(%e, "Error reading new lines");
                return PollResult::NoChange;
            }
        };

        if lines.is_empty() {
            return PollResult::NoChange;
        }

        // Parse new lines
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

        if records.is_empty() {
            PollResult::NoChange
        } else {
            PollResult::NewRecords(records)
        }
    }

    /// Read new complete lines from the file.
    fn read_new_lines(&mut self) -> std::io::Result<Vec<String>> {
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

    /// Reset offset to 0 (for reload after truncation/rotation).
    /// Preserves `next_record_id` so new records don't collide with old IDs.
    pub fn reset(&mut self) {
        self.offset = 0;
        self.partial.clear();
        self.inode = file_inode(&self.path).unwrap_or(0);
    }

    /// Reset offset and set a new starting record ID (for full reload after clear).
    pub fn reset_with_id(&mut self, start_record_id: u64) {
        self.offset = 0;
        self.partial.clear();
        self.next_record_id = start_record_id;
        self.inode = file_inode(&self.path).unwrap_or(0);
    }
}
