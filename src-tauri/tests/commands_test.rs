use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bebersih_lib::commands::{
    add_to_whitelist, clean_selected, get_cleanup_history, get_disk_free_space, get_whitelist,
    remove_from_whitelist, scan_all,
};
use bebersih_lib::models::{CleanCategory, CleanItem};

static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("bebersih_cmd_{}_{}", name, timestamp));
        fs::create_dir_all(&path).expect("failed to create temp dir");
        Self { path }
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn create_file(path: &PathBuf, size_bytes: usize) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent dir");
    }
    let mut file = File::create(path).expect("failed to create file");
    let content = vec![b'x'; size_bytes];
    file.write_all(&content).expect("failed to write content");
}

#[tokio::test]
async fn test_get_disk_free_space_returns_positive_number() {
    let free_bytes = get_disk_free_space().await.expect("disk free space should succeed");
    assert!(free_bytes > 0, "free disk space must be greater than 0");
}

#[tokio::test]
async fn test_scan_all_with_custom_paths() {
    let test_dir = TestDir::new("scan_all");
    let base = &test_dir.path;

    // Create a mock Rust project
    let rust_dir = base.join("mock_app");
    fs::create_dir_all(&rust_dir).unwrap();
    fs::write(rust_dir.join("Cargo.toml"), "[package]\nname = \"mock\"").unwrap();
    let rust_target = rust_dir.join("target");
    create_file(&rust_target.join("debug").join("mock_bin"), 2048);

    let paths = vec![base.to_string_lossy().to_string()];
    let items = scan_all(paths).await.expect("scan_all should succeed");

    assert!(!items.is_empty(), "items should not be empty");
    let target_item = items
        .iter()
        .find(|item| item.path.as_deref() == rust_target.to_str());
    assert!(target_item.is_some(), "should find rust target item");
    let item = target_item.unwrap();
    assert_eq!(item.category, CleanCategory::DevRust);
    assert!(item.size_bytes >= 2048);
}

#[tokio::test]
async fn test_clean_selected_purges_temporary_files() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let test_dir = TestDir::new("clean_purge");
    let test_file = test_dir.path.join("junk.tmp");
    create_file(&test_file, 1024);
    assert!(test_file.exists());

    let item = CleanItem {
        id: "junk-1".to_string(),
        category: CleanCategory::StaleCache,
        label: "Junk temp file".to_string(),
        path: Some(test_file.to_string_lossy().to_string()),
        size_bytes: 1024,
        requires_root: false,
        supports_trash: false,
        selected: true,
    };

    let report = clean_selected(vec![item], false).await.expect("clean_selected should succeed");
    assert_eq!(report.deleted_bytes, 1024);
    assert_eq!(report.freed_items, vec!["junk-1".to_string()]);
    assert!(report.errors.is_empty());
    assert!(!test_file.exists(), "file should be deleted");
}

#[tokio::test]
async fn test_clean_selected_rejects_unsafe_paths() {
    let item = CleanItem {
        id: "unsafe-root".to_string(),
        category: CleanCategory::SystemLogs,
        label: "Root system path".to_string(),
        path: Some("/etc".to_string()),
        size_bytes: 4096,
        requires_root: false,
        supports_trash: false,
        selected: true,
    };

    let report = clean_selected(vec![item], false).await.expect("clean_selected returns report");
    assert_eq!(report.deleted_bytes, 0);
    assert!(report.freed_items.is_empty());
    assert!(!report.errors.is_empty(), "should report error for unsafe path");
    assert!(std::path::Path::new("/etc").exists(), "/etc must still exist");
}

#[tokio::test]
async fn test_clean_selected_skips_unselected_items() {
    let test_dir = TestDir::new("clean_unselected");
    let test_file = test_dir.path.join("keep.tmp");
    create_file(&test_file, 512);

    let item = CleanItem {
        id: "keep-1".to_string(),
        category: CleanCategory::StaleCache,
        label: "Keep this file".to_string(),
        path: Some(test_file.to_string_lossy().to_string()),
        size_bytes: 512,
        requires_root: false,
        supports_trash: false,
        selected: false,
    };

    let report = clean_selected(vec![item], false).await.expect("clean_selected should succeed");
    assert_eq!(report.deleted_bytes, 0);
    assert!(report.freed_items.is_empty());
    assert!(test_file.exists(), "unselected file must remain untouched");
}

