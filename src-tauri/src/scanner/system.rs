use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

use crate::error::AppError;
use crate::models::{CleanCategory, CleanItem};
use crate::scanner::dev::calculate_dir_size;

#[derive(Debug, Clone)]
pub struct SystemScanConfig {
    pub apt_cache_dir: Option<PathBuf>,
    pub thumbnails_dir: Option<PathBuf>,
    pub user_cache_dir: Option<PathBuf>,
    pub crash_dir: Option<PathBuf>,
    pub journal_size_override: Option<u64>,
}

impl Default for SystemScanConfig {
    fn default() -> Self {
        // ponytail: std::env::var("HOME") is standard on Linux/Ubuntu.
        let home = std::env::var("HOME").ok().map(PathBuf::from);
        Self {
            apt_cache_dir: Some(PathBuf::from("/var/cache/apt/archives")),
            thumbnails_dir: home.as_ref().map(|h| h.join(".cache/thumbnails")),
            user_cache_dir: home.as_ref().map(|h| h.join(".cache")),
            crash_dir: Some(PathBuf::from("/var/crash")),
            journal_size_override: None,
        }
    }
}

pub fn parse_journal_disk_usage(output: &str) -> Option<u64> {
    let lower = output.to_lowercase();
    let idx = lower
        .find("take up ")
        .map(|i| i + "take up ".len())
        .or_else(|| lower.find("takes up ").map(|i| i + "takes up ".len()))?;

    let remainder = lower[idx..].trim_start();
    let token = remainder.split_whitespace().next()?;
    let token = token.trim_end_matches(|c: char| c.is_ascii_punctuation());

    let num_end = token.find(|c: char| !c.is_ascii_digit() && c != '.')?;
    let (num_str, unit_str) = token.split_at(num_end);
    let num: f64 = num_str.parse().ok()?;

    let multiplier = match unit_str {
        "b" | "bytes" => 1.0,
        "k" | "kb" | "kib" => 1024.0,
        "m" | "mb" | "mib" => 1024.0 * 1024.0,
        "g" | "gb" | "gib" => 1024.0 * 1024.0 * 1024.0,
        "t" | "tb" | "tib" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };

    Some((num * multiplier) as u64)
}

pub fn get_journal_disk_usage() -> Option<u64> {
    let output = Command::new("journalctl")
        .arg("--disk-usage")
        .output()
        .ok()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_journal_disk_usage(&stdout)
    } else {
        None
    }
}

pub fn scan_apt_cache(apt_dir: &Path) -> Option<CleanItem> {
    if !apt_dir.exists() || !apt_dir.is_dir() {
        return None;
    }

    let entries = std::fs::read_dir(apt_dir).ok()?;
    let mut total_deb_size = 0u64;
    let mut deb_count = 0usize;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("deb") {
            if let Ok(meta) = entry.metadata() {
                total_deb_size += meta.len();
                deb_count += 1;
            }
        }
    }

    if deb_count > 0 {
        Some(CleanItem {
            id: format!("system:apt:{}", apt_dir.display()),
            category: CleanCategory::AptCache,
            label: "APT package cache".to_string(),
            path: Some(apt_dir.to_string_lossy().to_string()),
            size_bytes: total_deb_size,
            requires_root: true,
            supports_trash: false,
            selected: true,
        })
    } else {
        None
    }
}

pub fn scan_thumbnails_cache(thumbnails_dir: &Path) -> Option<CleanItem> {
    if !thumbnails_dir.exists() || !thumbnails_dir.is_dir() {
        return None;
    }

    let size = calculate_dir_size(thumbnails_dir);
    Some(CleanItem {
        id: format!("system:thumbnails:{}", thumbnails_dir.display()),
        category: CleanCategory::StaleCache,
        label: "Thumbnail cache".to_string(),
        path: Some(thumbnails_dir.to_string_lossy().to_string()),
        size_bytes: size,
        requires_root: false,
        supports_trash: false,
        selected: true,
    })
}

