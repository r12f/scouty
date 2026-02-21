//! File-based log loader — loads log lines from a local text file.

#[cfg(test)]
#[path = "file_tests.rs"]
mod file_tests;

use crate::traits::{LoaderInfo, LoaderType, LogLoader, Result};
use chrono::Datelike;
use std::path::PathBuf;

/// Loads log lines from a local text file.
#[derive(Debug)]
pub struct FileLoader {
    path: PathBuf,
    info: LoaderInfo,
}

impl FileLoader {
    pub fn new(path: impl Into<PathBuf>, multiline: bool) -> Self {
        let path = path.into();
        let id = path.display().to_string();
        Self {
            info: LoaderInfo {
                id,
                loader_type: LoaderType::TextFile,
                multiline_enabled: multiline,
                sample_lines: Vec::new(),
                file_mod_year: None,
            },
            path,
        }
    }
}

impl LogLoader for FileLoader {
    fn info(&self) -> &LoaderInfo {
        &self.info
    }

    fn load(&mut self) -> Result<Vec<String>> {
        // Extract file modification year early for BSD syslog timestamp inference
        if let Ok(metadata) = std::fs::metadata(&self.path) {
            if let Ok(modified) = metadata.modified() {
                let dt: chrono::DateTime<chrono::Utc> = modified.into();
                self.info.file_mod_year = Some(dt.year());
            }
        }

        let is_gzip = self
            .path
            .extension()
            .and_then(|e| e.to_str())
            .map(|ext| {
                let ext = ext.to_ascii_lowercase();
                ext == "gz" || ext == "gzip"
            })
            .unwrap_or(false);

        let content = if is_gzip {
            use flate2::read::GzDecoder;
            use std::io::Read;
            let file = std::fs::File::open(&self.path)?;
            let mut decoder = GzDecoder::new(file);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed).map_err(|e| {
                std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to decompress gzip file '{}': {}",
                        self.path.display(),
                        e
                    ),
                )
            })?;
            String::from_utf8(decompressed).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Decompressed gzip file '{}' contains invalid UTF-8: {}",
                        self.path.display(),
                        e
                    ),
                )
            })?
        } else {
            // Memory-map the file for zero-copy reading
            let file = std::fs::File::open(&self.path)?;
            let mmap = unsafe { memmap2::Mmap::map(&file)? };

            // Validate UTF-8 and split into lines
            let text = std::str::from_utf8(&mmap).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "File '{}' contains invalid UTF-8: {}",
                        self.path.display(),
                        e
                    ),
                )
            })?;
            let mut lines: Vec<String> = Vec::with_capacity(text.len() / 100); // estimate
            for line in text.lines() {
                lines.push(line.to_string());
            }

            // Store sample lines for parser auto-detection
            self.info.sample_lines = lines.iter().take(10).cloned().collect();

            return Ok(lines);
        };
        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

        // Store sample lines for parser auto-detection
        self.info.sample_lines = lines.iter().take(10).cloned().collect();

        Ok(lines)
    }
}
