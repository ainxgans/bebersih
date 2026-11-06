use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::error::AppError;
use crate::models::{CleanCategory, CleanItem};
use crate::whitelist::{is_path_ignored_or_whitelisted, WhitelistConfig};

pub const DEFAULT_LARGE_FILE_THRESHOLD_BYTES: u64 = 500 * 1024 * 1024; // 500 MB

#[derive(Debug, Clone)]
pub struct LargeFilesScanConfig {
    pub search_dirs: Vec<PathBuf>,
    pub min_size_bytes: u64,
    pub whitelist: WhitelistConfig,
}

impl Default for LargeFilesScanConfig {
    fn default() -> Self {
        // ponytail: scan ~/Downloads and ~/Documents by default
        let home = std::env::var("HOME").ok().map(PathBuf::from);
        let mut search_dirs = Vec::new();
        if let Some(ref h) = home {
            let dl = h.join("Downloads");
            if dl.exists() {
                search_dirs.push(dl);
            }
            let docs = h.join("Documents");
            if docs.exists() {
                search_dirs.push(docs);
            }
        }
        let whitelist = WhitelistConfig::load_default();

        Self {
            search_dirs,
            min_size_bytes: DEFAULT_LARGE_FILE_THRESHOLD_BYTES,
            whitelist,
        }
    }
}

pub fn scan_large_files() -> Result<Vec<CleanItem>, AppError> {
    scan_large_files_with_config(&LargeFilesScanConfig::default())
}

pub fn scan_large_files_with_config(
    config: &LargeFilesScanConfig,
) -> Result<Vec<CleanItem>, AppError> {
    let mut items = Vec::new();
    let config_arc = Arc::new(config.whitelist.clone());

    for root in &config.search_dirs {
        if !root.exists() {
            continue;
        }

        let found = Arc::new(Mutex::new(Vec::new()));
        let found_clone = Arc::clone(&found);
        let cfg_clone = Arc::clone(&config_arc);
        let min_size = config.min_size_bytes;

        for _ in jwalk::WalkDir::new(root)
            .skip_hidden(false)
            .process_read_dir(move |_depth, dir_path, _state, children| {
                if is_path_ignored_or_whitelisted(dir_path, &cfg_clone) {
                    for entry in children.iter_mut().flatten() {
                        entry.read_children_path = None;
                    }
                    return;
                }

                for entry in children.iter_mut().flatten() {
                    let name = entry.file_name();
                    if entry.file_type().is_dir() {
                        if name == ".git"
                            || name == ".cache"
                            || name == ".local"
                            || name == "node_modules"
                            || name == "target"
                        {
                            entry.read_children_path = None;
                        }
                    } else if entry.file_type().is_file() {
                        let path = entry.path();
                        if is_path_ignored_or_whitelisted(&path, &cfg_clone) {
                            continue;
                        }

                        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        if size >= min_size {
                            let filename = path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default();
                            let lower = filename.to_lowercase();

                            let label = if lower.ends_with(".iso")
                                || lower.ends_with(".img")
                                || lower.ends_with(".qcow2")
                                || lower.ends_with(".vdi")
                            {
                                format!("ISO disk image: {}", filename)
                            } else if lower.ends_with(".tar.gz")
                                || lower.ends_with(".tgz")
                                || lower.ends_with(".tar.xz")
                                || lower.ends_with(".tar.bz2")
                                || lower.ends_with(".zip")
                                || lower.ends_with(".7z")
                                || lower.ends_with(".rar")
                                || lower.ends_with(".tar")
                            {
                                format!("Large Archive: {}", filename)
                            } else if lower.ends_with(".sql")
                                || lower.ends_with(".dump")
                                || lower.ends_with(".sqlite")
                                || lower.ends_with(".sqlite3")
                                || lower.ends_with(".db")
                            {
                                format!("Database Dump: {}", filename)
                            } else {
                                format!("Large File: {}", filename)
                            };

                            found_clone.lock().unwrap().push(CleanItem {
                                id: format!("stray:{}", path.display()),
                                category: CleanCategory::LargeFiles,
                                label,
                                path: Some(path.to_string_lossy().to_string()),
                                size_bytes: size,
                                requires_root: false,
                                supports_trash: true,
                                selected: false,
                            });
                        }
                    }
                }
            })
        {
            // drive iterator
        }

        let gathered =
            std::mem::take(&mut *found.lock().map_err(|e| AppError::Other(e.to_string()))?);
        items.extend(gathered);
    }

    Ok(items)
}
