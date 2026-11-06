use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct WhitelistConfig {
    pub whitelisted_paths: Vec<String>,
}

impl WhitelistConfig {
    pub fn default_path() -> Option<PathBuf> {
        // ponytail: XDG_CONFIG_HOME with fallback to ~/.config/bebersih/whitelist.json.
        std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(|p| PathBuf::from(p).join("bebersih/whitelist.json"))
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".config/bebersih/whitelist.json"))
            })
    }

    pub fn load_default() -> Self {
        if let Some(path) = Self::default_path() {
            if path.exists() {
                if let Ok(config) = load_whitelist_from_file(&path) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub fn save_default(&self) -> Result<(), AppError> {
        if let Some(path) = Self::default_path() {
            save_whitelist_to_file(self, &path)?;
        }
        Ok(())
    }
}

pub fn load_whitelist_from_file(path: &Path) -> Result<WhitelistConfig, AppError> {
    let content = fs::read_to_string(path).map_err(AppError::from)?;
    let config: WhitelistConfig = serde_json::from_str(&content).map_err(AppError::from)?;
    Ok(config)
}

pub fn save_whitelist_to_file(config: &WhitelistConfig, path: &Path) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(AppError::from)?;
    }
    let json = serde_json::to_string_pretty(config).map_err(AppError::from)?;
    fs::write(path, json).map_err(AppError::from)?;
    Ok(())
}

pub fn is_path_ignored_or_whitelisted(path: &Path, config: &WhitelistConfig) -> bool {
    // 1. Check explicit whitelist
    for wl in &config.whitelisted_paths {
        let wl_path = Path::new(wl);
        if path == wl_path || path.starts_with(wl_path) {
            return true;
        }
    }

    // 2. Check for .bebersihignore in current path and its ancestors
    let mut current = Some(path);
    while let Some(dir) = current {
        if dir.join(".bebersihignore").exists() {
            return true;
        }
        current = dir.parent();
    }

    false
}
