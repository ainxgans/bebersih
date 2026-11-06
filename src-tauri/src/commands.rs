use std::path::PathBuf;

use crate::models::{CleanItem, CleanReport};
use crate::scanner::{
    empty_user_trash, scan_browser_caches, scan_dev_caches, scan_docker_caches,
    scan_ide_caches, scan_large_files, scan_snap_and_flatpak, scan_system_caches,
    scan_user_trash,
};

fn get_disk_free_space_internal() -> Result<u64, String> {
    // ponytail: df -B1 is standard on Linux/Ubuntu coreutils.
    let target = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
    let output = std::process::Command::new("df")
        .args(["-B1", "--output=avail", &target])
        .output()
        .map_err(|e| format!("Failed to run df: {}", e))?;

    if !output.status.success() {
        return Err(format!("df failed with status: {:?}", output.status));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let last_line = stdout.lines().last().ok_or_else(|| "Empty df output".to_string())?;
    let free_bytes = last_line
        .trim()
        .parse::<u64>()
        .map_err(|e| format!("Failed to parse df output '{}': {}", last_line, e))?;

    Ok(free_bytes)
}

#[tauri::command]
pub async fn get_disk_free_space() -> Result<u64, String> {
    tokio::task::spawn_blocking(get_disk_free_space_internal)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn scan_all(custom_paths: Vec<String>) -> Result<Vec<CleanItem>, String> {
    tokio::task::spawn_blocking(move || {
        let mut all_items = Vec::new();

        let roots: Vec<PathBuf> = if custom_paths.is_empty() {
            let home = std::env::var("HOME").ok().map(PathBuf::from);
            let mut detected_roots = Vec::new();
            if let Some(h) = home {
                for dir in &["Projects", "code", "workspace", "dev", "git", "source"] {
                    let p = h.join(dir);
                    if p.exists() {
                        detected_roots.push(p);
                    }
                }
                if detected_roots.is_empty() {
                    detected_roots.push(h);
                }
            }
            detected_roots
        } else {
            custom_paths.into_iter().map(PathBuf::from).collect()
        };

        if let Ok(dev_items) = scan_dev_caches(&roots) {
            all_items.extend(dev_items);
        }

        if let Ok(browser_items) = scan_browser_caches() {
            all_items.extend(browser_items);
        }

        if let Ok(sys_items) = scan_system_caches(30) {
            all_items.extend(sys_items);
        }

        if let Ok(trash_items) = scan_user_trash() {
            all_items.extend(trash_items);
        }

        if let Ok(docker_items) = scan_docker_caches() {
            all_items.extend(docker_items);
        }

        if let Ok(pkg_items) = scan_snap_and_flatpak() {
            all_items.extend(pkg_items);
        }

        if let Ok(ide_items) = scan_ide_caches() {
            all_items.extend(ide_items);
        }

        if let Ok(stray_items) = scan_large_files() {
            all_items.extend(stray_items);
        }

        Ok(all_items)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn empty_trash() -> Result<CleanReport, String> {
    tokio::task::spawn_blocking(|| empty_user_trash().map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn clean_selected(
    items: Vec<CleanItem>,
    move_project_to_trash: bool,
) -> Result<CleanReport, String> {
    tokio::task::spawn_blocking(move || {
        let config = crate::whitelist::WhitelistConfig::load_default();
        let report = crate::cleaner::clean_items_with_whitelist(&items, move_project_to_trash, &config)
            .map_err(|e| e.to_string())?;

        if let Some(hist_path) = crate::history::CleanupHistory::default_path() {
            let _ = crate::history::record_cleanup_to_file(&report, &items, &hist_path);
        }

        Ok(report)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_cleanup_history() -> Result<crate::history::CleanupHistory, String> {
    tokio::task::spawn_blocking(|| Ok(crate::history::CleanupHistory::load_default()))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_whitelist() -> Result<Vec<String>, String> {
    tokio::task::spawn_blocking(|| {
        let config = crate::whitelist::WhitelistConfig::load_default();
        Ok(config.whitelisted_paths)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn add_to_whitelist(path: String) -> Result<Vec<String>, String> {
    tokio::task::spawn_blocking(move || {
        let mut config = crate::whitelist::WhitelistConfig::load_default();
        if !config.whitelisted_paths.contains(&path) {
            config.whitelisted_paths.push(path);
            config.save_default().map_err(|e| e.to_string())?;
        }
        Ok(config.whitelisted_paths)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn remove_from_whitelist(path: String) -> Result<Vec<String>, String> {
    tokio::task::spawn_blocking(move || {
        let mut config = crate::whitelist::WhitelistConfig::load_default();
        config.whitelisted_paths.retain(|p| p != &path);
        config.save_default().map_err(|e| e.to_string())?;
        Ok(config.whitelisted_paths)
    })
    .await
    .map_err(|e| e.to_string())?
}
