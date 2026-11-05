use std::process::Command;
use serde::Deserialize;

use crate::error::AppError;
use crate::models::{CleanCategory, CleanItem};

#[derive(Debug, Clone, Default)]
pub struct DockerScanConfig {
    pub custom_output: Option<String>,
    pub force_disabled: bool,
}

pub fn parse_docker_size(raw: &str) -> Option<u64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "N/A" {
        return None;
    }

    let token = trimmed.split_whitespace().next()?;
    let token = token.trim_end_matches(|c: char| c.is_ascii_punctuation());

    let (num_str, unit_str) = match token.find(|c: char| !c.is_ascii_digit() && c != '.') {
        Some(idx) => token.split_at(idx),
        None => (token, ""),
    };

    let num: f64 = num_str.parse().ok()?;

    let multiplier = match unit_str.to_lowercase().as_str() {
        "" | "b" | "bytes" => 1.0,
        "k" | "kb" | "kib" => 1024.0,
        "m" | "mb" | "mib" => 1024.0 * 1024.0,
        "g" | "gb" | "gib" => 1024.0 * 1024.0 * 1024.0,
        "t" | "tb" | "tib" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };

    Some((num * multiplier).round() as u64)
}

#[derive(Debug, Deserialize)]
struct DockerDfSummaryRow {
    #[serde(rename = "Type")]
    type_name: String,
    #[serde(rename = "TotalCount", default)]
    _total_count: String,
    #[serde(rename = "Active", default)]
    _active: String,
    #[serde(rename = "Size", default)]
    size: String,
    #[serde(rename = "Reclaimable", default)]
    reclaimable: String,
}

pub fn parse_docker_df_summary(output: &str) -> Result<Vec<CleanItem>, AppError> {
    let mut items = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let row: DockerDfSummaryRow = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let reclaimable_bytes = parse_docker_size(&row.reclaimable).unwrap_or(0);
        let total_size_bytes = parse_docker_size(&row.size).unwrap_or(0);
        let size_bytes = if reclaimable_bytes > 0 {
            reclaimable_bytes
        } else if row.type_name == "Build Cache" {
            total_size_bytes
        } else {
            0
        };

        let (id, category, label, selected) = match row.type_name.as_str() {
            "Images" => (
                "docker:images",
                CleanCategory::DockerCache,
                "Docker Dangling & Unused Images",
                true,
            ),
            "Containers" => (
                "docker:containers",
                CleanCategory::DockerCache,
                "Docker Stopped Containers",
                true,
            ),
            "Local Volumes" => (
                "docker:volumes",
                CleanCategory::DockerVolume,
                "Docker Unused Volumes",
                false, // Critical: never selected by default
            ),
            "Build Cache" => (
                "docker:build_cache",
                CleanCategory::DockerCache,
                "Docker Build Cache",
                true,
            ),
            _ => continue,
        };

        items.push(CleanItem {
            id: id.to_string(),
            category,
            label: label.to_string(),
            path: None,
            size_bytes,
            requires_root: false,
            supports_trash: false,
            selected,
        });
    }

    Ok(items)
}

#[derive(Debug, Deserialize)]
struct VerboseImage {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "Repository", default)]
    repository: String,
    #[serde(rename = "Tag", default)]
    tag: String,
    #[serde(rename = "Size", default)]
    size: String,
}

#[derive(Debug, Deserialize)]
struct VerboseContainer {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "Names", default)]
    names: String,
    #[serde(rename = "State", default)]
    state: String,
    #[serde(rename = "Size", default)]
    size: String,
}

#[derive(Debug, Deserialize)]
struct VerboseVolume {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Links", default)]
    links: String,
    #[serde(rename = "Size", default)]
    size: String,
    #[serde(rename = "Mountpoint", default)]
    mountpoint: String,
}

#[derive(Debug, Deserialize)]
struct VerboseBuildCache {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "Description", default)]
    description: String,
    #[serde(rename = "Size", default)]
    size: String,
    #[serde(rename = "InUse", default)]
    in_use: bool,
    #[serde(rename = "Reclaimable", default)]
    reclaimable: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct VerboseDf {
    #[serde(rename = "Images", default)]
    images: Vec<VerboseImage>,
    #[serde(rename = "Containers", default)]
    containers: Vec<VerboseContainer>,
    #[serde(rename = "Volumes", default)]
    volumes: Vec<VerboseVolume>,
    #[serde(rename = "BuildCache", default)]
    build_cache: Vec<VerboseBuildCache>,
}

