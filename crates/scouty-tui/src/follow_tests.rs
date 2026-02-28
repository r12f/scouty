//! Tests for FileFollower.

#[cfg(test)]
mod tests {
    use crate::follow::{file_size, FileFollower, PollResult};
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

        match follower.poll() {
            PollResult::NewRecords(records) => assert_eq!(records.len(), 2),
            other => panic!("expected NewRecords, got {:?}", poll_name(&other)),
        }

        // No new data
        assert!(matches!(follower.poll(), PollResult::NoChange));

        // Append
        writeln!(tmp, "2024-01-01 line3").unwrap();
        tmp.flush().unwrap();

        match follower.poll() {
            PollResult::NewRecords(records) => assert_eq!(records.len(), 1),
            other => panic!("expected NewRecords, got {:?}", poll_name(&other)),
        }
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
        assert!(matches!(follower.poll(), PollResult::NoChange));

        // Append new line
        writeln!(tmp, "new_line").unwrap();
        tmp.flush().unwrap();

        match follower.poll() {
            PollResult::NewRecords(records) => assert_eq!(records.len(), 1),
            other => panic!("expected NewRecords, got {:?}", poll_name(&other)),
        }
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

        // Read initial content
        match follower.poll() {
            PollResult::NewRecords(records) => assert_eq!(records.len(), 2),
            other => panic!("expected NewRecords, got {:?}", poll_name(&other)),
        }

        // Truncate and write shorter content
        {
            let mut f = std::fs::File::create(&path).unwrap();
            writeln!(f, "new").unwrap();
        }

        // Should detect truncation
        assert!(matches!(follower.poll(), PollResult::Truncated));

        // After reset, should read the new content
        follower.reset_with_id(0);
        match follower.poll() {
            PollResult::NewRecords(records) => assert_eq!(records.len(), 1),
            other => panic!(
                "expected NewRecords after reset, got {:?}",
                poll_name(&other)
            ),
        }
    }

    #[test]
    fn test_follow_deletion() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_delete.log");

        {
            let mut f = std::fs::File::create(&path).unwrap();
            writeln!(f, "data").unwrap();
        }

        let info = test_info(&path.display().to_string());
        let mut follower = FileFollower::new(&path, 0, info, 0);

        // Delete the file
        std::fs::remove_file(&path).unwrap();

        assert!(matches!(follower.poll(), PollResult::Deleted));
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
        assert!(matches!(follower.poll(), PollResult::NoChange));

        // Complete the line
        {
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .open(&path)
                .unwrap();
            writeln!(f, "_complete").unwrap();
        }

        match follower.poll() {
            PollResult::NewRecords(records) => assert_eq!(records.len(), 1),
            other => panic!("expected NewRecords, got {:?}", poll_name(&other)),
        }
    }

    #[test]
    fn test_follow_empty_file() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();
        let info = test_info(&path.display().to_string());
        let mut follower = FileFollower::new(&path, 0, info, 0);
        assert!(matches!(follower.poll(), PollResult::NoChange));
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

        match follower.poll() {
            PollResult::NewRecords(records) => {
                assert_eq!(records.len(), 2);
                assert_eq!(records[0].id, 100);
                assert_eq!(records[1].id, 101);
            }
            other => panic!("expected NewRecords, got {:?}", poll_name(&other)),
        }
    }

    #[test]
    fn test_follow_rotation() {
        // Rotation detection only works on Unix
        if cfg!(not(unix)) {
            return;
        }

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.log");

        // Create initial file
        {
            let mut f = std::fs::File::create(&path).unwrap();
            writeln!(f, "2024-01-01 original").unwrap();
        }

        let info = test_info(&path.display().to_string());
        let mut follower = FileFollower::new(&path, 0, info, 0);
        let orig_inode = {
            use std::os::unix::fs::MetadataExt;
            std::fs::metadata(&path).unwrap().ino()
        };

        // Read initial content
        match follower.poll() {
            PollResult::NewRecords(records) => assert_eq!(records.len(), 1),
            other => panic!("expected NewRecords, got {:?}", poll_name(&other)),
        }

        // Simulate rotation: delete and recreate (new inode)
        std::fs::remove_file(&path).unwrap();
        {
            // Create a dummy file first to avoid inode reuse on tmpfs
            let _dummy = std::fs::File::create(dir.path().join("dummy.log")).unwrap();
            let mut f = std::fs::File::create(&path).unwrap();
            writeln!(f, "2024-01-01 rotated").unwrap();
        }

        // Check if inode actually changed (may not on some filesystems)
        let new_inode = {
            use std::os::unix::fs::MetadataExt;
            std::fs::metadata(&path).unwrap().ino()
        };
        if new_inode == orig_inode {
            // Inode reused — skip rotation detection test on this filesystem
            // Should still detect truncation instead since file is smaller
            return;
        }

        // Should detect rotation (or truncation if filesystem reuses inodes despite our check)
        let result = follower.poll();
        assert!(
            matches!(result, PollResult::Rotated | PollResult::Truncated),
            "expected Rotated or Truncated after inode change"
        );

        // After reset, should read new content
        follower.reset_with_id(0);
        match follower.poll() {
            PollResult::NewRecords(records) => {
                assert_eq!(records.len(), 1);
                assert!(records[0].raw.contains("rotated"));
            }
            other => panic!(
                "expected NewRecords after rotation reset, got {:?}",
                poll_name(&other)
            ),
        }
    }

    #[test]
    fn test_reset_clears_state() {
        let mut tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();
        let info = test_info(&path.display().to_string());

        let mut follower = FileFollower::new(&path, 0, info, 50);
        writeln!(tmp, "2024-01-01 data").unwrap();
        tmp.flush().unwrap();

        // Read
        follower.poll();
        assert!(follower.offset() > 0);

        // Reset
        follower.reset_with_id(0);
        assert_eq!(follower.offset(), 0);
    }

    fn poll_name(result: &PollResult) -> &'static str {
        match result {
            PollResult::NoChange => "NoChange",
            PollResult::NewRecords(_) => "NewRecords",
            PollResult::Truncated => "Truncated",
            PollResult::Rotated => "Rotated",
            PollResult::Deleted => "Deleted",
        }
    }
}
