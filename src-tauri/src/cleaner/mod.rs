pub mod purge;
pub mod root;
pub mod trash;

pub use purge::purge_path;
pub use root::{build_root_batch_script, execute_root_batch, generate_root_batch_script};
pub use trash::move_to_trash;

use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::models::{CleanItem, CleanReport};

pub fn validate_safe_path(path: &Path) -> Result<(), AppError> {
    let s = path.to_string_lossy();
    if s.trim().is_empty() {
        return Err(AppError::InvalidPath("Path cannot be empty".to_string()));
    }

    if !path.is_absolute() {
        return Err(AppError::InvalidPath(format!(
            "Path must be absolute: {:?}",
            path
        )));
    }

    let mut normalized = PathBuf::new();
    for comp in path.components() {
        match comp {
            std::path::Component::Prefix(p) => normalized.push(p.as_os_str()),
            std::path::Component::RootDir => normalized.push("/"),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::Normal(c) => normalized.push(c),
        }
    }

    if normalized == Path::new("/") {
        return Err(AppError::InvalidPath(
            "Deleting root directory '/' is strictly prohibited".to_string(),
        ));
    }

    const FORBIDDEN_EXACT: &[&str] = &[
        "/bin", "/boot", "/dev", "/etc", "/home", "/lib", "/lib32", "/lib64",
        "/libx32", "/media", "/mnt", "/opt", "/proc", "/root", "/run", "/sbin",
        "/srv", "/sys", "/tmp", "/usr", "/var",
    ];

    for forbidden in FORBIDDEN_EXACT {
        if normalized == Path::new(forbidden) {
            return Err(AppError::InvalidPath(format!(
                "Deleting system directory '{}' is strictly prohibited",
                forbidden
            )));
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        if normalized == Path::new(&home) {
            return Err(AppError::InvalidPath(format!(
                "Deleting user home directory '{}' is strictly prohibited",
                home
            )));
        }
    }

    const FORBIDDEN_PREFIXES: &[&str] = &[
        "/bin", "/boot", "/dev", "/etc", "/lib", "/lib32", "/lib64", "/libx32",
        "/proc", "/root", "/sbin", "/sys", "/usr",
    ];

    for prefix in FORBIDDEN_PREFIXES {
        if normalized.starts_with(prefix) {
            return Err(AppError::InvalidPath(format!(
                "Modifying paths under '{}' is strictly prohibited",
                prefix
            )));
        }
    }

    if normalized.starts_with("/var") {
        let allowed_var = [
            "/var/cache",
            "/var/crash",
            "/var/log",
            "/var/tmp",
        ];
        let is_allowed = allowed_var.iter().any(|p| normalized.starts_with(p));
        if !is_allowed {
            return Err(AppError::InvalidPath(format!(
                "Modifying paths under '/var' outside cache/logs is strictly prohibited: {:?}",
                normalized
            )));
        }
    }

    if normalized.starts_with("/home") {
        let count = normalized.components().count();
        if count <= 3 {
            return Err(AppError::InvalidPath(format!(
                "Deleting user profile directory is strictly prohibited: {:?}",
                normalized
            )));
        }
    }

    Ok(())
}

pub fn clean_items(
    items: &[CleanItem],
    move_project_to_trash: bool,
) -> Result<CleanReport, AppError> {
    clean_items_with_root_runner(items, move_project_to_trash, root::execute_root_batch)
}

pub fn clean_items_with_root_runner<F>(
    items: &[CleanItem],
    move_project_to_trash: bool,
    root_runner: F,
) -> Result<CleanReport, AppError>
where
    F: FnOnce(&str) -> Result<(), AppError>,
{
    let mut report = CleanReport::default();
    let mut root_items = Vec::new();

    for item in items {
        if !item.selected {
            continue;
        }

        if item.requires_root {
            root_items.push(item.clone());
            continue;
        }

        if let Some(ref path_str) = item.path {
            let path = Path::new(path_str);
            if let Err(e) = validate_safe_path(path) {
                report.errors.push(format!("{}: {}", item.id, e));
                continue;
            }

            let result = if move_project_to_trash && item.supports_trash {
                trash::move_to_trash(path)
            } else {
                purge::purge_path(path)
            };

            match result {
                Ok(_) => {
                    report.deleted_bytes += item.size_bytes;
                    report.freed_items.push(item.id.clone());
                }
                Err(e) => {
                    report.errors.push(format!("{}: {}", item.id, e));
                }
            }
        } else {
            // Non-root items without path (e.g. Go global caches via command)
            if item.id.starts_with("dev:go") {
                let status = std::process::Command::new("go")
                    .args(["clean", "-cache", "-modcache"])
                    .status();
                match status {
                    Ok(s) if s.success() => {
                        report.deleted_bytes += item.size_bytes;
                        report.freed_items.push(item.id.clone());
                    }
                    Ok(s) => {
                        report.errors.push(format!(
                            "{}: go clean failed with status {:?}",
                            item.id, s
                        ));
                    }
                    Err(e) => {
                        report.errors.push(format!("{}: failed to run go clean: {}", item.id, e));
                    }
                }
            } else if item.id.starts_with("docker:") {
                let status = std::process::Command::new("docker")
                    .args(["system", "prune", "-f"])
                    .status();
                match status {
                    Ok(s) if s.success() => {
                        report.deleted_bytes += item.size_bytes;
                        report.freed_items.push(item.id.clone());
                    }
                    Ok(s) => {
                        report.errors.push(format!(
                            "{}: docker prune failed with status {:?}",
                            item.id, s
                        ));
                    }
                    Err(e) => {
                        report.errors.push(format!("{}: failed to run docker: {}", item.id, e));
                    }
                }
            }
        }
    }

    if !root_items.is_empty() {
        let script = match root::build_root_batch_script(&root_items) {
            Ok(s) => s,
            Err(e) => {
                report.errors.push(format!("Failed to build root script: {}", e));
                return Ok(report);
            }
        };

        if !script.is_empty() {
            match root_runner(&script) {
                Ok(_) => {
                    for item in root_items {
                        report.deleted_bytes += item.size_bytes;
                        report.freed_items.push(item.id);
                    }
                }
                Err(e) => {
                    report.errors.push(format!("Root cleanup failed: {}", e));
                }
            }
        }
    }

    Ok(report)
}
