#[cfg(test)]
mod tests {
    use crate::loader::archive::{ArchiveFormat, ArchiveLoader};
    use crate::traits::LogLoader;
    use std::io::Write;

    #[test]
    fn test_detect_format_gz() {
        let loader = ArchiveLoader::new("/tmp/test.log.gz", false).unwrap();
        assert_eq!(loader.format, ArchiveFormat::Gzip);
    }

    #[test]
    fn test_detect_format_zip() {
        let loader = ArchiveLoader::new("/tmp/test.zip", false).unwrap();
        assert_eq!(loader.format, ArchiveFormat::Zip);
    }

    #[test]
    fn test_detect_format_7z() {
        let loader = ArchiveLoader::new("/tmp/test.7z", false).unwrap();
        assert_eq!(loader.format, ArchiveFormat::SevenZ);
    }

    #[test]
    fn test_detect_format_unknown() {
        let result = ArchiveLoader::new("/tmp/test.tar", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_gzip() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.log.gz");

        // Create a gzip file
        let file = std::fs::File::create(&file_path).unwrap();
        let mut encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        write!(encoder, "hello\nworld\n").unwrap();
        encoder.finish().unwrap();

        let mut loader = ArchiveLoader::new(&file_path, false).unwrap();
        let lines = loader.load().unwrap();
        assert_eq!(lines, vec!["hello", "world"]);
        assert_eq!(loader.info().sample_lines.len(), 2);
    }

    #[test]
    fn test_load_zip() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.zip");

        // Create a zip file with one entry
        let file = std::fs::File::create(&file_path).unwrap();
        let mut zip_writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip_writer.start_file("log.txt", options).unwrap();
        write!(zip_writer, "line1\nline2\nline3\n").unwrap();
        zip_writer.finish().unwrap();

        let mut loader = ArchiveLoader::new(&file_path, false).unwrap();
        let lines = loader.load().unwrap();
        assert_eq!(lines, vec!["line1", "line2", "line3"]);
    }

    #[test]
    fn test_load_zip_multiple_files() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("multi.zip");

        let file = std::fs::File::create(&file_path).unwrap();
        let mut zip_writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        zip_writer.start_file("a.log", options).unwrap();
        write!(zip_writer, "alpha\n").unwrap();
        zip_writer.start_file("b.log", options).unwrap();
        write!(zip_writer, "beta\n").unwrap();
        zip_writer.finish().unwrap();

        let mut loader = ArchiveLoader::new(&file_path, false).unwrap();
        let lines = loader.load().unwrap();
        assert_eq!(lines, vec!["alpha", "beta"]);
    }

    #[test]
    fn test_loader_info() {
        let loader = ArchiveLoader::new("/tmp/test.gz", true).unwrap();
        let info = loader.info();
        assert_eq!(info.loader_type, crate::traits::LoaderType::Archive);
        assert!(info.multiline_enabled);
    }
}