pub fn parse_docker_df_verbose(json_str: &str) -> Result<Vec<CleanItem>, AppError> {
    let parsed: VerboseDf = serde_json::from_str(json_str)
        .map_err(|e| AppError::Other(format!("Failed to parse docker verbose json: {}", e)))?;

    let mut items = Vec::new();

    // 1. Images
    for img in parsed.images {
        let is_dangling = img.repository == "<none>" || img.tag == "<none>";
        if is_dangling {
            let size = parse_docker_size(&img.size).unwrap_or(0);
            items.push(CleanItem {
                id: format!("docker:image:{}", img.id),
                category: CleanCategory::DockerCache,
                label: format!("Dangling Image ({})", img.id),
                path: None,
                size_bytes: size,
                requires_root: false,
                supports_trash: false,
                selected: true,
            });
        }
    }

    // 2. Containers
    for cont in parsed.containers {
        if cont.state.to_lowercase() != "running" {
            let size = parse_docker_size(&cont.size).unwrap_or(0);
            items.push(CleanItem {
                id: format!("docker:container:{}", cont.id),
                category: CleanCategory::DockerCache,
                label: format!("Stopped Container ({})", cont.names),
                path: None,
                size_bytes: size,
                requires_root: false,
                supports_trash: false,
                selected: true,
            });
        }
    }

    // 3. Volumes
    for vol in parsed.volumes {
        let is_unused = vol.links == "0";
        if is_unused {
            let size = parse_docker_size(&vol.size).unwrap_or(0);
            let mount = if vol.mountpoint.is_empty() {
                None
            } else {
                Some(vol.mountpoint)
            };
            items.push(CleanItem {
                id: format!("docker:volume:{}", vol.name),
                category: CleanCategory::DockerVolume,
                label: format!("Unused Volume ({})", vol.name),
                path: mount,
                size_bytes: size,
                requires_root: false,
                supports_trash: false,
                selected: false, // Critical safety guard
            });
        }
    }

    // 4. BuildCache
    for cache in parsed.build_cache {
        let reclaimable_bool = match &cache.reclaimable {
            serde_json::Value::Bool(b) => *b,
            serde_json::Value::String(s) => s.to_lowercase() == "true",
            _ => false,
        };

        if !cache.in_use || reclaimable_bool {
            let size = parse_docker_size(&cache.size).unwrap_or(0);
            let desc = if cache.description.is_empty() {
                cache.id.clone()
            } else {
                cache.description
            };
            items.push(CleanItem {
                id: format!("docker:buildcache:{}", cache.id),
                category: CleanCategory::DockerCache,
                label: format!("Build Cache ({})", desc),
                path: None,
                size_bytes: size,
                requires_root: false,
                supports_trash: false,
                selected: true,
            });
        }
    }

    Ok(items)
}

pub fn parse_docker_df_output(output: &str) -> Result<Vec<CleanItem>, AppError> {
    if let Ok(items) = parse_docker_df_verbose(output) {
        return Ok(items);
    }
    parse_docker_df_summary(output)
}

pub fn scan_docker_caches_with_config(config: &DockerScanConfig) -> Result<Vec<CleanItem>, AppError> {
    if config.force_disabled {
        return Ok(Vec::new());
    }

    if let Some(custom) = &config.custom_output {
        if let Ok(verbose_items) = parse_docker_df_verbose(custom) {
            return Ok(verbose_items.into_iter().filter(|i| i.size_bytes > 0).collect());
        }
        let summary_items = parse_docker_df_summary(custom)?;
        return Ok(summary_items.into_iter().filter(|i| i.size_bytes > 0).collect());
    }

    // Attempt verbose query first
    let verbose_out = Command::new("docker")
        .args(["system", "df", "-v", "--format", "{{json .}}"])
        .output();

    if let Ok(out) = verbose_out {
        if out.status.success() {
            let stdout_str = String::from_utf8_lossy(&out.stdout);
            if let Ok(items) = parse_docker_df_verbose(&stdout_str) {
                return Ok(items.into_iter().filter(|i| i.size_bytes > 0).collect());
            }
        }
    }

    // Fallback to summary query
    let summary_out = Command::new("docker")
        .args(["system", "df", "--format", "{{json .}}"])
        .output();

    if let Ok(out) = summary_out {
        if out.status.success() {
            let stdout_str = String::from_utf8_lossy(&out.stdout);
            if let Ok(items) = parse_docker_df_summary(&stdout_str) {
                return Ok(items.into_iter().filter(|i| i.size_bytes > 0).collect());
            }
        }
    }

    // If docker is not available or down, return empty vector gracefully
    Ok(Vec::new())
}

pub fn scan_docker_caches() -> Result<Vec<CleanItem>, AppError> {
    scan_docker_caches_with_config(&DockerScanConfig::default())
}
