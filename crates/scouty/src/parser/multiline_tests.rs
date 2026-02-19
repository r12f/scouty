#[cfg(test)]
mod tests {
    use crate::parser::multiline::MultilineMerger;

    fn lines(input: &[&str]) -> Vec<String> {
        input.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_no_multiline() {
        let merger = MultilineMerger::new(r"^\d{4}-\d{2}-\d{2}", "\n").unwrap();
        let input = lines(&[
            "2024-01-15 10:00:00 INFO first",
            "2024-01-15 10:00:01 INFO second",
        ]);
        let blocks = merger.merge(&input);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0], "2024-01-15 10:00:00 INFO first");
        assert_eq!(blocks[1], "2024-01-15 10:00:01 INFO second");
    }

    #[test]
    fn test_java_stack_trace() {
        let merger = MultilineMerger::new(r"^\d{4}-\d{2}-\d{2}", "\n").unwrap();
        let input = lines(&[
            "2024-01-15 10:00:00 ERROR NullPointerException",
            "    at com.example.Foo.bar(Foo.java:42)",
            "    at com.example.Main.main(Main.java:10)",
            "2024-01-15 10:00:01 INFO Recovery complete",
        ]);
        let blocks = merger.merge(&input);
        assert_eq!(blocks.len(), 2);
        assert!(blocks[0].contains("NullPointerException"));
        assert!(blocks[0].contains("Foo.java:42"));
        assert!(blocks[0].contains("Main.java:10"));
        assert_eq!(blocks[1], "2024-01-15 10:00:01 INFO Recovery complete");
    }

    #[test]
    fn test_trailing_multiline() {
        let merger = MultilineMerger::new(r"^\d{4}-\d{2}-\d{2}", "\n").unwrap();
        let input = lines(&[
            "2024-01-15 10:00:00 ERROR crash",
            "  detail line 1",
            "  detail line 2",
        ]);
        let blocks = merger.merge(&input);
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0],
            "2024-01-15 10:00:00 ERROR crash\n  detail line 1\n  detail line 2"
        );
    }

    #[test]
    fn test_empty_input() {
        let merger = MultilineMerger::new(r"^\d{4}", "\n").unwrap();
        let blocks = merger.merge(&[]);
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_orphan_continuation_lines() {
        let merger = MultilineMerger::new(r"^\d{4}-\d{2}-\d{2}", "\n").unwrap();
        let input = lines(&[
            "  orphan line 1",
            "  orphan line 2",
            "2024-01-15 10:00:00 INFO normal",
        ]);
        let blocks = merger.merge(&input);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0], "  orphan line 1\n  orphan line 2");
        assert_eq!(blocks[1], "2024-01-15 10:00:00 INFO normal");
    }

    #[test]
    fn test_custom_separator() {
        let merger = MultilineMerger::new(r"^>>", " | ").unwrap();
        let input = lines(&[">> start", "cont1", "cont2"]);
        let blocks = merger.merge(&input);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0], ">> start | cont1 | cont2");
    }

    #[test]
    fn test_invalid_pattern() {
        let result = MultilineMerger::new("[invalid", "\n");
        assert!(result.is_err());
    }

    #[test]
    fn test_single_line() {
        let merger = MultilineMerger::new(r"^.", "\n").unwrap();
        let input = lines(&["just one line"]);
        let blocks = merger.merge(&input);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0], "just one line");
    }
}
