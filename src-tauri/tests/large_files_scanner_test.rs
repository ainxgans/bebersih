use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use bebersih_lib::models::CleanCategory;
use bebersih_lib::scanner::stray::{scan_large_files_with_config, LargeFilesScanConfig};
use bebersih_lib::whitelist::WhitelistConfig;

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("bebersih_large_files_{}_{}", name, std::process::id()));
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

fn create_file(path: &Path, size_bytes: usize) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut file = fs::File::create(path).unwrap();
    let data = vec![b'x'; size_bytes];
    file.write_all(&data).unwrap();
}

#[test]
fn test_scan_large_stray_files_detects_above_threshold() {
    let test_dir = TestDir::new("detect");
    let downloads = test_dir.path.join("Downloads");

    // 1. Large ISO image (100 KB in test, threshold 50 KB)
    create_file(&downloads.join("ubuntu-24.04-desktop-amd64.iso"), 100_000);

    // 2. Large archive
    create_file(&downloads.join("backup-2023.tar.gz"), 80_000);

    // 3. Large DB dump
    create_file(&downloads.join("production_dump.sql"), 60_000);

    // 4. Small file (below threshold)
    create_file(&downloads.join("small_notes.txt"), 10_000);
    create_file(&downloads.join("small_archive.zip"), 20_000);

    let config = LargeFilesScanConfig {
        search_dirs: vec![downloads],
        min_size_bytes: 50_000,
        whitelist: WhitelistConfig::default(),
    };

    let items = scan_large_files_with_config(&config).expect("scan large files");
    assert_eq!(items.len(), 3);

    let iso_item = items.iter().find(|i| i.label.contains("ISO")).expect("iso item found");
    assert_eq!(iso_item.category, CleanCategory::LargeFiles);
    assert_eq!(iso_item.size_bytes, 100_000);
    assert_eq!(iso_item.supports_trash, true);
    assert_eq!(iso_item.requires_root, false);
    assert_eq!(iso_item.selected, false, "large stray files should be opt-in unselected");

    let archive_item = items.iter().find(|i| i.label.contains("Archive")).expect("archive item found");
    assert_eq!(archive_item.category, CleanCategory::LargeFiles);
    assert_eq!(archive_item.size_bytes, 80_000);

    let sql_item = items.iter().find(|i| i.label.contains("Database")).expect("sql item found");
    assert_eq!(sql_item.category, CleanCategory::LargeFiles);
    assert_eq!(sql_item.size_bytes, 60_000);
}

#[test]
fn test_scan_large_stray_files_respects_whitelist() {
    let test_dir = TestDir::new("whitelist");
    let downloads = test_dir.path.join("Downloads");
    let keep_iso = downloads.join("keep-this.iso");
    create_file(&keep_iso, 100_000);

    let mut whitelist = WhitelistConfig::default();
    whitelist.whitelisted_paths.push(keep_iso.to_string_lossy().to_string());

    let config = LargeFilesScanConfig {
        search_dirs: vec![downloads],
        min_size_bytes: 50_000,
        whitelist,
    };

    let items = scan_large_files_with_config(&config).expect("scan large files with whitelist");
    assert_eq!(items.len(), 0);
}
