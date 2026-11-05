use std::fs;
use std::path::{Path, PathBuf};

use bebersih_lib::cleaner::{clean_items, generate_root_batch_script, validate_safe_path};
use bebersih_lib::models::{CleanCategory, CleanItem};

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let temp = std::env::temp_dir().join(format!("bebersih_cleaner_test_{}_{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        Self { path: temp }
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn test_validate_safe_path_rejects_dangerous_paths() {
    // Empty path
    assert!(validate_safe_path(Path::new("")).is_err());

    // Relative path
    assert!(validate_safe_path(Path::new("relative/path")).is_err());
    assert!(validate_safe_path(Path::new(".")).is_err());
    assert!(validate_safe_path(Path::new("..")).is_err());

    // Root filesystem
    assert!(validate_safe_path(Path::new("/")).is_err());

    // System critical roots
    assert!(validate_safe_path(Path::new("/bin")).is_err());
    assert!(validate_safe_path(Path::new("/boot")).is_err());
    assert!(validate_safe_path(Path::new("/dev")).is_err());
    assert!(validate_safe_path(Path::new("/etc")).is_err());
    assert!(validate_safe_path(Path::new("/home")).is_err());
    assert!(validate_safe_path(Path::new("/lib")).is_err());
    assert!(validate_safe_path(Path::new("/proc")).is_err());
    assert!(validate_safe_path(Path::new("/root")).is_err());
    assert!(validate_safe_path(Path::new("/sys")).is_err());
    assert!(validate_safe_path(Path::new("/usr")).is_err());
    assert!(validate_safe_path(Path::new("/var")).is_err());

    // Home directory itself
    if let Ok(home) = std::env::var("HOME") {
        assert!(validate_safe_path(Path::new(&home)).is_err());
    }

    // Safe subdirectories should be accepted
    let temp = std::env::temp_dir().join("some_project/target");
    assert!(validate_safe_path(&temp).is_ok());
}

#[test]
fn test_purge_deletes_directory_and_file() {
    let test_dir = TestDir::new("purge");
    let sub_dir = test_dir.path.join("target");
    fs::create_dir_all(&sub_dir).unwrap();
    let file = sub_dir.join("build.log");
    fs::write(&file, "sample data").unwrap();

    assert!(sub_dir.exists());

    let item = CleanItem {
        id: "test:target".to_string(),
        category: CleanCategory::DevRust,
        label: "Rust target".to_string(),
        path: Some(sub_dir.to_str().unwrap().to_string()),
        size_bytes: 1024,
        requires_root: false,
        supports_trash: true,
        selected: true,
    };

    // Clean with move_project_to_trash = false (direct purge)
    let report = clean_items(&[item], false).expect("clean_items should succeed");
    assert_eq!(report.freed_items.len(), 1);
    assert_eq!(report.deleted_bytes, 1024);
    assert!(report.errors.is_empty());
    assert!(!sub_dir.exists());
}

#[test]
fn test_generate_root_batch_script() {
    let apt_item = CleanItem {
        id: "apt:archives".to_string(),
        category: CleanCategory::AptCache,
        label: "APT package archives".to_string(),
        path: Some("/var/cache/apt/archives".to_string()),
        size_bytes: 2048,
        requires_root: true,
        supports_trash: false,
        selected: true,
    };

    let journal_item = CleanItem {
        id: "system:journal".to_string(),
        category: CleanCategory::SystemLogs,
        label: "Systemd journal logs".to_string(),
        path: None,
        size_bytes: 4096,
        requires_root: true,
        supports_trash: false,
        selected: true,
    };

    let script = generate_root_batch_script(&[apt_item, journal_item]);
    assert!(script.contains("apt-get clean"));
    assert!(script.contains("journalctl --vacuum-time=7d"));
}

#[test]
fn test_clean_items_with_simulated_root_runner() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use bebersih_lib::cleaner::clean_items_with_root_runner;

    let items = vec![
        CleanItem {
            id: "system:apt:/var/cache/apt/archives".to_string(),
            category: CleanCategory::AptCache,
            label: "APT package archives".to_string(),
            path: Some("/var/cache/apt/archives".to_string()),
            size_bytes: 1000,
            requires_root: true,
            supports_trash: false,
            selected: true,
        },
        CleanItem {
            id: "system:journalctl".to_string(),
            category: CleanCategory::SystemLogs,
            label: "Systemd journal logs".to_string(),
            path: None,
            size_bytes: 2000,
            requires_root: true,
            supports_trash: false,
            selected: true,
        },
    ];

    let executed = Arc::new(AtomicBool::new(false));
    let executed_clone = Arc::clone(&executed);

    let report = clean_items_with_root_runner(&items, false, move |script| {
        assert!(script.contains("apt-get clean"));
        assert!(script.contains("journalctl --vacuum-time="));
        executed_clone.store(true, Ordering::SeqCst);
        Ok(())
    })
    .expect("clean_items_with_root_runner should succeed");

    assert!(executed.load(Ordering::SeqCst));
    assert_eq!(report.deleted_bytes, 3000);
    assert_eq!(report.freed_items.len(), 2);
    assert!(report.errors.is_empty());
}

#[test]
fn test_clean_items_skips_unselected_items() {
    let test_dir = TestDir::new("unselected");
    let file = test_dir.path.join("keep.txt");
    fs::write(&file, "do not touch").unwrap();

    let item = CleanItem {
        id: "keep:file".to_string(),
        category: CleanCategory::DevRust,
        label: "Keep file".to_string(),
        path: Some(file.to_str().unwrap().to_string()),
        size_bytes: 500,
        requires_root: false,
        supports_trash: false,
        selected: false,
    };

    let report = clean_items(&[item], false).expect("clean_items should succeed");
    assert_eq!(report.deleted_bytes, 0);
    assert!(report.freed_items.is_empty());
    assert!(file.exists());
}

#[test]
fn test_clean_items_rejects_unsafe_path() {
    let item = CleanItem {
        id: "bad:path".to_string(),
        category: CleanCategory::SystemLogs,
        label: "Bad path".to_string(),
        path: Some("/etc".to_string()),
        size_bytes: 999,
        requires_root: false,
        supports_trash: false,
        selected: true,
    };

    let report = clean_items(&[item], false).expect("clean_items should succeed with error logged");
    assert_eq!(report.deleted_bytes, 0);
    assert!(report.freed_items.is_empty());
    assert_eq!(report.errors.len(), 1);
    assert!(report.errors[0].contains("bad:path"));
}
