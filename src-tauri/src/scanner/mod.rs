pub mod browser;
pub mod dev;
pub mod docker;
pub mod ide;
pub mod package;
pub mod stray;
pub mod system;
pub mod trash;

pub use browser::{
    scan_browser_caches, scan_browser_caches_with_config, BrowserScanConfig,
};
pub use dev::{
    scan_dev_caches, scan_dev_caches_with_config, scan_global_jvm_caches,
    scan_global_python_caches, scan_project_dev_caches, scan_project_dev_caches_with_config,
};
pub use docker::{
    parse_docker_df_output, parse_docker_df_summary, parse_docker_df_verbose, parse_docker_size,
    scan_docker_caches, scan_docker_caches_with_config, DockerScanConfig,
};
pub use ide::{scan_ide_caches, scan_ide_caches_with_config, IdeScanConfig};
pub use package::{
    parse_flatpak_unused_output, parse_snap_list_output, scan_snap_and_flatpak,
    scan_snap_and_flatpak_with_config, PackageScanConfig,
};
pub use stray::{
    scan_large_files, scan_large_files_with_config, LargeFilesScanConfig,
    DEFAULT_LARGE_FILE_THRESHOLD_BYTES,
};
pub use system::{scan_system_caches, scan_system_caches_with_config, SystemScanConfig};
pub use trash::{
    empty_user_trash, empty_user_trash_with_config, scan_user_trash, scan_user_trash_with_config,
    TrashScanConfig,
};
