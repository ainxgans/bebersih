use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use bebersih_lib::models::CleanCategory;
use bebersih_lib::scanner::dev::{scan_global_python_caches, scan_project_dev_caches_with_config};
use bebersih_lib::whitelist::WhitelistConfig;

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("bebersih_py_test_{}_{}", name, std::process::id()));
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
    let data = vec![b'p'; size_bytes];
    file.write_all(&data).unwrap();
}

#[test]
fn test_scan_python_project_virtualenv_and_pycache() {
    let test_dir = TestDir::new("py_proj");
    let proj_dir = test_dir.path.join("my_python_service");

    create_file(&proj_dir.join("pyproject.toml"), 50);
    create_file(&proj_dir.join(".venv/bin/python"), 5000);
    create_file(&proj_dir.join("src/__pycache__/module.cpython-312.pyc"), 1200);

    let config = WhitelistConfig::default();
    let items = scan_project_dev_caches_with_config(&[test_dir.path.clone()], &config)
        .expect("scan python project dev caches");

    let venv_item = items
        .iter()
        .find(|i| i.path.as_ref().map(|p| p.ends_with(".venv")).unwrap_or(false))
        .expect("venv item detected");

    assert_eq!(venv_item.category, CleanCategory::DevPython);
    assert!(venv_item.size_bytes >= 5000);
    assert_eq!(venv_item.supports_trash, true);

    let pycache_item = items
        .iter()
        .find(|i| i.path.as_ref().map(|p| p.ends_with("__pycache__")).unwrap_or(false))
        .expect("pycache item detected");

    assert_eq!(pycache_item.category, CleanCategory::DevPython);
    assert!(pycache_item.size_bytes >= 1200);
}

#[test]
fn test_scan_global_python_caches() {
    let test_dir = TestDir::new("py_global");
    let pip_dir = test_dir.path.join(".cache/pip");
    let uv_dir = test_dir.path.join(".cache/uv");
    create_file(&pip_dir.join("wheels/pkg.whl"), 2048);
    create_file(&uv_dir.join("cache.bin"), 4096);

    let items = scan_global_python_caches(Some(&test_dir.path)).expect("scan global python caches");
    assert_eq!(items.len(), 2);

    let pip_item = items.iter().find(|i| i.id.contains("pip")).expect("pip item found");
    assert_eq!(pip_item.category, CleanCategory::DevPython);
    assert_eq!(pip_item.size_bytes, 2048);

    let uv_item = items.iter().find(|i| i.id.contains("uv")).expect("uv item found");
    assert_eq!(uv_item.category, CleanCategory::DevPython);
    assert_eq!(uv_item.size_bytes, 4096);
}
