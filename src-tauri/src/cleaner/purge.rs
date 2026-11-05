use std::fs;
use std::path::Path;

use crate::cleaner::validate_safe_path;
use crate::error::AppError;

pub fn purge_path(path: &Path) -> Result<(), AppError> {
    validate_safe_path(path)?;

    if !path.exists() {
        return Ok(());
    }

    if path.is_file() || path.is_symlink() {
        fs::remove_file(path).map_err(AppError::from)?;
    } else if path.is_dir() {
        // ponytail: remove_dir_all directly. If permission denied on read-only files (e.g. go mod cache), fix write perms.
        if let Err(err) = fs::remove_dir_all(path) {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                for entry in jwalk::WalkDir::new(path).skip_hidden(false).into_iter().flatten() {
                    if let Ok(meta) = entry.metadata() {
                        let mut perms = meta.permissions();
                        let mode = perms.mode();
                        if mode & 0o200 == 0 {
                            perms.set_mode(mode | 0o700);
                            let _ = fs::set_permissions(entry.path(), perms);
                        }
                    }
                }
                fs::remove_dir_all(path).map_err(|_| AppError::from(err))?;
            }
            #[cfg(not(unix))]
            {
                return Err(AppError::from(err));
            }
        }
    }

    Ok(())
}
