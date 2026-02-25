//! Tests for RegionStore.

mod tests {
    use crate::region::store::RegionStore;
    use crate::region::Region;
    use std::collections::HashMap;

    fn make_region(name: &str, start: usize, end: usize) -> Region {
        Region {
            definition_name: "test".to_string(),
            name: name.to_string(),
            description: None,
            start_reason: None,
            end_reason: None,
            start_index: start,
            end_index: end,
            metadata: HashMap::new(),
            timed_out: false,
        }
    }

    #[test]
    fn test_regions_at_single() {
        let store = RegionStore::from_regions(vec![make_region("A", 5, 10)]);
        assert_eq!(store.regions_at(3).len(), 0);
        assert_eq!(store.regions_at(5).len(), 1);
        assert_eq!(store.regions_at(7).len(), 1);
        assert_eq!(store.regions_at(10).len(), 1);
        assert_eq!(store.regions_at(11).len(), 0);
    }

    #[test]
    fn test_regions_at_overlapping() {
        let store = RegionStore::from_regions(vec![
            make_region("outer", 2, 20),
            make_region("inner", 5, 10),
        ]);
        assert_eq!(store.regions_at(1).len(), 0);
        assert_eq!(store.regions_at(3).len(), 1); // outer only
        assert_eq!(store.regions_at(7).len(), 2); // both
        assert_eq!(store.regions_at(15).len(), 1); // outer only
    }

    #[test]
    fn test_innermost_at() {
        let store = RegionStore::from_regions(vec![
            make_region("outer", 2, 20),
            make_region("inner", 5, 10),
        ]);
        let inner = store.innermost_at(7).unwrap();
        assert_eq!(inner.name, "inner");
    }

    #[test]
    fn test_innermost_at_no_region() {
        let store = RegionStore::from_regions(vec![make_region("A", 5, 10)]);
        assert!(store.innermost_at(20).is_none());
    }

    #[test]
    fn test_push_maintains_order() {
        let mut store = RegionStore::new();
        store.push(make_region("B", 10, 20));
        store.push(make_region("A", 3, 8));
        store.push(make_region("C", 15, 25));
        assert_eq!(store.regions()[0].name, "A");
        assert_eq!(store.regions()[1].name, "B");
        assert_eq!(store.regions()[2].name, "C");
    }

    #[test]
    fn test_empty_store() {
        let store = RegionStore::new();
        assert!(store.is_empty());
        assert_eq!(store.regions_at(5).len(), 0);
        assert!(store.innermost_at(5).is_none());
    }

    #[test]
    fn test_timed_out_regions_included() {
        let mut r = make_region("timeout", 5, 10);
        r.timed_out = true;
        let store = RegionStore::from_regions(vec![r]);
        assert_eq!(store.regions_at(7).len(), 1);
        assert!(store.regions_at(7)[0].timed_out);
    }
}
