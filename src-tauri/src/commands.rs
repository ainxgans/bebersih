use std::path::PathBuf;

use crate::models::{CleanItem, CleanReport};
use crate::scanner::{scan_dev_caches, scan_docker_caches, scan_system_caches};

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

        if let Ok(sys_items) = scan_system_caches(30) {
            all_items.extend(sys_items);
        }

        if let Ok(docker_items) = scan_docker_caches() {
            all_items.extend(docker_items);
        }

        Ok(all_items)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn clean_selected(
    items: Vec<CleanItem>,
    move_project_to_trash: bool,
) -> Result<CleanReport, String> {
    tokio::task::spawn_blocking(move || {
        crate::cleaner::clean_items(&items, move_project_to_trash)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
