use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use crate::cleaner::validate_safe_path;
use crate::error::AppError;
use crate::models::{CleanCategory, CleanItem};

pub fn build_root_batch_script(items: &[CleanItem]) -> Result<String, AppError> {
    let mut commands = Vec::new();
    let mut apt_cleaned = false;
    let mut journalctl_cleaned = false;
    let mut paths_to_remove = BTreeSet::new();

    for item in items {
        if !item.requires_root || !item.selected {
            continue;
        }

        match item.category {
            CleanCategory::AptCache => {
                if !apt_cleaned {
                    commands.push("apt-get clean".to_string());
                    apt_cleaned = true;
                }
            }
            CleanCategory::SystemLogs => {
                if item.id == "system:journalctl" || item.path.is_none() {
                    if !journalctl_cleaned {
                        commands.push("journalctl --vacuum-time=7d".to_string());
                        journalctl_cleaned = true;
                    }
                } else if let Some(ref path_str) = item.path {
                    let path = Path::new(path_str);
                    validate_safe_path(path)?;
                    paths_to_remove.insert(path_str.clone());
                }
            }
            CleanCategory::SnapFlatpak => {
                if item.id.starts_with("snap:") {
                    let parts: Vec<&str> = item.id.split(':').collect();
                    if parts.len() >= 3 {
                        let name = parts[1];
                        let rev = parts[2];
                        commands.push(format!("snap remove --revision={} {}", rev, name));
                    }
                } else if let Some(ref path_str) = item.path {
                    let path = Path::new(path_str);
                    validate_safe_path(path)?;
                    paths_to_remove.insert(path_str.clone());
                }
            }
            _ => {
                if let Some(ref path_str) = item.path {
                    let path = Path::new(path_str);
                    validate_safe_path(path)?;
                    paths_to_remove.insert(path_str.clone());
                }
            }
        }
    }

    for path_str in paths_to_remove {
        let escaped = path_str.replace('\'', "'\\''");
        commands.push(format!("rm -rf -- '{}'", escaped));
    }

    if commands.is_empty() {
        return Ok(String::new());
    }

    Ok(format!("set -e\n{}", commands.join("\n")))
}

pub fn generate_root_batch_script(items: &[CleanItem]) -> String {
    build_root_batch_script(items).unwrap_or_default()
}

// ponytail: pkexec opens native polkit GUI dialog on Ubuntu; single prompt for all batch tasks.
pub fn execute_root_batch(script: &str) -> Result<(), AppError> {
    if script.trim().is_empty() {
        return Ok(());
    }

    let status = Command::new("pkexec")
        .args(["bash", "-c", script])
        .status()
        .map_err(|e| AppError::CommandFailed(format!("Failed to execute pkexec: {}", e)))?;

    if !status.success() {
        return Err(AppError::CommandFailed(format!(
            "pkexec batch failed with status: {:?}",
            status.code()
        )));
    }

    Ok(())
}
