#[cfg(test)]
mod tests {
    use crate::category::*;
    use crate::filter::expr;
    use std::io::Write;
    use std::path::Path;
    use tempfile::TempDir;

    fn make_definition(name: &str, filter_str: &str) -> CategoryDefinition {
        CategoryDefinition {
            name: name.to_string(),
            filter: expr::parse(filter_str).unwrap(),
        }
    }

    // ── CategoryStats tests ─────────────────────────────────────────

    #[test]
    fn test_category_stats_new() {
        let def = make_definition("test", "level == \"error\"");
        let stats = CategoryStats::new(def, 10);
        assert_eq!(stats.count, 0);
        assert_eq!(stats.density.len(), 10);
        assert!(stats.density.iter().all(|&v| v == 0));
    }

    #[test]
    fn test_category_stats_record_match() {
        let def = make_definition("test", "level == \"error\"");
        let mut stats = CategoryStats::new(def, 5);

        stats.record_match(Some(2));
        assert_eq!(stats.count, 1);
        assert_eq!(stats.density[2], 1);

        stats.record_match(Some(2));
        assert_eq!(stats.count, 2);
        assert_eq!(stats.density[2], 2);

        // No bucket
        stats.record_match(None);
        assert_eq!(stats.count, 3);

        // Out of bounds bucket (should not panic)
        stats.record_match(Some(100));
        assert_eq!(stats.count, 4);
    }

    #[test]
    fn test_category_stats_resize_density() {
        let def = make_definition("test", "level == \"error\"");
        let mut stats = CategoryStats::new(def, 5);
        stats.density[0] = 10;
        stats.resize_density(8);
        assert_eq!(stats.density.len(), 8);
        assert_eq!(stats.density[0], 10); // preserved
        assert_eq!(stats.density[7], 0); // new zeros
    }

    // ── CategoryStore tests ─────────────────────────────────────────

    #[test]
    fn test_store_from_definitions() {
        let defs = vec![
            make_definition("errors", "level == \"error\""),
            make_definition("warnings", "level == \"warning\""),
        ];
        let store = CategoryStore::from_definitions(defs, 20);
        assert_eq!(store.categories.len(), 2);
        assert_eq!(store.categories[0].definition.name, "errors");
        assert_eq!(store.categories[1].definition.name, "warnings");
    }

    #[test]
    fn test_store_reset() {
        let defs = vec![make_definition("test", "level == \"error\"")];
        let mut store = CategoryStore::from_definitions(defs, 5);
        store.categories[0].count = 42;
        store.categories[0].density[0] = 10;
        store.reset();
        assert_eq!(store.categories[0].count, 0);
        assert!(store.categories[0].density.iter().all(|&v| v == 0));
    }

    // ── Config loading tests ────────────────────────────────────────

    #[test]
    fn test_load_file_valid() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.yaml");
        let mut f = std::fs::File::create(&file).unwrap();
        write!(
            f,
            r#"categories:
  - name: "Errors"
    filter: 'level == "error"'
  - name: "Warnings"
    filter: 'level == "warning"'
"#
        )
        .unwrap();

        let (defs, warnings) = load_file(&file);
        assert!(warnings.is_empty(), "warnings: {:?}", warnings);
        assert_eq!(defs.len(), 2);
        assert_eq!(defs[0].name, "Errors");
        assert_eq!(defs[1].name, "Warnings");
    }

    #[test]
    fn test_load_file_invalid_filter_skipped() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.yaml");
        let mut f = std::fs::File::create(&file).unwrap();
        write!(
            f,
            r#"categories:
  - name: "Good"
    filter: 'level == "error"'
  - name: "Bad"
    filter: '=== invalid ==='
  - name: "Also Good"
    filter: 'message contains "hello"'
"#
        )
        .unwrap();

        let (defs, warnings) = load_file(&file);
        assert_eq!(defs.len(), 2, "Should skip invalid filter");
        assert_eq!(defs[0].name, "Good");
        assert_eq!(defs[1].name, "Also Good");
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("Bad"));
    }

    #[test]
    fn test_load_file_invalid_yaml() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("bad.yaml");
        std::fs::write(&file, "not: valid: yaml: [[[").unwrap();

        let (defs, warnings) = load_file(&file);
        assert!(defs.is_empty());
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn test_load_file_missing() {
        let (defs, warnings) = load_file(Path::new("/nonexistent/path.yaml"));
        assert!(defs.is_empty());
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn test_load_file_complex_filters() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.yaml");
        let mut f = std::fs::File::create(&file).unwrap();
        write!(
            f,
            r#"categories:
  - name: "Complex AND"
    filter: 'component == "bgp" AND level >= "info"'
  - name: "Complex OR"
    filter: 'level == "error" OR level == "critical"'
  - name: "Contains"
    filter: 'message contains "link state"'
"#
        )
        .unwrap();

        let (defs, warnings) = load_file(&file);
        assert!(warnings.is_empty(), "warnings: {:?}", warnings);
        assert_eq!(defs.len(), 3);
    }
}
