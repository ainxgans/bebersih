export type CleanCategory =
  | "DevRust"
  | "DevGo"
  | "DevNode"
  | "DevPython"
  | "DevJvm"
  | "DockerCache"
  | "DockerVolume"
  | "AptCache"
  | "SystemLogs"
  | "StaleCache"
  | "BrowserCache"
  | "UserTrash"
  | "SnapFlatpak"
  | "IdeCache"
  | "LargeFiles";

export interface CleanItem {
  id: string;
  category: CleanCategory;
  label: string;
  path: string | null;
  size_bytes: number;
  requires_root: boolean;
  supports_trash: boolean;
  selected: boolean;
}

export interface ScanProgress {
  scanned_items: number;
  current_path: string;
  found_bytes: number;
}

export interface CleanReport {
  deleted_bytes: number;
  freed_items: string[];
  errors: string[];
}

export interface CleanupRecord {
  id: string;
  timestamp_epoch_secs: number;
  deleted_bytes: number;
  freed_items_count: number;
  categories: CleanCategory[];
}

export interface CleanupHistory {
  total_freed_bytes: number;
  total_clean_runs: number;
  records: CleanupRecord[];
}

export interface CategoryMeta {
  id: CleanCategory;
  label: string;
  group: "Developer" | "Docker" | "System" | "Stale" | "Browser";
  description: string;
  defaultTrash: boolean;
}

export const CATEGORY_DEFINITIONS: Record<CleanCategory, CategoryMeta> = {
  DevRust: {
    id: "DevRust",
    label: "Rust target/",
    group: "Developer",
    description: "Cargo build artifacts in projects",
    defaultTrash: true,
  },
  DevGo: {
    id: "DevGo",
    label: "Go Build/Mod Cache",
    group: "Developer",
    description: "Go build artifacts & downloaded module caches",
    defaultTrash: false,
  },
  DevNode: {
    id: "DevNode",
    label: "node_modules & npm",
    group: "Developer",
    description: "Node.js dependencies and npm cache",
    defaultTrash: true,
  },
  DevPython: {
    id: "DevPython",
    label: "Python .venv & Cache",
    group: "Developer",
    description: "Virtualenvs, __pycache__, pip, and uv package caches",
    defaultTrash: true,
  },
  DevJvm: {
    id: "DevJvm",
    label: "JVM (Gradle & Maven)",
    group: "Developer",
    description: "Gradle caches, wrapper dists, and Maven local repositories",
    defaultTrash: true,
  },
  DockerCache: {
    id: "DockerCache",
    label: "Docker Build Cache",
    group: "Docker",
    description: "Dangling images and build cache",
    defaultTrash: false,
  },
  DockerVolume: {
    id: "DockerVolume",
    label: "Docker Unused Volumes",
    group: "Docker",
    description: "Unreferenced container volumes (opt-in)",
    defaultTrash: false,
  },
  AptCache: {
    id: "AptCache",
    label: "APT Archives",
    group: "System",
    description: "Cached .deb package files in /var/cache/apt",
    defaultTrash: false,
  },
  SystemLogs: {
    id: "SystemLogs",
    label: "System Logs & Journal",
    group: "System",
    description: "Archived systemd journal logs > 7 days & crash files",
    defaultTrash: false,
  },
  StaleCache: {
    id: "StaleCache",
    label: "Stale User Cache",
    group: "Stale",
    description: "Thumbnails and ~/.cache files older than 30 days",
    defaultTrash: false,
  },
  BrowserCache: {
    id: "BrowserCache",
    label: "Browser Caches",
    group: "Browser",
    description: "Media and asset caches from Chrome, Firefox, Brave, and Chromium",
    defaultTrash: false,
  },
  UserTrash: {
    id: "UserTrash",
    label: "User Trash",
    group: "System",
    description: "Deleted files and folders in ~/.local/share/Trash",
    defaultTrash: false,
  },
  SnapFlatpak: {
    id: "SnapFlatpak",
    label: "Snap & Flatpak Packages",
    group: "System",
    description: "Disabled snap revisions and unused flatpak runtimes",
    defaultTrash: false,
  },
  IdeCache: {
    id: "IdeCache",
    label: "IDE & Editor Caches",
    group: "Developer",
    description: "VS Code, Cursor, and JetBrains IDE caches & cached data",
    defaultTrash: true,
  },
  LargeFiles: {
    id: "LargeFiles",
    label: "Large Stray Files & Archives",
    group: "Stale",
    description: "Old ISO disk images, stray tar/zip archives, and database dumps (>500MB)",
    defaultTrash: true,
  },
};
