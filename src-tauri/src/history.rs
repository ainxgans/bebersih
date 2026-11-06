use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::models::{CleanCategory, CleanItem, CleanReport};

static HISTORY_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CleanupRecord {
    pub id: String,
    pub timestamp_epoch_secs: u64,
    pub deleted_bytes: u64,
    pub freed_items_count: usize,
    pub categories: Vec<CleanCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CleanupHistory {
    pub total_freed_bytes: u64,
    pub total_clean_runs: usize,
    pub records: Vec<CleanupRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CleanupStats {
    pub total_freed_bytes: u64,
    pub total_runs: usize,
}

impl CleanupHistory {
    pub fn default_path() -> Option<PathBuf> {
        // ponytail: XDG_DATA_HOME with fallback to ~/.local/share/bebersih/history.json.
        std::env::var("XDG_DATA_HOME")
            .ok()
            .map(|p| PathBuf::from(p).join("bebersih/history.json"))
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".local/share/bebersih/history.json"))
            })
    }

    pub fn load_default() -> Self {
        if let Some(path) = Self::default_path() {
            if path.exists() {
                if let Ok(hist) = load_history_from_file(&path) {
                    return hist;
                }
            }
        }
        Self::default()
    }
}

pub fn load_history_from_file(path: &Path) -> Result<CleanupHistory, AppError> {
    let _lock = HISTORY_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    if !path.exists() {
        return Ok(CleanupHistory::default());
    }
    let content = fs::read_to_string(path).map_err(AppError::from)?;
    let history: CleanupHistory = serde_json::from_str(&content).map_err(AppError::from)?;
    Ok(history)
}

pub fn save_history_to_file(history: &CleanupHistory, path: &Path) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(AppError::from)?;
    }
    let json = serde_json::to_string_pretty(history).map_err(AppError::from)?;
    fs::write(path, json).map_err(AppError::from)?;
    Ok(())
}

pub fn record_cleanup_to_file(
    report: &CleanReport,
    items: &[CleanItem],
    path: &Path,
) -> Result<CleanupHistory, AppError> {
    let _lock = HISTORY_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let mut history = if path.exists() {
        let content = fs::read_to_string(path).map_err(AppError::from)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        CleanupHistory::default()
    };

    if report.deleted_bytes == 0 && report.freed_items.is_empty() {
        return Ok(history);
    }

    let mut categories = Vec::new();
    for freed_id in &report.freed_items {
        if let Some(item) = items.iter().find(|i| &i.id == freed_id) {
            if !categories.contains(&item.category) {
                categories.push(item.category);
            }
        }
    }

    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let record = CleanupRecord {
        id: format!("run-{}", now_secs),
        timestamp_epoch_secs: now_secs,
        deleted_bytes: report.deleted_bytes,
        freed_items_count: report.freed_items.len(),
        categories,
    };

    history.total_freed_bytes += report.deleted_bytes;
    history.total_clean_runs += 1;
    history.records.push(record);

    save_history_to_file(&history, path)?;
    Ok(history)
}

pub fn get_cleanup_stats(history: &CleanupHistory) -> CleanupStats {
    CleanupStats {
        total_freed_bytes: history.total_freed_bytes,
        total_runs: history.total_clean_runs,
    }
}
