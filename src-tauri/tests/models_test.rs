use bebersih_lib::error::AppError;
use bebersih_lib::models::{CleanCategory, CleanItem, CleanReport, ScanProgress};

#[test]
fn test_clean_category_variants_serde() {
    let categories = vec![
        CleanCategory::DevRust,
        CleanCategory::DevGo,
        CleanCategory::DevNode,
        CleanCategory::DevPython,
        CleanCategory::DevJvm,
        CleanCategory::DockerCache,
        CleanCategory::DockerVolume,
        CleanCategory::AptCache,
        CleanCategory::SystemLogs,
        CleanCategory::StaleCache,
        CleanCategory::BrowserCache,
        CleanCategory::UserTrash,
        CleanCategory::SnapFlatpak,
        CleanCategory::IdeCache,
        CleanCategory::LargeFiles,
    ];

    for cat in categories {
        let json = serde_json::to_string(&cat).expect("serialize CleanCategory");
        let deserialized: CleanCategory =
            serde_json::from_str(&json).expect("deserialize CleanCategory");
        assert_eq!(cat, deserialized);
    }
}

#[test]
fn test_clean_item_roundtrip_serde() {
    let item = CleanItem {
        id: "item-1".to_string(),
        category: CleanCategory::DevRust,
        label: "Rust target directory".to_string(),
        path: Some("/home/user/project/target".to_string()),
        size_bytes: 1024 * 1024 * 50, // 50MB
        requires_root: false,
        supports_trash: true,
        selected: true,
    };

    let json = serde_json::to_string(&item).expect("serialize CleanItem");
    let deserialized: CleanItem = serde_json::from_str(&json).expect("deserialize CleanItem");

    assert_eq!(item, deserialized);
    assert_eq!(deserialized.id, "item-1");
    assert_eq!(deserialized.category, CleanCategory::DevRust);
    assert_eq!(deserialized.path, Some("/home/user/project/target".to_string()));
    assert_eq!(deserialized.size_bytes, 52428800);
    assert!(!deserialized.requires_root);
    assert!(deserialized.supports_trash);
    assert!(deserialized.selected);
}

#[test]
fn test_scan_progress_and_clean_report_serde() {
    let progress = ScanProgress {
        scanned_items: 42,
        current_path: "/var/log".to_string(),
        found_bytes: 1024 * 500,
    };
    let progress_json = serde_json::to_string(&progress).expect("serialize ScanProgress");
    let deser_progress: ScanProgress =
        serde_json::from_str(&progress_json).expect("deserialize ScanProgress");
    assert_eq!(progress, deser_progress);

    let report = CleanReport {
        deleted_bytes: 1024 * 1024 * 10,
        freed_items: vec!["item-1".to_string(), "item-2".to_string()],
        errors: vec![],
    };
    let report_json = serde_json::to_string(&report).expect("serialize CleanReport");
    let deser_report: CleanReport =
        serde_json::from_str(&report_json).expect("deserialize CleanReport");
    assert_eq!(report, deser_report);
}

#[test]
fn test_app_error_display_and_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "cannot access directory");
    let app_err = AppError::from(io_err);
    assert!(matches!(app_err, AppError::Io(_)));
    assert!(app_err.to_string().contains("cannot access directory"));

    let err = AppError::PermissionDenied("need root".to_string());
    assert_eq!(err.to_string(), "Permission denied: need root");

    let json = serde_json::to_string(&err).expect("serialize AppError");
    let deserialized: AppError = serde_json::from_str(&json).expect("deserialize AppError");
    assert_eq!(err, deserialized);
}
