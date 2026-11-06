use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use bebersih_lib::models::CleanCategory;
use bebersih_lib::scanner::dev::{scan_global_jvm_caches, scan_project_dev_caches_with_config};
use bebersih_lib::whitelist::WhitelistConfig;

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("bebersih_jvm_test_{}_{}", name, std::process::id()));
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
    let data = vec![b'j'; size_bytes];
    file.write_all(&data).unwrap();
}

#[test]
fn test_scan_jvm_project_caches() {
    let test_dir = TestDir::new("jvm_proj");
    
    // 1. Gradle project with build/ directory
    let gradle_proj = test_dir.path.join("my_gradle_app");
    create_file(&gradle_proj.join("build.gradle"), 100);
    create_file(&gradle_proj.join("build/libs/app.jar"), 4000);

    // 2. Maven project with target/ directory
    let maven_proj = test_dir.path.join("my_maven_service");
    create_file(&maven_proj.join("pom.xml"), 100);
    create_file(&maven_proj.join("target/classes/Main.class"), 2500);

    let config = WhitelistConfig::default();
    let items = scan_project_dev_caches_with_config(&[test_dir.path.clone()], &config)
        .expect("scan jvm project dev caches");

    let gradle_item = items
        .iter()
        .find(|i| i.path.as_ref().map(|p| p.ends_with("my_gradle_app/build")).unwrap_or(false))
        .expect("gradle build item detected");
    assert_eq!(gradle_item.category, CleanCategory::DevJvm);
    assert!(gradle_item.size_bytes >= 4000);
    assert_eq!(gradle_item.supports_trash, true);

    let maven_item = items
        .iter()
        .find(|i| i.path.as_ref().map(|p| p.ends_with("my_maven_service/target")).unwrap_or(false))
        .expect("maven target item detected");
    assert_eq!(maven_item.category, CleanCategory::DevJvm);
    assert!(maven_item.size_bytes >= 2500);
    assert_eq!(maven_item.supports_trash, true);
}

#[test]
fn test_scan_global_jvm_caches() {
    let test_dir = TestDir::new("jvm_global");
    let gradle_caches = test_dir.path.join(".gradle/caches");
    let gradle_dists = test_dir.path.join(".gradle/wrapper/dists");
    let m2_repo = test_dir.path.join(".m2/repository");

    create_file(&gradle_caches.join("modules-2/files.bin"), 10000);
    create_file(&gradle_dists.join("gradle-8.5-bin/xyz/gradle.zip"), 50000);
    create_file(&m2_repo.join("org/springframework/spring.jar"), 20000);

    let items = scan_global_jvm_caches(Some(&test_dir.path)).expect("scan global jvm caches");
    assert_eq!(items.len(), 3);

    let caches_item = items.iter().find(|i| i.id.contains("gradle-caches")).expect("gradle caches item");
    assert_eq!(caches_item.category, CleanCategory::DevJvm);
    assert_eq!(caches_item.size_bytes, 10000);

    let dists_item = items.iter().find(|i| i.id.contains("gradle-dists")).expect("gradle dists item");
    assert_eq!(dists_item.category, CleanCategory::DevJvm);
    assert_eq!(dists_item.size_bytes, 50000);

    let m2_item = items.iter().find(|i| i.id.contains("m2-repo")).expect("m2 repo item");
    assert_eq!(m2_item.category, CleanCategory::DevJvm);
    assert_eq!(m2_item.size_bytes, 20000);
}
