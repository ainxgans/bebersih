use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use bebersih_lib::models::{CleanCategory, CleanItem};
use bebersih_lib::scanner::dev::scan_project_dev_caches_with_config;
use bebersih_lib::whitelist::{
    is_path_ignored_or_whitelisted, load_whitelist_from_file, save_whitelist_to_file,
    WhitelistConfig,
};

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("bebersih_wl_test_{}_{}", name, std::process::id()));
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
fn test_bebersihignore_skips_project() {
    let test_dir = TestDir::new("bebersihignore");
    let proj_ignored = test_dir.path.join("proj_ignored");
    let proj_active = test_dir.path.join("proj_active");

    // Ignored project with .bebersihignore
    create_file(&proj_ignored.join("Cargo.toml"), 50);
    create_file(&proj_ignored.join(".bebersihignore"), 1);
    create_file(&proj_ignored.join("target/debug/app"), 1024);

    // Active project
    create_file(&proj_active.join("Cargo.toml"), 50);
    create_file(&proj_active.join("target/debug/app"), 2048);

    let config = WhitelistConfig::default();
    let items = scan_project_dev_caches_with_config(&[test_dir.path.clone()], &config)
        .expect("scan dev caches with config");

    let has_ignored = items.iter().any(|i| i.path.as_ref().map(|p| p.contains("proj_ignored")).unwrap_or(false));
    let has_active = items.iter().any(|i| i.path.as_ref().map(|p| p.contains("proj_active")).unwrap_or(false));

    assert!(!has_ignored, "project with .bebersihignore must be skipped");
    assert!(has_active, "normal project target must be detected");
}

#[test]
fn test_explicit_whitelist_paths_excluded() {
    let test_dir = TestDir::new("explicit_whitelist");
    let proj_whitelisted = test_dir.path.join("proj_whitelisted");
    let proj_normal = test_dir.path.join("proj_normal");

    create_file(&proj_whitelisted.join("package.json"), 50);
    create_file(&proj_whitelisted.join("node_modules/pkg/index.js"), 1024);

    create_file(&proj_normal.join("package.json"), 50);
    create_file(&proj_normal.join("node_modules/pkg/index.js"), 2048);

    let mut config = WhitelistConfig::default();
    config.whitelisted_paths.push(proj_whitelisted.to_string_lossy().to_string());

    let items = scan_project_dev_caches_with_config(&[test_dir.path.clone()], &config)
        .expect("scan dev caches with whitelist");

    let has_whitelisted = items.iter().any(|i| i.path.as_ref().map(|p| p.contains("proj_whitelisted")).unwrap_or(false));
    let has_normal = items.iter().any(|i| i.path.as_ref().map(|p| p.contains("proj_normal")).unwrap_or(false));

    assert!(!has_whitelisted, "explicitly whitelisted path must be excluded");
    assert!(has_normal, "non-whitelisted path must be detected");
}

#[test]
fn test_whitelist_persistence_roundtrip() {
    let test_dir = TestDir::new("persist");
    let config_file = test_dir.path.join("whitelist.json");

    let mut config = WhitelistConfig::default();
    config.whitelisted_paths.push("/home/user/important_project".to_string());
    config.whitelisted_paths.push("/home/user/work".to_string());

    save_whitelist_to_file(&config, &config_file).expect("save whitelist");
    assert!(config_file.exists());

    let loaded = load_whitelist_from_file(&config_file).expect("load whitelist");
    assert_eq!(loaded.whitelisted_paths.len(), 2);
    assert_eq!(loaded.whitelisted_paths[0], "/home/user/important_project");
}

#[test]
fn test_cleaner_refuses_to_clean_whitelisted_items() {
    let test_dir = TestDir::new("clean_whitelist");
    let target_dir = test_dir.path.join("protected_project/target");
    create_file(&target_dir.join("app"), 1024);

    let item = CleanItem {
        id: "dev:rust:protected".to_string(),
        category: CleanCategory::DevRust,
        label: "Protected target".to_string(),
        path: Some(target_dir.to_string_lossy().to_string()),
        size_bytes: 1024,
        requires_root: false,
        supports_trash: false,
        selected: true,
    };

    let mut config = WhitelistConfig::default();
    config.whitelisted_paths.push(test_dir.path.join("protected_project").to_string_lossy().to_string());

    assert!(is_path_ignored_or_whitelisted(&target_dir, &config));

    // When cleaned with whitelist guard
    let report = bebersih_lib::cleaner::clean_items_with_whitelist(&[item], false, &config)
        .expect("clean with whitelist");

    assert_eq!(report.deleted_bytes, 0);
    assert!(report.freed_items.is_empty());
    assert!(!report.errors.is_empty(), "error should be reported for whitelisted item");
    assert!(target_dir.exists(), "whitelisted path must not be deleted");
}
