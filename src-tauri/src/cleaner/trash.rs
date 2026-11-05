use std::path::Path;
use std::process::Command;

use crate::cleaner::validate_safe_path;
use crate::error::AppError;

// ponytail: gio trash is standard on GNOME / Ubuntu desktop. Add trash crate or fallback if porting to non-GLib.
pub fn move_to_trash(path: &Path) -> Result<(), AppError> {
    validate_safe_path(path)?;

    if !path.exists() {
        return Ok(());
    }

    let output = Command::new("gio")
        .args(["trash", &path.to_string_lossy()])
        .output()
        .map_err(|e| AppError::CommandFailed(format!("Failed to execute gio trash: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::CommandFailed(format!(
            "gio trash failed for {:?}: {}",
            path,
            stderr.trim()
        )));
    }

    Ok(())
}
