pub mod cleaner;
pub mod commands;
pub mod error;
pub mod history;
pub mod models;
pub mod scanner;
pub mod whitelist;

pub use cleaner::{
    build_root_batch_script, clean_items, clean_items_with_root_runner,
    clean_items_with_whitelist, execute_root_batch, generate_root_batch_script,
    move_to_trash, purge_path, validate_safe_path,
};
pub use commands::*;
pub use error::AppError;
pub use history::{
    get_cleanup_stats, load_history_from_file, record_cleanup_to_file, save_history_to_file,
    CleanupHistory, CleanupRecord, CleanupStats,
};
pub use models::*;
pub use scanner::{
    empty_user_trash, empty_user_trash_with_config, parse_docker_df_output,
    parse_docker_df_summary, parse_docker_df_verbose, parse_docker_size,
    scan_browser_caches, scan_browser_caches_with_config, scan_dev_caches,
    scan_dev_caches_with_config, scan_docker_caches, scan_docker_caches_with_config,
    scan_global_jvm_caches, scan_global_python_caches, scan_ide_caches,
    scan_ide_caches_with_config, scan_large_files, scan_large_files_with_config,
    scan_project_dev_caches, scan_project_dev_caches_with_config, scan_snap_and_flatpak,
    scan_snap_and_flatpak_with_config, scan_system_caches, scan_system_caches_with_config,
    scan_user_trash, scan_user_trash_with_config, BrowserScanConfig, DockerScanConfig,
    IdeScanConfig, LargeFilesScanConfig, PackageScanConfig, SystemScanConfig, TrashScanConfig,
};
pub use whitelist::{
    is_path_ignored_or_whitelisted, load_whitelist_from_file, save_whitelist_to_file,
    WhitelistConfig,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::scan_all,
            commands::clean_selected,
            commands::get_disk_free_space,
            commands::empty_trash,
            commands::get_whitelist,
            commands::add_to_whitelist,
            commands::remove_from_whitelist,
            commands::get_cleanup_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_tauri_context_generates() {
        let _context: tauri::Context<tauri::Wry> = tauri::generate_context!();
    }
}
