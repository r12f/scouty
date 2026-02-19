//! File-based log loader (placeholder for Phase 2).

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
        let content = std::fs::read_to_string(&self.path)?;
        let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

        // Store sample lines for parser auto-detection
        self.info.sample_lines = lines.iter().take(10).cloned().collect();

        Ok(lines)
    }
}
