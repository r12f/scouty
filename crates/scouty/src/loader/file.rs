//! File-based log loader — loads log lines from a local text file.

#[cfg(test)]
#[path = "file_tests.rs"]
mod file_tests;

use crate::traits::{LoaderInfo, LoaderType, LogLoader, Result};
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
        let content = if self.path.extension().and_then(|e| e.to_str()) == Some("gz") {
            // Decompress gzip file
            use flate2::read::GzDecoder;
            use std::io::Read;
            let file = std::fs::File::open(&self.path)?;
            let mut decoder = GzDecoder::new(file);
            let mut s = String::new();
            decoder.read_to_string(&mut s).map_err(|e| {
                std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to decompress gzip file '{}': {}",
                        self.path.display(),
                        e
                    ),
                )
            })?;
            s
        } else {
            std::fs::read_to_string(&self.path)?
        };
        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

        // Store sample lines for parser auto-detection
        self.info.sample_lines = lines.iter().take(10).cloned().collect();

        Ok(lines)
    }
}
