use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use bebersih_lib::models::CleanCategory;
use bebersih_lib::scanner::system::{
    parse_journal_disk_usage, scan_system_caches, scan_system_caches_with_config, SystemScanConfig,
};

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("bebersih_{}_{}", name, timestamp));
        fs::create_dir_all(&path).expect("failed to create temp test dir");
        Self { path }
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn create_file(path: &Path, size_bytes: usize) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent dir");
    }
    let mut file = File::create(path).expect("failed to create file");
    let content = vec![b'x'; size_bytes];
    file.write_all(&content).expect("failed to write content");
}

fn set_file_mtime_days_ago(path: &Path, days_ago: u64) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let target_time = now.saturating_sub(days_ago * 86400);

    let status = std::process::Command::new("touch")
        .args(["-d", &format!("@{}", target_time), path.to_str().unwrap()])
        .status()
        .expect("failed to run touch command");
    assert!(status.success(), "touch command failed to set mtime");
}

#[test]
fn test_parse_journal_disk_usage() {
    assert_eq!(
        parse_journal_disk_usage("Archived and active journals take up 2.6G in the file system."),
        Some(2791728742)
    );
    assert_eq!(
        parse_journal_disk_usage("Journals take up 45.0M on disk."),
        Some(47185920)
    );
    assert_eq!(
        parse_journal_disk_usage("Archived and active journals take up 512.0K on disk."),
        Some(524288)
    );
    assert_eq!(
        parse_journal_disk_usage("Journals take up 1024B in the file system."),
        Some(1024)
    );
    assert_eq!(parse_journal_disk_usage("Invalid output with no size"), None);
    assert_eq!(parse_journal_disk_usage(""), None);
}

