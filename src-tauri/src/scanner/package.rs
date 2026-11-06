use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::error::AppError;
use crate::models::{CleanCategory, CleanItem};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisabledSnapRevision {
    pub name: String,
    pub revision: String,
    pub snap_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnusedFlatpakRuntime {
    pub id: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Default)]
pub struct PackageScanConfig {
    pub snap_output: Option<String>,
    pub flatpak_output: Option<String>,
    pub snap_file_sizes: HashMap<String, u64>,
}

fn parse_package_size(s: &str) -> u64 {
    let s = s.trim();
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.is_empty() {
        return 0;
    }
    let num = parts[0].parse::<f64>().unwrap_or(0.0);
    let unit = parts.get(1).map(|u| u.to_lowercase()).unwrap_or_default();
    match unit.as_str() {
        "gb" | "gib" => (num * 1_000_000_000.0) as u64,
        "mb" | "mib" => (num * 1_000_000.0) as u64,
        "kb" | "kib" => (num * 1_000.0) as u64,
        _ => num as u64,
    }
}

pub fn parse_snap_list_output(output: &str) -> Vec<DisabledSnapRevision> {
    let mut revisions = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("Name") {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let notes = parts.last().copied().unwrap_or_default();
        if notes.contains("disabled") {
            let name = parts[0].to_string();
            let revision = parts[2].to_string();
            let snap_path = format!("/var/lib/snapd/snaps/{}_{}.snap", name, revision);
            revisions.push(DisabledSnapRevision {
                name,
                revision,
                snap_path,
            });
        }
    }
    revisions
}

pub fn parse_flatpak_unused_output(output: &str) -> Vec<UnusedFlatpakRuntime> {
    let mut runtimes = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("ID") {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let id = parts[0].to_string();
        // Look for size in trailing tokens (e.g. "12.5 MB" or "350.0 MB")
        let mut size_bytes = 0;
        if parts.len() >= 2 {
            let last_two = format!("{} {}", parts[parts.len() - 2], parts[parts.len() - 1]);
            size_bytes = parse_package_size(&last_two);
            if size_bytes == 0 {
                size_bytes = parse_package_size(parts[parts.len() - 1]);
            }
        }

        if !id.is_empty() {
            runtimes.push(UnusedFlatpakRuntime { id, size_bytes });
        }
    }
    runtimes
}

pub fn scan_snap_and_flatpak() -> Result<Vec<CleanItem>, AppError> {
    let snap_output = Command::new("snap")
        .args(["list", "--all"])
        .output()
        .ok()
        .and_then(|o| if o.status.success() { String::from_utf8(o.stdout).ok() } else { None });

    let flatpak_output = Command::new("flatpak")
        .args(["uninstall", "--unused", "--dry-run"])
        .output()
        .ok()
        .and_then(|o| if o.status.success() { String::from_utf8(o.stdout).ok() } else { None });

    let config = PackageScanConfig {
        snap_output,
        flatpak_output,
        snap_file_sizes: HashMap::new(),
    };

    scan_snap_and_flatpak_with_config(&config)
}

pub fn scan_snap_and_flatpak_with_config(
    config: &PackageScanConfig,
) -> Result<Vec<CleanItem>, AppError> {
    let mut items = Vec::new();

    // 1. Process snap disabled revisions
    if let Some(ref out) = config.snap_output {
        let snaps = parse_snap_list_output(out);
        for snap in snaps {
            let path_obj = Path::new(&snap.snap_path);
            let size = if let Some(&s) = config.snap_file_sizes.get(&snap.snap_path) {
                s
            } else if path_obj.exists() {
                fs::metadata(path_obj).map(|m| m.len()).unwrap_or(50_000_000)
            } else {
                50_000_000
            };

            items.push(CleanItem {
                id: format!("snap:{}:{}", snap.name, snap.revision),
                category: CleanCategory::SnapFlatpak,
                label: format!("Old snap revision: {} (rev {})", snap.name, snap.revision),
                path: Some(snap.snap_path),
                size_bytes: size,
                requires_root: true,
                supports_trash: false,
                selected: true,
            });
        }
    }

    // 2. Process flatpak unused runtimes
    if let Some(ref out) = config.flatpak_output {
        let flatpaks = parse_flatpak_unused_output(out);
        for fp in flatpaks {
            items.push(CleanItem {
                id: format!("flatpak:unused:{}", fp.id),
                category: CleanCategory::SnapFlatpak,
                label: format!("Unused flatpak runtime: {}", fp.id),
                path: None,
                size_bytes: fp.size_bytes,
                requires_root: false,
                supports_trash: false,
                selected: true,
            });
        }
    }

    Ok(items)
}
