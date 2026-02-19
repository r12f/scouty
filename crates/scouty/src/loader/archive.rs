//! Archive-based log loader — supports gz, zip, and 7z compressed files.

#[cfg(test)]
#[path = "archive_tests.rs"]
mod archive_tests;

use crate::traits::{LoaderInfo, LoaderType, LogLoader, Result, ScoutyError};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

/// Supported archive formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    Gzip,
    Zip,
    SevenZ,
}

/// Loads log lines from compressed archive files (gz, zip, 7z).
#[derive(Debug)]
pub struct ArchiveLoader {
    path: PathBuf,
    format: ArchiveFormat,
    info: LoaderInfo,
}

impl ArchiveLoader {
    /// Create a new ArchiveLoader, auto-detecting format from file extension.
    pub fn new(path: impl Into<PathBuf>, multiline: bool) -> Result<Self> {
        let path = path.into();
        let format = Self::detect_format(&path)?;
        let id = path.display().to_string();
        Ok(Self {
            info: LoaderInfo {
                id,
                loader_type: LoaderType::Archive,
                multiline_enabled: multiline,
                sample_lines: Vec::new(),
            },
            path,
            format,
        })
    }

    /// Create with an explicit format.
    pub fn with_format(path: impl Into<PathBuf>, format: ArchiveFormat, multiline: bool) -> Self {
        let path = path.into();
        let id = path.display().to_string();
        Self {
            info: LoaderInfo {
                id,
                loader_type: LoaderType::Archive,
                multiline_enabled: multiline,
                sample_lines: Vec::new(),
            },
            path,
            format,
        }
    }

    fn detect_format(path: &std::path::Path) -> Result<ArchiveFormat> {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        if name.ends_with(".gz") || name.ends_with(".gzip") {
            Ok(ArchiveFormat::Gzip)
        } else if name.ends_with(".zip") {
            Ok(ArchiveFormat::Zip)
        } else if name.ends_with(".7z") {
            Ok(ArchiveFormat::SevenZ)
        } else {
            Err(ScoutyError::Config(format!(
                "Cannot detect archive format for: {}",
                path.display()
            )))
        }
    }

    fn load_gzip(&self) -> Result<Vec<String>> {
        let file = std::fs::File::open(&self.path)?;
        let decoder = flate2::read::GzDecoder::new(file);
        let reader = BufReader::new(decoder);
        let lines: Vec<String> = reader.lines().collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(lines)
    }

    fn load_zip(&self) -> Result<Vec<String>> {
        let file = std::fs::File::open(&self.path)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| ScoutyError::Other(format!("Failed to open zip: {}", e)))?;

        let mut all_lines = Vec::new();
        for i in 0..archive.len() {
            let entry = archive
                .by_index(i)
                .map_err(|e| ScoutyError::Other(format!("Failed to read zip entry: {}", e)))?;
            if entry.is_dir() {
                continue;
            }
            let reader = BufReader::new(entry);
            for line in reader.lines() {
                all_lines.push(line?);
            }
        }
        Ok(all_lines)
    }

    fn load_7z(&self) -> Result<Vec<String>> {
        let mut all_lines = Vec::new();
        sevenz_rust::decompress_file(&self.path, std::env::temp_dir().join("scouty_7z_tmp"))
            .map_err(|e| ScoutyError::Other(format!("Failed to decompress 7z: {}", e)))?;

        // Read extracted files from temp dir
        let tmp_dir = std::env::temp_dir().join("scouty_7z_tmp");
        Self::read_dir_recursive(&tmp_dir, &mut all_lines)?;

        // Clean up
        let _ = std::fs::remove_dir_all(&tmp_dir);
        Ok(all_lines)
    }

    fn read_dir_recursive(dir: &std::path::Path, lines: &mut Vec<String>) -> Result<()> {
        if dir.is_file() {
            let file = std::fs::File::open(dir)?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                lines.push(line?);
            }
            return Ok(());
        }
        if dir.is_dir() {
            let mut entries: Vec<_> = std::fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();
            entries.sort_by_key(|e| e.path());
            for entry in entries {
                Self::read_dir_recursive(&entry.path(), lines)?;
            }
        }
        Ok(())
    }
}

impl LogLoader for ArchiveLoader {
    fn info(&self) -> &LoaderInfo {
        &self.info
    }

    fn load(&mut self) -> Result<Vec<String>> {
        let lines = match self.format {
            ArchiveFormat::Gzip => self.load_gzip()?,
            ArchiveFormat::Zip => self.load_zip()?,
            ArchiveFormat::SevenZ => self.load_7z()?,
        };

        self.info.sample_lines = lines.iter().take(10).cloned().collect();
        Ok(lines)
    }
}