#[test]
fn test_scan_system_caches_with_mock_dirs() {
    let test_dir = TestDir::new("system_scan");
    let base = &test_dir.path;

    // 1. Mock APT cache (/var/cache/apt/archives)
    let apt_dir = base.join("apt_archives");
    fs::create_dir_all(&apt_dir).unwrap();
    let deb1 = apt_dir.join("pkg1_1.0_amd64.deb");
    let deb2 = apt_dir.join("pkg2_2.0_amd64.deb");
    let apt_lock = apt_dir.join("lock");
    create_file(&deb1, 2048);
    create_file(&deb2, 4096);
    create_file(&apt_lock, 0);

    // 2. Mock Thumbnails cache (~/.cache/thumbnails)
    let thumb_dir = base.join("cache").join("thumbnails");
    let thumb_file = thumb_dir.join("normal").join("thumb1.png");
    create_file(&thumb_file, 8192);

    // 3. Mock User cache (~/.cache) with stale and fresh files
    let user_cache = base.join("cache");
    let stale_file = user_cache.join("app1").join("old_cache.log");
    let fresh_file = user_cache.join("app2").join("recent_cache.log");
    create_file(&stale_file, 1500);
    create_file(&fresh_file, 3000);
    set_file_mtime_days_ago(&stale_file, 40); // 40 days old > 30 days
    set_file_mtime_days_ago(&fresh_file, 1);  // 1 day old < 30 days

    // 4. Mock Crash logs (/var/crash/*.crash)
    let crash_dir = base.join("crash");
    fs::create_dir_all(&crash_dir).unwrap();
    let old_crash = crash_dir.join("app_crash.crash");
    let recent_crash = crash_dir.join("fresh_crash.crash");
    let other_file = crash_dir.join("kdump_lock");
    create_file(&old_crash, 16384);
    create_file(&recent_crash, 8192);
    create_file(&other_file, 1024);
    set_file_mtime_days_ago(&old_crash, 10);    // 10 days old > 7 days
    set_file_mtime_days_ago(&recent_crash, 2); // 2 days old < 7 days
    set_file_mtime_days_ago(&other_file, 15);  // Not .crash extension

    let config = SystemScanConfig {
        apt_cache_dir: Some(apt_dir.clone()),
        thumbnails_dir: Some(thumb_dir.clone()),
        user_cache_dir: Some(user_cache),
        crash_dir: Some(crash_dir.clone()),
        journal_size_override: Some(50 * 1024 * 1024),
    };

    let items = scan_system_caches_with_config(30, &config)
        .expect("scan_system_caches_with_config should succeed");

    // Verify APT cache item
    let apt_item = items
        .iter()
        .find(|i| i.category == CleanCategory::AptCache)
        .expect("APT cache item should be detected");
    assert_eq!(apt_item.path.as_deref(), apt_dir.to_str());
    assert_eq!(apt_item.size_bytes, 6144);
    assert!(apt_item.requires_root);
    assert!(!apt_item.supports_trash);
    assert!(apt_item.selected);

    // Verify Thumbnails item
    let thumb_item = items
        .iter()
        .find(|i| i.category == CleanCategory::StaleCache && i.id.starts_with("system:thumbnails:"))
        .expect("Thumbnail cache item should be detected");
    assert_eq!(thumb_item.path.as_deref(), thumb_dir.to_str());
    assert_eq!(thumb_item.size_bytes, 8192);
    assert!(!thumb_item.requires_root);
    assert!(!thumb_item.supports_trash);
    assert!(thumb_item.selected);

    // Verify Stale cache item (stale_file detected, fresh_file and thumbnails not duplicated)
    let stale_item = items
        .iter()
        .find(|i| i.path.as_deref() == stale_file.to_str())
        .expect("Stale file should be detected");
    assert_eq!(stale_item.category, CleanCategory::StaleCache);
    assert_eq!(stale_item.size_bytes, 1500);
    assert!(!stale_item.requires_root);
    assert!(!stale_item.supports_trash);

    assert!(
        items.iter().all(|i| i.path.as_deref() != fresh_file.to_str()),
        "Fresh file must not be detected as stale"
    );
    assert!(
        items
            .iter()
            .all(|i| i.path.as_deref() != thumb_file.to_str()),
        "Thumbnail child files should not be duplicated as generic stale cache"
    );

    // Verify Crash log item (old_crash detected, recent_crash and other_file skipped)
    let crash_item = items
        .iter()
        .find(|i| i.path.as_deref() == old_crash.to_str())
        .expect("Old crash file should be detected");
    assert_eq!(crash_item.category, CleanCategory::SystemLogs);
    assert_eq!(crash_item.size_bytes, 16384);
    assert!(crash_item.requires_root);
    assert!(!crash_item.supports_trash);

    assert!(
        items.iter().all(|i| i.path.as_deref() != recent_crash.to_str()),
        "Recent crash file (<7 days) must not be detected"
    );
    assert!(
        items.iter().all(|i| i.path.as_deref() != other_file.to_str()),
        "Non-crash file must not be detected as crash log"
    );

    // Verify Journalctl item
    let journal_item = items
        .iter()
        .find(|i| i.id == "system:journalctl")
        .expect("Journalctl item should be detected");
    assert_eq!(journal_item.category, CleanCategory::SystemLogs);
    assert_eq!(journal_item.size_bytes, 50 * 1024 * 1024);
    assert_eq!(journal_item.path, None);
    assert!(journal_item.requires_root);
}

#[test]
fn test_graceful_handling_missing_dirs() {
    let non_existent = PathBuf::from("/non/existent/bebersih_path_test_12345");
    let config = SystemScanConfig {
        apt_cache_dir: Some(non_existent.join("apt")),
        thumbnails_dir: Some(non_existent.join("thumbnails")),
        user_cache_dir: Some(non_existent.join("cache")),
        crash_dir: Some(non_existent.join("crash")),
        journal_size_override: Some(0),
    };

    let items = scan_system_caches_with_config(30, &config)
        .expect("missing directories should be handled gracefully without error");
    assert!(items.is_empty(), "items should be empty for non-existent paths");
}

#[test]
fn test_scan_system_caches_default_does_not_panic() {
    // Verifies calling the public default entry point completes cleanly on the host system
    let result = scan_system_caches(30);
    assert!(result.is_ok(), "scan_system_caches should return Ok on real system");
}
