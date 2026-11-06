use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use bebersih_lib::models::CleanCategory;
use bebersih_lib::scanner::trash::{
    empty_user_trash_with_config, scan_user_trash_with_config, TrashScanConfig,
};

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("bebersih_trash_test_{}_{}", name, std::process::id()));
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
    let data = vec![b'a'; size_bytes];
    file.write_all(&data).unwrap();
}

#[test]
fn test_scan_user_trash_empty_or_nonexistent() {
    let test_dir = TestDir::new("nonexistent");
    let fake_trash = test_dir.path.join("does_not_exist");
    let config = TrashScanConfig {
        trash_dir: Some(fake_trash),
    };

    let items = scan_user_trash_with_config(&config).expect("scan should succeed even if trash missing");
    assert!(items.is_empty());
}

#[test]
fn test_scan_user_trash_detects_files_and_directories() {
    let test_dir = TestDir::new("detect");
    let trash_dir = test_dir.path.join("Trash");
    let files_dir = trash_dir.join("files");
    let info_dir = trash_dir.join("info");

    fs::create_dir_all(&files_dir).unwrap();
    fs::create_dir_all(&info_dir).unwrap();

    // 1. Single file in Trash/files with corresponding .trashinfo
    let single_file = files_dir.join("report.pdf");
    create_file(&single_file, 5000);

    let info_content = "[Trash Info]\nPath=/home/user/Documents/report.pdf\nDeletionDate=2026-11-05T10:00:00\n";
    let info_file = info_dir.join("report.pdf.trashinfo");
    fs::write(&info_file, info_content).unwrap();

    // 2. Folder in Trash/files
    let subfolder = files_dir.join("my_old_project");
    let subfile = subfolder.join("src/main.rs");
    create_file(&subfile, 10000);

    let config = TrashScanConfig {
        trash_dir: Some(trash_dir.clone()),
    };

    let items = scan_user_trash_with_config(&config).expect("scan should succeed");
    assert_eq!(items.len(), 2);

    let file_item = items.iter().find(|i| i.id == "trash:report.pdf").expect("file item found");
    assert_eq!(file_item.category, CleanCategory::UserTrash);
    assert!(file_item.size_bytes >= 5000);
    assert_eq!(file_item.supports_trash, false);
    assert_eq!(file_item.requires_root, false);
    assert!(file_item.label.contains("report.pdf"));

    let folder_item = items.iter().find(|i| i.id == "trash:my_old_project").expect("folder item found");
    assert_eq!(folder_item.category, CleanCategory::UserTrash);
    assert!(folder_item.size_bytes >= 10000);
    assert_eq!(folder_item.supports_trash, false);
}

#[test]
fn test_empty_user_trash() {
    let test_dir = TestDir::new("empty");
    let trash_dir = test_dir.path.join("Trash");
    let files_dir = trash_dir.join("files");
    let info_dir = trash_dir.join("info");

    create_file(&files_dir.join("item1.txt"), 1024);
    create_file(&files_dir.join("dir1/nested.txt"), 2048);
    create_file(&info_dir.join("item1.txt.trashinfo"), 120);

    let config = TrashScanConfig {
        trash_dir: Some(trash_dir.clone()),
    };

    let report = empty_user_trash_with_config(&config).expect("empty trash should succeed");
    assert!(report.deleted_bytes >= 1024 + 2048);
    assert!(report.errors.is_empty());

    // files_dir should now be empty
    let files_count = fs::read_dir(&files_dir).map(|rd| rd.count()).unwrap_or(0);
    assert_eq!(files_count, 0, "files directory should be empty");
    let info_count = fs::read_dir(&info_dir).map(|rd| rd.count()).unwrap_or(0);
    assert_eq!(info_count, 0, "info directory should be empty");
}
