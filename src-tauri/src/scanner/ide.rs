use std::fs;
use std::path::PathBuf;

use crate::error::AppError;
use crate::models::{CleanCategory, CleanItem};
use crate::scanner::dev::calculate_dir_size;

#[derive(Debug, Clone)]
pub struct IdeScanConfig {
    pub config_dir: Option<PathBuf>,
    pub cache_dir: Option<PathBuf>,
}

impl Default for IdeScanConfig {
    fn default() -> Self {
        // ponytail: fallback to standard ~/.config and ~/.cache
        let home = std::env::var("HOME").ok().map(PathBuf::from);
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| home.as_ref().map(|h| h.join(".config")));
        let cache_dir = std::env::var("XDG_CACHE_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| home.as_ref().map(|h| h.join(".cache")));

        Self {
            config_dir,
            cache_dir,
        }
    }
}

pub fn scan_ide_caches() -> Result<Vec<CleanItem>, AppError> {
    scan_ide_caches_with_config(&IdeScanConfig::default())
}

pub fn scan_ide_caches_with_config(config: &IdeScanConfig) -> Result<Vec<CleanItem>, AppError> {
    let mut items = Vec::new();

    // 1. Electron/VS Code based editors in config_dir
    if let Some(ref cfg_dir) = config.config_dir {
        let editors = [
            ("Code", "VS Code"),
            ("Cursor", "Cursor"),
            ("VSCodium", "VSCodium"),
        ];

        let subcaches = [
            ("Cache", "cache", "Cache"),
            ("CachedData", "cached-data", "Cached Data"),
            ("Service Worker/CacheStorage", "sw-cache", "Service Worker Cache"),
        ];

        for (editor_dir, editor_name) in editors {
            let base = cfg_dir.join(editor_dir);
            if !base.exists() {
                continue;
            }

            for (rel_path, id_suffix, label_suffix) in subcaches {
                let target_path = base.join(rel_path);
                if target_path.exists() && target_path.is_dir() {
                    let size_bytes = calculate_dir_size(&target_path);
                    if size_bytes > 0 {
                        items.push(CleanItem {
                            id: format!(
                                "ide:{}:{}:{}",
                                editor_dir.to_lowercase(),
                                id_suffix,
                                target_path.display()
                            ),
                            category: CleanCategory::IdeCache,
                            label: format!("{} {}", editor_name, label_suffix),
                            path: Some(target_path.to_string_lossy().to_string()),
                            size_bytes,
                            requires_root: false,
                            supports_trash: true,
                            selected: true,
                        });
                    }
                }
            }
        }
    }

    // 2. JetBrains IDE caches in cache_dir/JetBrains/*/caches
    if let Some(ref cache_dir) = config.cache_dir {
        let jb_root = cache_dir.join("JetBrains");
        if jb_root.exists() && jb_root.is_dir() {
            if let Ok(entries) = fs::read_dir(&jb_root) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let ide_name = entry.file_name().to_string_lossy().to_string();
                        let caches_path = path.join("caches");
                        if caches_path.exists() && caches_path.is_dir() {
                            let size_bytes = calculate_dir_size(&caches_path);
                            if size_bytes > 0 {
                                items.push(CleanItem {
                                    id: format!("ide:jetbrains:{}:{}", ide_name, caches_path.display()),
                                    category: CleanCategory::IdeCache,
                                    label: format!("JetBrains {} caches", ide_name),
                                    path: Some(caches_path.to_string_lossy().to_string()),
                                    size_bytes,
                                    requires_root: false,
                                    supports_trash: true,
                                    selected: true,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(items)
}