pub fn scan_crash_logs(crash_dir: &Path, min_age_days: u32) -> Result<Vec<CleanItem>, AppError> {
    if !crash_dir.exists() || !crash_dir.is_dir() {
        return Ok(Vec::new());
    }

    let entries = match std::fs::read_dir(crash_dir) {
        Ok(e) => e,
        Err(_) => return Ok(Vec::new()),
    };

    let min_age = Duration::from_secs(min_age_days as u64 * 86400);
    let now = SystemTime::now();
    let mut items = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("crash") {
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    if let Ok(age) = now.duration_since(modified) {
                        if age >= min_age {
                            let file_name = path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "crash log".to_string());
                            items.push(CleanItem {
                                id: format!("system:crash:{}", path.display()),
                                category: CleanCategory::SystemLogs,
                                label: format!("Crash log ({})", file_name),
                                path: Some(path.to_string_lossy().to_string()),
                                size_bytes: meta.len(),
                                requires_root: true,
                                supports_trash: false,
                                selected: true,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(items)
}

pub fn scan_stale_cache_files(
    cache_dir: &Path,
    stale_days: u32,
    exclude_dirs: &[PathBuf],
) -> Result<Vec<CleanItem>, AppError> {
    if !cache_dir.exists() || !cache_dir.is_dir() {
        return Ok(Vec::new());
    }

    let min_age = Duration::from_secs(stale_days as u64 * 86400);
    let now = SystemTime::now();
    let mut items = Vec::new();

    let excludes: Vec<PathBuf> = exclude_dirs.to_vec();

    for entry in jwalk::WalkDir::new(cache_dir)
        .skip_hidden(false)
        .process_read_dir({
            let excludes = excludes.clone();
            move |_depth, _dir_path, _state, children| {
                for entry in children.iter_mut().flatten() {
                    if entry.file_type().is_dir() {
                        let entry_path = entry.path();
                        if excludes.iter().any(|ex| entry_path.starts_with(ex)) {
                            entry.read_children_path = None;
                        }
                    }
                }
            }
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        if excludes.iter().any(|ex| path.starts_with(ex)) {
            continue;
        }

        if let Ok(meta) = entry.metadata() {
            if let Ok(modified) = meta.modified() {
                if let Ok(age) = now.duration_since(modified) {
                    if age >= min_age {
                        let file_name = path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "cache file".to_string());
                        items.push(CleanItem {
                            id: format!("system:stale:{}", path.display()),
                            category: CleanCategory::StaleCache,
                            label: format!("Stale cache ({})", file_name),
                            path: Some(path.to_string_lossy().to_string()),
                            size_bytes: meta.len(),
                            requires_root: false,
                            supports_trash: false,
                            selected: true,
                        });
                    }
                }
            }
        }
    }

    Ok(items)
}

pub fn scan_system_caches_with_config(
    stale_days: u32,
    config: &SystemScanConfig,
) -> Result<Vec<CleanItem>, AppError> {
    let mut items = Vec::new();

    // 1. APT Cache
    if let Some(ref apt_dir) = config.apt_cache_dir {
        if let Some(apt_item) = scan_apt_cache(apt_dir) {
            items.push(apt_item);
        }
    }

    // 2. Thumbnail Cache
    if let Some(ref thumb_dir) = config.thumbnails_dir {
        if let Some(thumb_item) = scan_thumbnails_cache(thumb_dir) {
            items.push(thumb_item);
        }
    }

    // 3. Stale Files in user cache
    if let Some(ref cache_dir) = config.user_cache_dir {
        let mut excludes = Vec::new();
        if let Some(ref thumb_dir) = config.thumbnails_dir {
            excludes.push(thumb_dir.clone());
        }
        if let Ok(stale_items) = scan_stale_cache_files(cache_dir, stale_days, &excludes) {
            items.extend(stale_items);
        }
    }

    // 4. Crash Logs (older than 7 days)
    if let Some(ref crash_dir) = config.crash_dir {
        if let Ok(crash_items) = scan_crash_logs(crash_dir, 7) {
            items.extend(crash_items);
        }
    }

    // 5. Systemd Journal Logs
    if let Some(override_size) = config.journal_size_override {
        if override_size > 0 {
            items.push(CleanItem {
                id: "system:journalctl".to_string(),
                category: CleanCategory::SystemLogs,
                label: "Systemd journal logs".to_string(),
                path: None,
                size_bytes: override_size,
                requires_root: true,
                supports_trash: false,
                selected: true,
            });
        }
    } else if let Some(journal_size) = get_journal_disk_usage() {
        if journal_size > 0 {
            items.push(CleanItem {
                id: "system:journalctl".to_string(),
                category: CleanCategory::SystemLogs,
                label: "Systemd journal logs".to_string(),
                path: None,
                size_bytes: journal_size,
                requires_root: true,
                supports_trash: false,
                selected: true,
            });
        }
    }

    Ok(items)
}

pub fn scan_system_caches(stale_days: u32) -> Result<Vec<CleanItem>, AppError> {
    scan_system_caches_with_config(stale_days, &SystemScanConfig::default())
}
