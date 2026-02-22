//! Stdin-based log loader — reads log lines from piped stdin.

#[cfg(test)]
#[path = "stdin_tests.rs"]
mod stdin_tests;

use crate::traits::{LoaderInfo, LoaderType, LogLoader, Result};
use std::io::BufRead;

/// Loads log lines from stdin (when piped).
#[derive(Debug)]
pub struct StdinLoader {
    info: LoaderInfo,
}

impl StdinLoader {
    pub fn new() -> Self {
        Self {
            info: LoaderInfo {
                id: "<stdin>".to_string(),
                loader_type: LoaderType::TextFile,
                multiline_enabled: false,
                sample_lines: Vec::new(),
                file_mod_year: None,
            },
        }
    }
}

impl Default for StdinLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl LogLoader for StdinLoader {
    fn info(&self) -> &LoaderInfo {
        &self.info
    }

    /// Read all lines from stdin until EOF and return them.
    ///
    /// **Known limitation:** This blocks until the producer closes the pipe,
    /// so streaming sources (`journalctl -f`, `tail -f`, …) will never
    /// return.  A streaming / incremental loader API is planned as a future
    /// enhancement.
    fn load(&mut self) -> Result<Vec<String>> {
        let stdin = std::io::stdin();
        let reader = stdin.lock();
        let lines: Vec<String> = reader.lines().collect::<std::result::Result<Vec<_>, _>>()?;
        self.info.sample_lines = lines.iter().take(10).cloned().collect();
        Ok(lines)
    }
}
