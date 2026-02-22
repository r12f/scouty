#[cfg(test)]
mod tests {
    use crate::loader::stdin::StdinLoader;
    use crate::traits::{LoaderType, LogLoader};

    #[test]
    fn test_stdin_loader_info() {
        let loader = StdinLoader::new();
        let info = loader.info();
        assert_eq!(info.id, "<stdin>");
        assert_eq!(info.loader_type, LoaderType::TextFile);
        assert!(!info.multiline_enabled);
        assert!(info.sample_lines.is_empty());
    }
}
