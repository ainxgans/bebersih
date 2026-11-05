export type CleanCategory =
  | "DevRust"
  | "DevGo"
  | "DevNode"
  | "DockerCache"
  | "DockerVolume"
  | "AptCache"
  | "SystemLogs"
  | "StaleCache";

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

export interface CategoryMeta {
  id: CleanCategory;
  label: string;
  group: "Developer" | "Docker" | "System" | "Stale";
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
};
