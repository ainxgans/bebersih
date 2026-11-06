use std::fs;
use std::path::PathBuf;

use bebersih_lib::history::{
    get_cleanup_stats, load_history_from_file, record_cleanup_to_file,
};
use bebersih_lib::models::{CleanCategory, CleanItem, CleanReport};

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("bebersih_hist_test_{}_{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&mut self.path);
    }
}

#[test]
fn test_history_records_and_aggregates_cleanup() {
    let test_dir = TestDir::new("records");
    let history_file = test_dir.path.join("history.json");

    let items = vec![
        CleanItem {
            id: "dev:rust:app".to_string(),
            category: CleanCategory::DevRust,
            label: "target".to_string(),
            path: Some("/tmp/app/target".to_string()),
            size_bytes: 1048576,
            requires_root: false,
            supports_trash: true,
            selected: true,
        },
        CleanItem {
            id: "browser:chrome:cache".to_string(),
            category: CleanCategory::BrowserCache,
            label: "Cache".to_string(),
            path: Some("/tmp/cache".to_string()),
            size_bytes: 2097152,
            requires_root: false,
            supports_trash: false,
            selected: true,
        },
    ];

    let report = CleanReport {
        deleted_bytes: 3145728,
        freed_items: vec!["dev:rust:app".to_string(), "browser:chrome:cache".to_string()],
        errors: vec![],
    };

    let history = record_cleanup_to_file(&report, &items, &history_file).expect("record cleanup");
    assert_eq!(history.total_freed_bytes, 3145728);
    assert_eq!(history.total_clean_runs, 1);
    assert_eq!(history.records.len(), 1);
    assert_eq!(history.records[0].deleted_bytes, 3145728);
    assert!(history.records[0].categories.contains(&CleanCategory::DevRust));
    assert!(history.records[0].categories.contains(&CleanCategory::BrowserCache));

    // Second run
    let report2 = CleanReport {
        deleted_bytes: 1000000,
        freed_items: vec!["trash:item".to_string()],
        errors: vec![],
    };
    let items2 = vec![CleanItem {
        id: "trash:item".to_string(),
        category: CleanCategory::UserTrash,
        label: "trash".to_string(),
        path: None,
        size_bytes: 1000000,
        requires_root: false,
        supports_trash: false,
        selected: true,
    }];

    let history2 = record_cleanup_to_file(&report2, &items2, &history_file).expect("second record cleanup");
    assert_eq!(history2.total_freed_bytes, 4145728);
    assert_eq!(history2.total_clean_runs, 2);
    assert_eq!(history2.records.len(), 2);

    // Verify loading persisted file
    let loaded = load_history_from_file(&history_file).expect("load history file");
    assert_eq!(loaded, history2);

    let stats = get_cleanup_stats(&loaded);
    assert_eq!(stats.total_freed_bytes, 4145728);
    assert_eq!(stats.total_runs, 2);
}
