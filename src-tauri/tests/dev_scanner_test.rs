use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bebersih_lib::models::CleanCategory;
use bebersih_lib::scanner::scan_dev_caches;

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

fn create_file(path: &PathBuf, size_bytes: usize) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent dir");
    }
    let mut file = File::create(path).expect("failed to create file");
    let content = vec![b'a'; size_bytes];
    file.write_all(&content).expect("failed to write content");
}

#[test]
fn test_scan_dev_caches_detects_rust_and_node() {
    let test_dir = TestDir::new("dev_scan");
    let base = &test_dir.path;

    // 1. Valid Rust project: Cargo.toml + target/
    let rust_dir = base.join("rust_proj");
    fs::create_dir_all(&rust_dir).unwrap();
    fs::write(rust_dir.join("Cargo.toml"), "[package]\nname = \"dummy\"").unwrap();
    let rust_target = rust_dir.join("target");
    create_file(&rust_target.join("debug").join("app"), 2048);

    // 2. Valid Node project: package.json + node_modules/
    let node_dir = base.join("node_proj");
    fs::create_dir_all(&node_dir).unwrap();
    fs::write(node_dir.join("package.json"), "{\"name\": \"dummy\"}").unwrap();
    let node_modules = node_dir.join("node_modules");
    create_file(&node_modules.join("pkg").join("index.js"), 4096);

    // 3. Ignored directory: target/ without Cargo.toml
    let orphan_target = base.join("orphan_rust").join("target");
    create_file(&orphan_target.join("file.bin"), 1024);

    // 4. Ignored directory: node_modules/ without package.json
    let orphan_node = base.join("orphan_node").join("node_modules");
    create_file(&orphan_node.join("file.bin"), 1024);

    let results = scan_dev_caches(std::slice::from_ref(base)).expect("scan_dev_caches should succeed");

    // Verify Rust target detection
    let rust_item = results
        .iter()
        .find(|item| item.path.as_deref() == rust_target.to_str())
        .expect("should find valid rust target dir");
    assert_eq!(rust_item.category, CleanCategory::DevRust);
    assert!(rust_item.size_bytes >= 2048);
    assert!(rust_item.supports_trash);
    assert!(!rust_item.requires_root);
    assert!(rust_item.selected);

    // Verify Node node_modules detection
    let node_item = results
        .iter()
        .find(|item| item.path.as_deref() == node_modules.to_str())
        .expect("should find valid node_modules dir");
    assert_eq!(node_item.category, CleanCategory::DevNode);
    assert!(node_item.size_bytes >= 4096);
    assert!(node_item.supports_trash);
    assert!(!node_item.requires_root);
    assert!(node_item.selected);

    // Verify non-adjacent target/node_modules are ignored
    assert!(
        results
            .iter()
            .all(|item| item.path.as_deref() != orphan_target.to_str()),
        "orphan target without Cargo.toml must not be detected"
    );
    assert!(
        results
            .iter()
            .all(|item| item.path.as_deref() != orphan_node.to_str()),
        "orphan node_modules without package.json must not be detected"
    );
}

#[test]
fn test_scan_dev_caches_empty_roots_handles_globals() {
    let results = scan_dev_caches(&[]).expect("scan_dev_caches with empty roots should succeed");
    for item in &results {
        assert!(
            item.category == CleanCategory::DevRust || item.category == CleanCategory::DevGo,
            "global cache items must be DevRust or DevGo"
        );
        assert!(
            !item.supports_trash,
            "global caches should not support trash (direct purge)"
        );
        assert!(!item.requires_root);
        assert!(item.path.is_some());
    }
}
