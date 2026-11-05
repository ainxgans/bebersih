pub mod cleaner;
pub mod commands;
pub mod error;
pub mod models;
pub mod scanner;

pub use cleaner::*;
pub use commands::*;
pub use error::AppError;
pub use models::*;
pub use scanner::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::scan_all,
            commands::clean_selected,
            commands::get_disk_free_space,
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
