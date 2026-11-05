use std::process::Command;
use serde::Deserialize;

use crate::error::AppError;
use crate::models::{CleanCategory, CleanItem};

#[derive(Debug, Clone, Default)]
pub struct DockerScanConfig {
    pub custom_output: Option<String>,
    pub force_disabled: bool,
}

#[derive(Debug, Deserialize)]
struct DockerDfRow {
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

pub fn parse_docker_size(raw: &str) -> Option<u64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    // E.g. "7.22GB (37%)" -> take "7.22GB"
    let token = trimmed.split_whitespace().next()?;
    let token = token.trim_end_matches(|c: char| c.is_ascii_punctuation());

    let num_end = token.find(|c: char| !c.is_ascii_digit() && c != '.')?;
    let (num_str, unit_str) = token.split_at(num_end);
    let num: f64 = num_str.parse().ok()?;

    let multiplier = match unit_str.to_lowercase().as_str() {
        "b" | "bytes" => 1.0,
        "k" | "kb" | "kib" => 1024.0,
        "m" | "mb" | "mib" => 1024.0 * 1024.0,
        "g" | "gb" | "gib" => 1024.0 * 1024.0 * 1024.0,
        "t" | "tb" | "tib" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };

    Some((num * multiplier) as u64)
}

pub fn parse_docker_df_output(output: &str) -> Result<Vec<CleanItem>, AppError> {
    let mut items = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let row: DockerDfRow = match serde_json::from_str(line) {
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

pub fn scan_docker_caches_with_config(config: &DockerScanConfig) -> Result<Vec<CleanItem>, AppError> {
    if config.force_disabled {
        return Ok(Vec::new());
    }

    if let Some(custom) = &config.custom_output {
        let items = parse_docker_df_output(custom)?;
        return Ok(items.into_iter().filter(|i| i.size_bytes > 0).collect());
    }

    // Try executing docker system df --format '{{json .}}'
    let output = match Command::new("docker")
        .args(["system", "df", "--format", "{{json .}}"])
        .output()
    {
        Ok(out) => {
            if !out.status.success() {
                // If daemon is not running or user lacks permission, return empty gracefully
                return Ok(Vec::new());
            }
            String::from_utf8_lossy(&out.stdout).to_string()
        }
        Err(_) => {
            // Docker command not found on host
            return Ok(Vec::new());
        }
    };

    let items = parse_docker_df_output(&output)?;
    Ok(items.into_iter().filter(|i| i.size_bytes > 0).collect())
}

pub fn scan_docker_caches() -> Result<Vec<CleanItem>, AppError> {
    scan_docker_caches_with_config(&DockerScanConfig::default())
}
