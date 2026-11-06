use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use crate::error::AppError;
use crate::models::{CleanCategory, CleanItem};
use crate::whitelist::{is_path_ignored_or_whitelisted, WhitelistConfig};

// ponytail: std::env::var("HOME") is enough for Linux; add dirs crate if porting to Windows.
fn get_home_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

fn get_go_env(var: &str) -> Option<PathBuf> {
    let output = Command::new("go").args(["env", var]).output().ok()?;
    if output.status.success() {
        let text = String::from_utf8(output.stdout).ok()?;
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
    }
    None
}

pub fn calculate_dir_size(path: &Path) -> u64 {
    jwalk::WalkDir::new(path)
        .skip_hidden(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

pub fn scan_project_dev_caches(roots: &[PathBuf]) -> Result<Vec<CleanItem>, AppError> {
    let config = WhitelistConfig::load_default();
    scan_project_dev_caches_with_config(roots, &config)
}

pub fn scan_project_dev_caches_with_config(
    roots: &[PathBuf],
    config: &WhitelistConfig,
) -> Result<Vec<CleanItem>, AppError> {
    let mut items = Vec::new();

    for root in roots {
        if !root.exists() {
            continue;
        }

        let matched = Arc::new(Mutex::new(Vec::new()));
        let matched_clone = Arc::clone(&matched);
        let config_clone = Arc::new(config.clone());

        for _ in jwalk::WalkDir::new(root)
            .skip_hidden(false)
            .process_read_dir(move |_depth, dir_path, _state, children| {
                if is_path_ignored_or_whitelisted(dir_path, &config_clone) {
                    for entry in children.iter_mut().flatten() {
                        entry.read_children_path = None;
                    }
                    return;
                }

                let mut has_cargo_toml = false;
                let mut has_package_json = false;
                let mut has_python_project = false;
                let mut has_gradle_project = false;
                let mut has_maven_project = false;
                let mut has_bebersihignore = false;

                for entry in children.iter().flatten() {
                    let name = entry.file_name();
                    if name == ".bebersihignore" {
                        has_bebersihignore = true;
                    } else if name == "Cargo.toml" && entry.file_type().is_file() {
                        has_cargo_toml = true;
                    } else if name == "package.json" && entry.file_type().is_file() {
                        has_package_json = true;
                    } else if (name == "build.gradle"
                        || name == "build.gradle.kts"
                        || name == "settings.gradle"
                        || name == "settings.gradle.kts")
                        && entry.file_type().is_file()
                    {
                        has_gradle_project = true;
                    } else if name == "pom.xml" && entry.file_type().is_file() {
                        has_maven_project = true;
                    } else if (name == "pyproject.toml"
                        || name == "requirements.txt"
                        || name == "Pipfile"
                        || name == "setup.py"
                        || name == "poetry.lock"
                        || name == "uv.lock")
                        && entry.file_type().is_file()
                    {
                        has_python_project = true;
                    }
                }

                if has_bebersihignore {
                    for entry in children.iter_mut().flatten() {
                        entry.read_children_path = None;
                    }
                    return;
                }

                for entry in children.iter_mut().flatten() {
                    if !entry.file_type().is_dir() {
                        continue;
                    }
                    let name = entry.file_name();
                    if name == "target" {
                        entry.read_children_path = None;
                        if has_cargo_toml && !is_path_ignored_or_whitelisted(&entry.path(), &config_clone) {
                            matched_clone.lock().unwrap().push((
                                CleanCategory::DevRust,
                                entry.path(),
                                "Rust target".to_string(),
                            ));
                        } else if has_maven_project && !is_path_ignored_or_whitelisted(&entry.path(), &config_clone) {
                            matched_clone.lock().unwrap().push((
                                CleanCategory::DevJvm,
                                entry.path(),
                                "Maven target".to_string(),
                            ));
                        }
                    } else if name == "build" {
                        if has_gradle_project && !is_path_ignored_or_whitelisted(&entry.path(), &config_clone) {
                            entry.read_children_path = None;
                            matched_clone.lock().unwrap().push((
                                CleanCategory::DevJvm,
                                entry.path(),
                                "Gradle build output".to_string(),
                            ));
                        }
                    } else if name == "node_modules" {
                        entry.read_children_path = None;
                        if has_package_json && !is_path_ignored_or_whitelisted(&entry.path(), &config_clone) {
                            matched_clone.lock().unwrap().push((
                                CleanCategory::DevNode,
                                entry.path(),
                                "Node modules".to_string(),
                            ));
                        }
                    } else if name == ".gradle" {
                        entry.read_children_path = None;
                        if !is_path_ignored_or_whitelisted(&entry.path(), &config_clone) {
                            matched_clone.lock().unwrap().push((
                                CleanCategory::DevJvm,
                                entry.path(),
                                "Gradle project cache".to_string(),
                            ));
                        }
                    } else if name == ".venv" || name == "venv" {
                        entry.read_children_path = None;
                        if (has_python_project
                            || entry.path().join("pyvenv.cfg").exists()
                            || entry.path().join("bin/python").exists())
                            && !is_path_ignored_or_whitelisted(&entry.path(), &config_clone)
                        {
                            matched_clone.lock().unwrap().push((
                                CleanCategory::DevPython,
                                entry.path(),
                                "Python virtualenv".to_string(),
                            ));
                        }
                    } else if name == "__pycache__" {
                        entry.read_children_path = None;
                        if !is_path_ignored_or_whitelisted(&entry.path(), &config_clone) {
                            matched_clone.lock().unwrap().push((
                                CleanCategory::DevPython,
                                entry.path(),
                                "Python bytecode cache".to_string(),
                            ));
                        }
                    } else if name == ".pytest_cache" || name == ".mypy_cache" || name == ".ruff_cache" {
                        entry.read_children_path = None;
                        if !is_path_ignored_or_whitelisted(&entry.path(), &config_clone) {
                            matched_clone.lock().unwrap().push((
                                CleanCategory::DevPython,
                                entry.path(),
                                "Python tool cache".to_string(),
                            ));
                        }
                    } else if name == ".git"
                        || name == ".cache"
                        || name == ".cargo"
                        || name == ".local"
                        || name == ".rustup"
                    {
                        entry.read_children_path = None;
                    }
                }
            })
        {
            // drive iterator
        }

        let matched_items = std::mem::take(&mut *matched.lock().map_err(|e| AppError::Other(e.to_string()))?);

        for (category, path, label_prefix) in matched_items {
            let project_name = path
                .parent()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "project".to_string());

            let category_str = match category {
                CleanCategory::DevRust => "rust",
                CleanCategory::DevNode => "node",
                CleanCategory::DevGo => "go",
                CleanCategory::DevPython => "python",
                CleanCategory::DevJvm => "jvm",
                _ => "dev",
            };

            let size_bytes = calculate_dir_size(&path);
            items.push(CleanItem {
                id: format!("dev:{}:{}", category_str, path.display()),
                category,
                label: format!("{} ({})", label_prefix, project_name),
                path: Some(path.to_string_lossy().to_string()),
                size_bytes,
                requires_root: false,
                supports_trash: true,
                selected: true,
            });
        }
    }

    Ok(items)
}

pub fn scan_global_dev_caches() -> Result<Vec<CleanItem>, AppError> {
    let mut items = Vec::new();
    let home = get_home_dir();

    // 1. Go build cache
    let go_build_cache = get_go_env("GOCACHE").or_else(|| home.as_ref().map(|h| h.join(".cache/go-build")));
    if let Some(path) = go_build_cache {
        if path.exists() && path.is_dir() {
            let size_bytes = calculate_dir_size(&path);
            items.push(CleanItem {
                id: format!("dev:go:{}", path.display()),
                category: CleanCategory::DevGo,
                label: "Go build cache".to_string(),
                path: Some(path.to_string_lossy().to_string()),
                size_bytes,
                requires_root: false,
                supports_trash: false,
                selected: true,
            });
        }
    }

    // 2. Go module cache
    let go_mod_cache = get_go_env("GOMODCACHE")
        .map(|p| if p.join("cache").exists() { p.join("cache") } else { p })
        .or_else(|| {
            home.as_ref().map(|h| {
                let cache_dir = h.join("go/pkg/mod/cache");
                if cache_dir.exists() {
                    cache_dir
                } else {
                    h.join("go/pkg/mod")
                }
            })
        });

    if let Some(path) = go_mod_cache {
        if path.exists() && path.is_dir() {
            let size_bytes = calculate_dir_size(&path);
            items.push(CleanItem {
                id: format!("dev:go:{}", path.display()),
                category: CleanCategory::DevGo,
                label: "Go module cache".to_string(),
                path: Some(path.to_string_lossy().to_string()),
                size_bytes,
                requires_root: false,
                supports_trash: false,
                selected: true,
            });
        }
    }

    // 3. Cargo global registry cache
    let cargo_cache = std::env::var("CARGO_HOME")
        .ok()
        .map(|c| PathBuf::from(c).join("registry/cache"))
        .or_else(|| home.as_ref().map(|h| h.join(".cargo/registry/cache")));

    if let Some(path) = cargo_cache {
        if path.exists() && path.is_dir() {
            let size_bytes = calculate_dir_size(&path);
            items.push(CleanItem {
                id: format!("dev:cargo:{}", path.display()),
                category: CleanCategory::DevRust,
                label: "Cargo registry cache".to_string(),
                path: Some(path.to_string_lossy().to_string()),
                size_bytes,
                requires_root: false,
                supports_trash: false,
                selected: true,
            });
        }
    }

    // 4. Python global caches (pip, uv, poetry)
    items.extend(scan_global_python_caches(None)?);

    // 5. JVM global caches (Gradle caches, wrapper dists, Maven repo)
    items.extend(scan_global_jvm_caches(None)?);

    Ok(items)
}

pub fn scan_global_jvm_caches(home_override: Option<&Path>) -> Result<Vec<CleanItem>, AppError> {
    let mut items = Vec::new();
    let home = home_override.map(PathBuf::from).or_else(get_home_dir);

    if let Some(h) = home {
        // 1. Gradle caches
        let gradle_caches = h.join(".gradle/caches");
        if gradle_caches.exists() && gradle_caches.is_dir() {
            let size_bytes = calculate_dir_size(&gradle_caches);
            items.push(CleanItem {
                id: format!("dev:jvm:gradle-caches:{}", gradle_caches.display()),
                category: CleanCategory::DevJvm,
                label: "Gradle dependency & artifact caches".to_string(),
                path: Some(gradle_caches.to_string_lossy().to_string()),
                size_bytes,
                requires_root: false,
                supports_trash: false,
                selected: true,
            });
        }

        // 2. Gradle wrapper dists
        let gradle_dists = h.join(".gradle/wrapper/dists");
        if gradle_dists.exists() && gradle_dists.is_dir() {
            let size_bytes = calculate_dir_size(&gradle_dists);
            items.push(CleanItem {
                id: format!("dev:jvm:gradle-dists:{}", gradle_dists.display()),
                category: CleanCategory::DevJvm,
                label: "Gradle wrapper distributions".to_string(),
                path: Some(gradle_dists.to_string_lossy().to_string()),
                size_bytes,
                requires_root: false,
                supports_trash: false,
                selected: true,
            });
        }

        // 3. Maven repository
        let m2_repo = h.join(".m2/repository");
        if m2_repo.exists() && m2_repo.is_dir() {
            let size_bytes = calculate_dir_size(&m2_repo);
            items.push(CleanItem {
                id: format!("dev:jvm:m2-repo:{}", m2_repo.display()),
                category: CleanCategory::DevJvm,
                label: "Maven local repository dependencies".to_string(),
                path: Some(m2_repo.to_string_lossy().to_string()),
                size_bytes,
                requires_root: false,
                supports_trash: false,
                selected: true,
            });
        }
    }

    Ok(items)
}

pub fn scan_global_python_caches(home_override: Option<&Path>) -> Result<Vec<CleanItem>, AppError> {
    let mut items = Vec::new();
    let home = home_override.map(PathBuf::from).or_else(get_home_dir);

    if let Some(h) = home {
        // pip cache
        let pip_cache = h.join(".cache/pip");
        if pip_cache.exists() && pip_cache.is_dir() {
            let size_bytes = calculate_dir_size(&pip_cache);
            items.push(CleanItem {
                id: format!("dev:python:pip:{}", pip_cache.display()),
                category: CleanCategory::DevPython,
                label: "Pip wheel & download cache".to_string(),
                path: Some(pip_cache.to_string_lossy().to_string()),
                size_bytes,
                requires_root: false,
                supports_trash: false,
                selected: true,
            });
        }

        // uv cache
        let uv_cache = h.join(".cache/uv");
        if uv_cache.exists() && uv_cache.is_dir() {
            let size_bytes = calculate_dir_size(&uv_cache);
            items.push(CleanItem {
                id: format!("dev:python:uv:{}", uv_cache.display()),
                category: CleanCategory::DevPython,
                label: "uv package cache".to_string(),
                path: Some(uv_cache.to_string_lossy().to_string()),
                size_bytes,
                requires_root: false,
                supports_trash: false,
                selected: true,
            });
        }

        // poetry cache
        let poetry_cache = h.join(".cache/pypoetry");
        if poetry_cache.exists() && poetry_cache.is_dir() {
            let size_bytes = calculate_dir_size(&poetry_cache);
            items.push(CleanItem {
                id: format!("dev:python:poetry:{}", poetry_cache.display()),
                category: CleanCategory::DevPython,
                label: "Poetry cache".to_string(),
                path: Some(poetry_cache.to_string_lossy().to_string()),
                size_bytes,
                requires_root: false,
                supports_trash: false,
                selected: true,
            });
        }
    }

    Ok(items)
}

pub fn scan_dev_caches(roots: &[PathBuf]) -> Result<Vec<CleanItem>, AppError> {
    let config = WhitelistConfig::load_default();
    scan_dev_caches_with_config(roots, &config)
}

pub fn scan_dev_caches_with_config(
    roots: &[PathBuf],
    config: &WhitelistConfig,
) -> Result<Vec<CleanItem>, AppError> {
    let mut items = Vec::new();
    items.extend(scan_project_dev_caches_with_config(roots, config)?);
    items.extend(scan_global_dev_caches()?);

    // Filter global caches against whitelist as well
    items.retain(|item| {
        if let Some(ref p) = item.path {
            !is_path_ignored_or_whitelisted(Path::new(p), config)
        } else {
            true
        }
    });

    // Deduplicate by path
    let mut seen = HashSet::new();
    items.retain(|item| {
        if let Some(ref p) = item.path {
            seen.insert(p.clone())
        } else {
            true
        }
    });

    Ok(items)
}
