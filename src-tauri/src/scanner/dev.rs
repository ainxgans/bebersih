use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use crate::error::AppError;
use crate::models::{CleanCategory, CleanItem};

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
    let mut items = Vec::new();

    for root in roots {
        if !root.exists() {
            continue;
        }

        let matched = Arc::new(Mutex::new(Vec::new()));
        let matched_clone = Arc::clone(&matched);

        for _ in jwalk::WalkDir::new(root)
            .skip_hidden(false)
            .process_read_dir(move |_depth, _dir_path, _state, children| {
                let mut has_cargo_toml = false;
                let mut has_package_json = false;

                for entry in children.iter().flatten() {
                    let name = entry.file_name();
                    if name == "Cargo.toml" && entry.file_type().is_file() {
                        has_cargo_toml = true;
                    } else if name == "package.json" && entry.file_type().is_file() {
                        has_package_json = true;
                    }
                }

                for entry in children.iter_mut().flatten() {
                    if !entry.file_type().is_dir() {
                        continue;
                    }
                    let name = entry.file_name();
                    if name == "target" {
                        entry.read_children_path = None;
                        if has_cargo_toml {
                            matched_clone.lock().unwrap().push((
                                CleanCategory::DevRust,
                                entry.path(),
                                "Rust target".to_string(),
                            ));
                        }
                    } else if name == "node_modules" {
                        entry.read_children_path = None;
                        if has_package_json {
                            matched_clone.lock().unwrap().push((
                                CleanCategory::DevNode,
                                entry.path(),
                                "Node modules".to_string(),
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

    Ok(items)
}

pub fn scan_dev_caches(roots: &[PathBuf]) -> Result<Vec<CleanItem>, AppError> {
    let mut items = Vec::new();
    items.extend(scan_project_dev_caches(roots)?);
    items.extend(scan_global_dev_caches()?);

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
