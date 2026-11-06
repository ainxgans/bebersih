use std::fs;
use std::path::{Path, PathBuf};

use crate::cleaner::purge::purge_path;
use crate::error::AppError;
use crate::models::{CleanCategory, CleanItem, CleanReport};
use crate::scanner::dev::calculate_dir_size;

#[derive(Debug, Clone)]
pub struct TrashScanConfig {
    pub trash_dir: Option<PathBuf>,
}

impl Default for TrashScanConfig {
    fn default() -> Self {
        // ponytail: XDG_DATA_HOME/Trash with fallback to ~/.local/share/Trash is FreeDesktop standard.
        let trash_dir = std::env::var("XDG_DATA_HOME")
            .ok()
            .map(|p| PathBuf::from(p).join("Trash"))
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".local/share/Trash"))
            });

        Self { trash_dir }
    }
}

pub fn scan_user_trash() -> Result<Vec<CleanItem>, AppError> {
    let config = TrashScanConfig::default();
    scan_user_trash_with_config(&config)
}

pub fn scan_user_trash_with_config(config: &TrashScanConfig) -> Result<Vec<CleanItem>, AppError> {
    let trash_dir = match &config.trash_dir {
        Some(dir) => dir,
        None => return Ok(Vec::new()),
    };

    let files_dir = trash_dir.join("files");
    let info_dir = trash_dir.join("info");

    if !files_dir.exists() || !files_dir.is_dir() {
        return Ok(Vec::new());
    }

    let read_dir = match fs::read_dir(&files_dir) {
        Ok(rd) => rd,
        Err(_) => return Ok(Vec::new()),
    };

    let mut items = Vec::new();

    for entry in read_dir.flatten() {
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        let mut size = calculate_dir_size(&path);

        let info_file = info_dir.join(format!("{}.trashinfo", file_name));
        let mut original_path = None;

        if info_file.exists() {
            if let Ok(meta) = fs::metadata(&info_file) {
                size += meta.len();
            }
            if let Ok(content) = fs::read_to_string(&info_file) {
                for line in content.lines() {
                    if let Some(stripped) = line.strip_prefix("Path=") {
                        original_path = Some(stripped.trim().to_string());
                        break;
                    }
                }
            }
        }

        let label = match original_path {
            Some(orig) => format!("Trash: {} ({})", file_name, orig),
            None => format!("Trash: {}", file_name),
        };

        items.push(CleanItem {
            id: format!("trash:{}", file_name),
            category: CleanCategory::UserTrash,
            label,
            path: Some(path.to_string_lossy().to_string()),
            size_bytes: size,
            requires_root: false,
            supports_trash: false,
            selected: true,
        });
    }

    Ok(items)
}

pub fn empty_user_trash() -> Result<CleanReport, AppError> {
    let config = TrashScanConfig::default();
    empty_user_trash_with_config(&config)
}

pub fn empty_user_trash_with_config(config: &TrashScanConfig) -> Result<CleanReport, AppError> {
    let items = scan_user_trash_with_config(config)?;
    let mut report = CleanReport::default();

    let trash_dir = match &config.trash_dir {
        Some(dir) => dir,
        None => return Ok(report),
    };

    let info_dir = trash_dir.join("info");

    for item in items {
        if let Some(ref path_str) = item.path {
            let path = Path::new(path_str);
            let file_name = path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default();

            match purge_path(path) {
                Ok(_) => {
                    report.deleted_bytes += item.size_bytes;
                    report.freed_items.push(item.id.clone());

                    if !file_name.is_empty() {
                        let info_file = info_dir.join(format!("{}.trashinfo", file_name));
                        if info_file.exists() {
                            let _ = purge_path(&info_file);
                        }
                    }
                }
                Err(e) => {
                    report.errors.push(format!("{}: {}", item.id, e));
                }
            }
        }
    }

    Ok(report)
}
