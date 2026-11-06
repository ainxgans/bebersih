use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use bebersih_lib::models::CleanCategory;
use bebersih_lib::scanner::browser::{
    scan_browser_caches, scan_browser_caches_with_config, BrowserScanConfig,
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
        let path = std::env::temp_dir().join(format!("bebersih_browser_test_{}_{}", name, timestamp));
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
    let content = vec![b'c'; size_bytes];
    file.write_all(&content).expect("failed to write content");
}

#[test]
fn test_scan_chrome_caches_and_preserves_profile_configs() {
    let fixture = TestDir::new("chrome");
    let chrome_cache_dir = fixture.path.join(".cache/google-chrome");
    let chrome_config_dir = fixture.path.join(".config/google-chrome");

    // Default profile cache in ~/.cache (should be found)
    create_file(
        &chrome_cache_dir.join("Default/Cache/Cache_Data/data_0"),
        1024 * 50,
    );
    create_file(
        &chrome_cache_dir.join("Default/Code Cache/js/index"),
        1024 * 30,
    );
    create_file(
        &chrome_cache_dir.join("Default/GPUCache/data_0"),
        1024 * 10,
    );

    // Profile 1 cache
    create_file(
        &chrome_cache_dir.join("Profile 1/Cache/Cache_Data/data_0"),
        1024 * 40,
    );

    // Sensitive profile configs in ~/.config (MUST NOT be included in browser cache items!)
    create_file(&chrome_config_dir.join("Default/Cookies"), 1024 * 20);
    create_file(&chrome_config_dir.join("Default/Login Data"), 1024 * 15);

    let config = BrowserScanConfig {
        chrome_dirs: vec![chrome_cache_dir.clone()],
        firefox_dir: None,
        brave_dirs: vec![],
        chromium_dir: None,
    };

    let items = scan_browser_caches_with_config(&config).expect("scan_browser_caches_with_config");

    // Verify all items are BrowserCache category and do not require root
    for item in &items {
        assert_eq!(item.category, CleanCategory::BrowserCache);
        assert!(!item.requires_root);
        assert!(!item.supports_trash);
        assert!(item.selected);
        // Verify path does NOT touch ~/.config or Cookies / Login Data
        if let Some(ref p) = item.path {
            assert!(
                !p.contains(".config"),
                "Browser cache scanner must not touch .config: {}",
                p
            );
            assert!(
                !p.contains("Cookies"),
                "Browser cache scanner must not target Cookies: {}",
                p
            );
        }
    }

    // Check Default profile cache
    let default_cache = items
        .iter()
        .find(|i| i.id.starts_with("browser:google-chrome:Default:cache"))
        .expect("Default cache item found");
    assert_eq!(default_cache.size_bytes, 1024 * 50);
    assert!(default_cache.label.contains("Google Chrome"));
    assert!(default_cache.label.contains("Default"));

    // Check Default Code Cache
    let default_code_cache = items
        .iter()
        .find(|i| i.id.starts_with("browser:google-chrome:Default:code_cache"))
        .expect("Default code cache item found");
    assert_eq!(default_code_cache.size_bytes, 1024 * 30);

    // Check Profile 1 cache
    let p1_cache = items
        .iter()
        .find(|i| i.id.starts_with("browser:google-chrome:Profile 1:cache"))
        .expect("Profile 1 cache item found");
    assert_eq!(p1_cache.size_bytes, 1024 * 40);
}

#[test]
fn test_scan_firefox_caches_and_preserves_logins() {
    let fixture = TestDir::new("firefox");
    let ff_cache_dir = fixture.path.join(".cache/mozilla/firefox");
    let ff_profile_dir = fixture.path.join(".mozilla/firefox/profile1.default");

    // Cache in ~/.cache/mozilla/firefox/profile1.default
    create_file(
        &ff_cache_dir.join("profile1.default/cache2/entries/entry1"),
        1024 * 80,
    );
    create_file(
        &ff_cache_dir.join("profile1.default/startupCache/script.bin"),
        1024 * 25,
    );
    create_file(
        &ff_cache_dir.join("profile1.default/thumbnails/thumb1.png"),
        1024 * 5,
    );

    // Sensitive files in ~/.mozilla (logins, cookies) MUST NOT be returned!
    create_file(&ff_profile_dir.join("cookies.sqlite"), 1024 * 40);
    create_file(&ff_profile_dir.join("logins.json"), 1024 * 10);
    create_file(&ff_profile_dir.join("key4.db"), 1024 * 30);

    let config = BrowserScanConfig {
        chrome_dirs: vec![],
        firefox_dir: Some(ff_cache_dir.clone()),
        brave_dirs: vec![],
        chromium_dir: None,
    };

    let items = scan_browser_caches_with_config(&config).expect("scan firefox caches");

    assert_eq!(items.len(), 3);
    for item in &items {
        assert_eq!(item.category, CleanCategory::BrowserCache);
        if let Some(ref p) = item.path {
            assert!(!p.contains(".mozilla"));
            assert!(!p.contains("cookies.sqlite"));
            assert!(!p.contains("logins.json"));
        }
    }

    let web_cache = items
        .iter()
        .find(|i| i.id.contains("cache2"))
        .expect("Firefox cache2 found");
    assert_eq!(web_cache.size_bytes, 1024 * 80);
    assert!(web_cache.label.contains("Mozilla Firefox"));

    let startup_cache = items
        .iter()
        .find(|i| i.id.contains("startup_cache"))
        .expect("Firefox startupCache found");
    assert_eq!(startup_cache.size_bytes, 1024 * 25);
}

#[test]
fn test_scan_brave_and_chromium() {
    let fixture = TestDir::new("brave_chromium");
    let brave_cache = fixture.path.join(".cache/BraveSoftware/Brave-Browser");
    let chromium_cache = fixture.path.join(".cache/chromium");

    create_file(
        &brave_cache.join("Default/Cache/data_0"),
        1024 * 60,
    );
    create_file(
        &chromium_cache.join("Default/Cache/data_0"),
        1024 * 70,
    );

    let config = BrowserScanConfig {
        chrome_dirs: vec![],
        firefox_dir: None,
        brave_dirs: vec![brave_cache],
        chromium_dir: Some(chromium_cache),
    };

    let items = scan_browser_caches_with_config(&config).expect("scan brave & chromium");

    assert_eq!(items.len(), 2);
    let brave_item = items
        .iter()
        .find(|i| i.id.starts_with("browser:brave:Default:cache"))
        .expect("brave item found");
    assert_eq!(brave_item.size_bytes, 1024 * 60);
    assert!(brave_item.label.contains("Brave"));

    let chromium_item = items
        .iter()
        .find(|i| i.id.starts_with("browser:chromium:Default:cache"))
        .expect("chromium item found");
    assert_eq!(chromium_item.size_bytes, 1024 * 70);
    assert!(chromium_item.label.contains("Chromium"));
}

#[test]
fn test_empty_or_nonexistent_dirs_handled_gracefully() {
    let nonexistent = PathBuf::from("/tmp/bebersih_nonexistent_browser_dir_xyz123");
    let config = BrowserScanConfig {
        chrome_dirs: vec![nonexistent.clone()],
        firefox_dir: Some(nonexistent.clone()),
        brave_dirs: vec![nonexistent.clone()],
        chromium_dir: Some(nonexistent),
    };

    let items = scan_browser_caches_with_config(&config).expect("handles nonexistent dirs");
    assert!(items.is_empty());
}

#[test]
fn test_scan_browser_caches_default_does_not_panic() {
    let result = scan_browser_caches();
    assert!(result.is_ok());
}
