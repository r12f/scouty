//! Tests for FileFollower.

#[cfg(test)]
mod tests {
    use crate::follow::{file_size, FileFollower};
    use scouty::traits::{LoaderInfo, LoaderType};
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn test_info(path: &str) -> LoaderInfo {
        LoaderInfo {
            id: path.to_string(),
            loader_type: LoaderType::TextFile,
            multiline_enabled: false,
            sample_lines: Vec::new(),
            file_mod_year: None,
        }
    }

    #[test]
    fn test_poll_new_lines() {
        let mut tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();
        let info = test_info(&path.display().to_string());

        let mut follower = FileFollower::new(&path, 0, info, 0);
        writeln!(tmp, "2024-01-01 line1").unwrap();
        writeln!(tmp, "2024-01-01 line2").unwrap();
        tmp.flush().unwrap();

        let records = follower.poll().unwrap();
        assert_eq!(records.len(), 2);

        // No new data
        let records = follower.poll().unwrap();
        assert!(records.is_empty());

        // Append
        writeln!(tmp, "2024-01-01 line3").unwrap();
        tmp.flush().unwrap();

        let records = follower.poll().unwrap();
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn test_follow_from_offset() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "existing").unwrap();
        tmp.flush().unwrap();

        let path = tmp.path().to_path_buf();
        let size = file_size(&path).unwrap();
        let info = test_info(&path.display().to_string());

        let mut follower = FileFollower::new(&path, size, info, 0);

        // Should not return existing content
        let records = follower.poll().unwrap();
        assert!(records.is_empty());

        // Append new line
        writeln!(tmp, "new_line").unwrap();
        tmp.flush().unwrap();

        let records = follower.poll().unwrap();
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn test_follow_truncation() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        {
            let mut f = std::fs::File::create(&path).unwrap();
            writeln!(f, "original_line1").unwrap();
            writeln!(f, "original_line2").unwrap();
        }

        let info = test_info(&path.display().to_string());
        let mut follower = FileFollower::new(&path, 0, info, 0);
        let records = follower.poll().unwrap();
        assert_eq!(records.len(), 2);

        // Truncate and write new content
        {
            let mut f = std::fs::File::create(&path).unwrap();
            writeln!(f, "new").unwrap();
        }

        let records = follower.poll().unwrap();
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn test_follow_partial_line() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        {
            let mut f = std::fs::File::create(&path).unwrap();
            write!(f, "partial").unwrap();
        }

        let info = test_info(&path.display().to_string());
        let mut follower = FileFollower::new(&path, 0, info, 0);
        let records = follower.poll().unwrap();
        assert!(records.is_empty());

        // Complete the line
        {
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .open(&path)
                .unwrap();
            writeln!(f, "_complete").unwrap();
        }

        let records = follower.poll().unwrap();
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn test_follow_empty_file() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();
        let info = test_info(&path.display().to_string());
        let mut follower = FileFollower::new(&path, 0, info, 0);
        let records = follower.poll().unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn test_file_size() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "hello").unwrap();
        tmp.flush().unwrap();
        let size = file_size(tmp.path()).unwrap();
        assert!(size > 0);
    }

    #[test]
    fn test_record_ids_increment() {
        let mut tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();
        let info = test_info(&path.display().to_string());

        let mut follower = FileFollower::new(&path, 0, info, 100);
        writeln!(tmp, "2024-01-01 line1").unwrap();
        writeln!(tmp, "2024-01-01 line2").unwrap();
        tmp.flush().unwrap();

        let records = follower.poll().unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, 100);
        assert_eq!(records[1].id, 101);
    }
}