#[tokio::test]
async fn test_clean_selected_purges_browser_cache_item() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let test_dir = TestDir::new("clean_browser");
    let cache_dir = test_dir.path.join(".cache/google-chrome/Default/Cache");
    fs::create_dir_all(&cache_dir).unwrap();
    let data_file = cache_dir.join("data_0");
    create_file(&data_file, 2048);
    assert!(data_file.exists());

    let item = CleanItem {
        id: "browser:google-chrome:Default:cache".to_string(),
        category: CleanCategory::BrowserCache,
        label: "Google Chrome (Default) - Web Cache".to_string(),
        path: Some(cache_dir.to_string_lossy().to_string()),
        size_bytes: 2048,
        requires_root: false,
        supports_trash: false,
        selected: true,
    };

    let report = clean_selected(vec![item], false).await.expect("clean_selected browser cache");
    assert_eq!(report.deleted_bytes, 2048);
    assert_eq!(report.freed_items, vec!["browser:google-chrome:Default:cache".to_string()]);
    assert!(report.errors.is_empty());
    assert!(!cache_dir.exists(), "browser cache directory should be purged");
}

#[tokio::test]
async fn test_clean_selected_purges_trash_item_and_info() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let test_dir = TestDir::new("clean_trash_item");
    let trash_dir = test_dir.path.join("Trash");
    let files_dir = trash_dir.join("files");
    let info_dir = trash_dir.join("info");
    fs::create_dir_all(&files_dir).unwrap();
    fs::create_dir_all(&info_dir).unwrap();

    let trash_file = files_dir.join("delete_me.txt");
    let trash_info = info_dir.join("delete_me.txt.trashinfo");
    create_file(&trash_file, 1024);
    create_file(&trash_info, 128);

    let item = CleanItem {
        id: "trash:delete_me.txt".to_string(),
        category: CleanCategory::UserTrash,
        label: "Trash: delete_me.txt".to_string(),
        path: Some(trash_file.to_string_lossy().to_string()),
        size_bytes: 1024 + 128,
        requires_root: false,
        supports_trash: false,
        selected: true,
    };

    let report = clean_selected(vec![item], false).await.expect("clean_selected trash item");
    assert_eq!(report.deleted_bytes, 1152);
    assert_eq!(report.freed_items, vec!["trash:delete_me.txt".to_string()]);
    assert!(!trash_file.exists(), "trash file should be deleted");
    assert!(!trash_info.exists(), "trash info file should be deleted");
}

#[tokio::test]
async fn test_whitelist_commands_add_and_remove() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let test_dir = TestDir::new("whitelist_cmd");
    let _fake_config = test_dir.path.join("whitelist.json");
    std::env::set_var("XDG_CONFIG_HOME", test_dir.path.to_str().unwrap());

    let initial = get_whitelist().await.expect("get initial whitelist");
    assert!(initial.is_empty() || !initial.contains(&"/tmp/custom_safe_project".to_string()));

    let updated = add_to_whitelist("/tmp/custom_safe_project".to_string())
        .await
        .expect("add to whitelist");
    assert!(updated.contains(&"/tmp/custom_safe_project".to_string()));

    let after_removal = remove_from_whitelist("/tmp/custom_safe_project".to_string())
        .await
        .expect("remove from whitelist");
    assert!(!after_removal.contains(&"/tmp/custom_safe_project".to_string()));
    std::env::remove_var("XDG_CONFIG_HOME");
}

#[tokio::test]
async fn test_clean_selected_updates_cleanup_history() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let test_dir = TestDir::new("clean_history");
    std::env::set_var("XDG_DATA_HOME", test_dir.path.to_str().unwrap());

    let test_file = test_dir.path.join("clean_me.tmp");
    create_file(&test_file, 4096);

    let item = CleanItem {
        id: "temp:clean_me".to_string(),
        category: CleanCategory::StaleCache,
        label: "Temp File".to_string(),
        path: Some(test_file.to_string_lossy().to_string()),
        size_bytes: 4096,
        requires_root: false,
        supports_trash: false,
        selected: true,
    };

    let report = clean_selected(vec![item], false).await.expect("clean selected");
    assert_eq!(report.deleted_bytes, 4096);

    let history = get_cleanup_history().await.expect("get cleanup history");
    assert!(history.total_freed_bytes >= 4096);
    assert!(history.total_clean_runs >= 1);
    std::env::remove_var("XDG_DATA_HOME");
}
