use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CleanCategory {
    DevRust,
    DevGo,
    DevNode,
    DockerCache,
    DockerVolume,
    AptCache,
    SystemLogs,
    StaleCache,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanItem {
    pub id: String,
    pub category: CleanCategory,
    pub label: String,
    pub path: Option<String>,
    pub size_bytes: u64,
    pub requires_root: bool,
    pub supports_trash: bool,
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ScanProgress {
    pub scanned_items: usize,
    pub current_path: String,
    pub found_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CleanReport {
    pub deleted_bytes: u64,
    pub freed_items: Vec<String>,
    pub errors: Vec<String>,
}
