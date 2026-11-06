use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::models::{CleanCategory, CleanItem};
use crate::scanner::dev::calculate_dir_size;

#[derive(Debug, Clone)]
pub struct BrowserScanConfig {
    pub chrome_dirs: Vec<PathBuf>,
    pub firefox_dir: Option<PathBuf>,
    pub brave_dirs: Vec<PathBuf>,
    pub chromium_dir: Option<PathBuf>,
}

impl Default for BrowserScanConfig {
    fn default() -> Self {
        // ponytail: std::env::var("HOME") is standard on Linux/Ubuntu.
        let home = std::env::var("HOME").ok().map(PathBuf::from);
        let cache = home.as_ref().map(|h| h.join(".cache"));

        Self {
            chrome_dirs: cache
                .as_ref()
                .map(|c| {
                    vec![
                        c.join("google-chrome"),
                        c.join("google-chrome-beta"),
                        c.join("google-chrome-unstable"),
                    ]
                })
                .unwrap_or_default(),
            firefox_dir: cache.as_ref().map(|c| c.join("mozilla/firefox")),
            brave_dirs: cache
                .as_ref()
                .map(|c| {
                    vec![
                        c.join("BraveSoftware/Brave-Browser"),
                        c.join("brave"),
                        c.join("brave-browser"),
                    ]
                })
                .unwrap_or_default(),
            chromium_dir: cache.as_ref().map(|c| c.join("chromium")),
        }
    }
}

const CHROMIUM_SUBCACHES: &[(&str, &str, &str)] = &[
    ("Cache", "Web Cache", "cache"),
    ("Code Cache", "Code Cache", "code_cache"),
    ("GPUCache", "GPU Cache", "gpu_cache"),
    ("image_cache", "Image Cache", "image_cache"),
    ("Media Cache", "Media Cache", "media_cache"),
];

const FIREFOX_SUBCACHES: &[(&str, &str, &str)] = &[
    ("cache2", "Web Cache", "cache2"),
    ("startupCache", "Startup Cache", "startup_cache"),
    ("thumbnails", "Thumbnails Cache", "thumbnails"),
    ("jumpListCache", "Jump List Cache", "jump_list_cache"),
];

fn scan_chromium_browser(
    browser_dir: &Path,
    browser_name: &str,
    browser_slug: &str,
    items: &mut Vec<CleanItem>,
) {
    if !browser_dir.exists() || !browser_dir.is_dir() {
        return;
    }

    let entries = match std::fs::read_dir(browser_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }

        let dir_name = entry.file_name().to_string_lossy().to_string();

        // Check if entry itself is a root-level cache folder
        let mut matched_root_subcache = false;
        for (sub_dir, sub_label, sub_id) in CHROMIUM_SUBCACHES {
            if dir_name == *sub_dir {
                matched_root_subcache = true;
                let size = calculate_dir_size(&entry_path);
                if size > 0 {
                    items.push(CleanItem {
                        id: format!("browser:{}:{}:{}", browser_slug, dir_name, sub_id),
                        category: CleanCategory::BrowserCache,
                        label: format!("{} - {}", browser_name, sub_label),
                        path: Some(entry_path.to_string_lossy().to_string()),
                        size_bytes: size,
                        requires_root: false,
                        supports_trash: false,
                        selected: true,
                    });
                }
                break;
            }
        }

        if matched_root_subcache {
            continue;
        }

        // Top-level shader caches
        if dir_name == "ShaderCache" || dir_name == "GrShaderCache" {
            let size = calculate_dir_size(&entry_path);
            if size > 0 {
                items.push(CleanItem {
                    id: format!("browser:{}:{}", browser_slug, dir_name.to_lowercase()),
                    category: CleanCategory::BrowserCache,
                    label: format!("{} - {}", browser_name, dir_name),
                    path: Some(entry_path.to_string_lossy().to_string()),
                    size_bytes: size,
                    requires_root: false,
                    supports_trash: false,
                    selected: true,
                });
            }
            continue;
        }

        // Otherwise assume profile directory (Default, Profile 1, System Profile, etc.)
        let profile_name = dir_name;
        for (sub_dir, sub_label, sub_id) in CHROMIUM_SUBCACHES {
            let sub_path = entry_path.join(sub_dir);
            if sub_path.exists() && sub_path.is_dir() {
                let size = calculate_dir_size(&sub_path);
                if size > 0 {
                    items.push(CleanItem {
                        id: format!("browser:{}:{}:{}", browser_slug, profile_name, sub_id),
                        category: CleanCategory::BrowserCache,
                        label: format!("{} ({}) - {}", browser_name, profile_name, sub_label),
                        path: Some(sub_path.to_string_lossy().to_string()),
                        size_bytes: size,
                        requires_root: false,
                        supports_trash: false,
                        selected: true,
                    });
                }
            }
        }
    }
}

fn scan_firefox_browser(firefox_dir: &Path, items: &mut Vec<CleanItem>) {
    if !firefox_dir.exists() || !firefox_dir.is_dir() {
        return;
    }

    let entries = match std::fs::read_dir(firefox_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }

        let profile_name = entry.file_name().to_string_lossy().to_string();

        for (sub_dir, sub_label, sub_id) in FIREFOX_SUBCACHES {
            let sub_path = entry_path.join(sub_dir);
            if sub_path.exists() && sub_path.is_dir() {
                let size = calculate_dir_size(&sub_path);
                if size > 0 {
                    items.push(CleanItem {
                        id: format!("browser:firefox:{}:{}", profile_name, sub_id),
                        category: CleanCategory::BrowserCache,
                        label: format!("Mozilla Firefox ({}) - {}", profile_name, sub_label),
                        path: Some(sub_path.to_string_lossy().to_string()),
                        size_bytes: size,
                        requires_root: false,
                        supports_trash: false,
                        selected: true,
                    });
                }
            }
        }
    }
}

pub fn scan_browser_caches_with_config(
    config: &BrowserScanConfig,
) -> Result<Vec<CleanItem>, AppError> {
    let mut items = Vec::new();

    // 1. Google Chrome
    for chrome_dir in &config.chrome_dirs {
        scan_chromium_browser(chrome_dir, "Google Chrome", "google-chrome", &mut items);
    }

    // 2. Mozilla Firefox
    if let Some(ref ff_dir) = config.firefox_dir {
        scan_firefox_browser(ff_dir, &mut items);
    }

    // 3. Brave Browser
    for brave_dir in &config.brave_dirs {
        scan_chromium_browser(brave_dir, "Brave", "brave", &mut items);
    }

    // 4. Chromium
    if let Some(ref chromium_dir) = config.chromium_dir {
        scan_chromium_browser(chromium_dir, "Chromium", "chromium", &mut items);
    }

    Ok(items)
}

pub fn scan_browser_caches() -> Result<Vec<CleanItem>, AppError> {
    scan_browser_caches_with_config(&BrowserScanConfig::default())
}
