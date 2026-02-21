#[cfg(test)]
mod tests {
    use crate::loader::file::FileLoader;
    use crate::traits::LogLoader;
    use std::io::Write;

    #[test]
    fn test_load_text_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.log");
        let mut f = std::fs::File::create(&file_path).unwrap();
        writeln!(f, "line one").unwrap();
        writeln!(f, "line two").unwrap();
        writeln!(f, "line three").unwrap();
        drop(f);

        let mut loader = FileLoader::new(&file_path, false);
        let lines = loader.load().unwrap();
        assert_eq!(lines, vec!["line one", "line two", "line three"]);
    }

    #[test]
    fn test_load_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("empty.log");
        std::fs::File::create(&file_path).unwrap();

        let mut loader = FileLoader::new(&file_path, false);
        let lines = loader.load().unwrap();
        assert!(lines.is_empty());
    }

    #[test]
    fn test_sample_lines_populated() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("sample.log");
        let mut f = std::fs::File::create(&file_path).unwrap();
        for i in 0..20 {
            writeln!(f, "line {}", i).unwrap();
        }
        drop(f);

        let mut loader = FileLoader::new(&file_path, false);
        loader.load().unwrap();
        assert_eq!(loader.info().sample_lines.len(), 10);
        assert_eq!(loader.info().sample_lines[0], "line 0");
    }

    #[test]
    fn test_load_nonexistent_file() {
        let mut loader = FileLoader::new("/nonexistent/path.log", false);
        assert!(loader.load().is_err());
    }

    #[test]
    fn test_load_gzip_file() {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.log.gz");
        let f = std::fs::File::create(&file_path).unwrap();
        let mut encoder = GzEncoder::new(f, Compression::default());
        encoder
            .write_all(b"line one\nline two\nline three\n")
            .unwrap();
        encoder.finish().unwrap();

        let mut loader = FileLoader::new(&file_path, false);
        let lines = loader.load().unwrap();
        assert_eq!(lines, vec!["line one", "line two", "line three"]);
    }

    #[test]
    fn test_load_corrupted_gzip_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("corrupt.gz");
        let mut f = std::fs::File::create(&file_path).unwrap();
        f.write_all(b"this is not gzip data").unwrap();
        drop(f);

        let mut loader = FileLoader::new(&file_path, false);
        let result = loader.load();
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("decompress") || err_msg.contains("gzip"));
    }

    #[test]
    fn test_load_empty_gzip_file() {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("empty.log.gz");
        let f = std::fs::File::create(&file_path).unwrap();
        let encoder = GzEncoder::new(f, Compression::default());
        encoder.finish().unwrap();

        let mut loader = FileLoader::new(&file_path, false);
        let lines = loader.load().unwrap();
        assert!(lines.is_empty());
    }

    #[test]
    fn test_loader_info() {
        let loader = FileLoader::new("/tmp/test.log", true);
        let info = loader.info();
        assert_eq!(info.id, "/tmp/test.log");
        assert_eq!(info.loader_type, crate::traits::LoaderType::TextFile);
        assert!(info.multiline_enabled);
        assert!(info.sample_lines.is_empty());
    }

    #[test]
    fn test_file_mod_year_populated() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("dated.log");
        let mut f = std::fs::File::create(&file_path).unwrap();
        writeln!(f, "test line").unwrap();
        drop(f);

        let mut loader = FileLoader::new(&file_path, false);
        loader.load().unwrap();
        let year = loader.info().file_mod_year;
        assert!(year.is_some(), "file_mod_year should be populated after load");
        assert!(year.unwrap() >= 2020, "file_mod_year should be a reasonable year");
    }
}
