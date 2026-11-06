use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use bebersih_lib::models::CleanCategory;
use bebersih_lib::scanner::ide::{scan_ide_caches_with_config, IdeScanConfig};

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("bebersih_ide_test_{}_{}", name, std::process::id()));
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
    let data = vec![b'c'; size_bytes];
    file.write_all(&data).unwrap();
}

#[test]
fn test_scan_vscode_and_cursor_caches() {
    let test_dir = TestDir::new("vscode");
    let config_dir = test_dir.path.join(".config");
    let cache_dir = test_dir.path.join(".cache");

    // VS Code caches
    create_file(&config_dir.join("Code/Cache/index"), 15000);
    create_file(&config_dir.join("Code/CachedData/bundle.js"), 25000);
    create_file(&config_dir.join("Code/Service Worker/CacheStorage/storage.bin"), 10000);

    // Cursor caches
    create_file(&config_dir.join("Cursor/Cache/cache.data"), 12000);

    let config = IdeScanConfig {
        config_dir: Some(config_dir),
        cache_dir: Some(cache_dir),
    };

    let items = scan_ide_caches_with_config(&config).expect("scan ide caches");
    assert!(!items.is_empty());

    let vscode_cache = items.iter().find(|i| i.id.contains("code:cache")).expect("code cache found");
    assert_eq!(vscode_cache.category, CleanCategory::IdeCache);
    assert_eq!(vscode_cache.size_bytes, 15000);
    assert_eq!(vscode_cache.requires_root, false);

    let vscode_cached_data = items.iter().find(|i| i.id.contains("code:cached-data")).expect("code cached data found");
    assert_eq!(vscode_cached_data.category, CleanCategory::IdeCache);
    assert_eq!(vscode_cached_data.size_bytes, 25000);

    let cursor_cache = items.iter().find(|i| i.id.contains("cursor:cache")).expect("cursor cache found");
    assert_eq!(cursor_cache.category, CleanCategory::IdeCache);
    assert_eq!(cursor_cache.size_bytes, 12000);
}

#[test]
fn test_scan_jetbrains_ide_caches() {
    let test_dir = TestDir::new("jetbrains");
    let config_dir = test_dir.path.join(".config");
    let cache_dir = test_dir.path.join(".cache");

    // IntelliJ IDEA & PyCharm caches
    let intellij_cache = cache_dir.join("JetBrains/IntelliJIdea2024.1/caches");
    let pycharm_cache = cache_dir.join("JetBrains/PyCharm2024.1/caches");
    create_file(&intellij_cache.join("indices.db"), 50000);
    create_file(&pycharm_cache.join("types.bin"), 30000);

    let config = IdeScanConfig {
        config_dir: Some(config_dir),
        cache_dir: Some(cache_dir),
    };

    let items = scan_ide_caches_with_config(&config).expect("scan jetbrains caches");
    assert_eq!(items.len(), 2);

    let idea_item = items.iter().find(|i| i.label.contains("IntelliJIdea")).expect("idea item");
    assert_eq!(idea_item.category, CleanCategory::IdeCache);
    assert_eq!(idea_item.size_bytes, 50000);

    let pycharm_item = items.iter().find(|i| i.label.contains("PyCharm")).expect("pycharm item");
    assert_eq!(pycharm_item.category, CleanCategory::IdeCache);
    assert_eq!(pycharm_item.size_bytes, 30000);
}
