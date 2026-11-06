import { CleanItem, CleanReport, CleanupHistory } from "../types";

export const isTauriEnvironment = (): boolean => {
  return (
    typeof window !== "undefined" &&
    Boolean(
      (window as unknown as { __TAURI_INTERNALS__?: unknown })
        .__TAURI_INTERNALS__
    )
  );
};

export async function scanAll(
  customPaths: string[] = []
): Promise<CleanItem[]> {
  if (isTauriEnvironment()) {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<CleanItem[]>("scan_all", { customPaths });
  }

  // Development mock fallback for testing in browser without Tauri engine
  await new Promise((res) => setTimeout(res, 800));
  return [
    {
      id: "mock-1",
      category: "DevRust",
      label: "cargo target (bebersih)",
      path: "/home/user/code/personal/bebersih/src-tauri/target",
      size_bytes: 420 * 1024 * 1024,
      requires_root: false,
      supports_trash: true,
      selected: true,
    },
    {
      id: "mock-2",
      category: "DevNode",
      label: "node_modules (frontend-app)",
      path: "/home/user/code/work/frontend-app/node_modules",
      size_bytes: 650 * 1024 * 1024,
      requires_root: false,
      supports_trash: true,
      selected: true,
    },
    {
      id: "mock-3",
      category: "DevGo",
      label: "Go build cache",
      path: "/home/user/.cache/go-build",
      size_bytes: 180 * 1024 * 1024,
      requires_root: false,
      supports_trash: false,
      selected: true,
    },
    {
      id: "mock-4",
      category: "DockerCache",
      label: "Docker dangling images",
      path: null,
      size_bytes: 1250 * 1024 * 1024,
      requires_root: false,
      supports_trash: false,
      selected: true,
    },
    {
      id: "mock-5",
      category: "DockerVolume",
      label: "Docker unused volume: redis_data_temp",
      path: null,
      size_bytes: 340 * 1024 * 1024,
      requires_root: false,
      supports_trash: false,
      selected: false,
    },
    {
      id: "mock-6",
      category: "AptCache",
      label: "APT package archives",
      path: "/var/cache/apt/archives",
      size_bytes: 512 * 1024 * 1024,
      requires_root: true,
      supports_trash: false,
      selected: true,
    },
    {
      id: "mock-7",
      category: "SystemLogs",
      label: "Systemd journal logs (> 7 days)",
      path: "/var/log/journal",
      size_bytes: 380 * 1024 * 1024,
      requires_root: true,
      supports_trash: false,
      selected: true,
    },
    {
      id: "mock-8",
      category: "StaleCache",
      label: "GNOME Thumbnail cache",
      path: "/home/user/.cache/thumbnails",
      size_bytes: 95 * 1024 * 1024,
      requires_root: false,
      supports_trash: false,
      selected: true,
    },
  ];
}

export async function cleanSelected(
  items: CleanItem[],
  moveProjectToTrash: boolean
): Promise<CleanReport> {
  if (isTauriEnvironment()) {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<CleanReport>("clean_selected", {
      items,
      moveProjectToTrash,
    });
  }

  // Development mock fallback
  await new Promise((res) => setTimeout(res, 1200));
  const totalFreed = items.reduce((acc, it) => acc + it.size_bytes, 0);
  return {
    deleted_bytes: totalFreed,
    freed_items: items.map((i) => i.id),
    errors: [],
  };
}

export async function getDiskFreeSpace(): Promise<number> {
  if (isTauriEnvironment()) {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<number>("get_disk_free_space");
  }

  // Development mock: 48 GB free
  return 48 * 1024 * 1024 * 1024;
}

export async function emptyTrash(): Promise<CleanReport> {
  if (isTauriEnvironment()) {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<CleanReport>("empty_trash");
  }

  // Development mock fallback
  await new Promise((res) => setTimeout(res, 800));
  return {
    deleted_bytes: 120 * 1024 * 1024,
    freed_items: ["trash:mock-item"],
    errors: [],
  };
}

export async function getWhitelist(): Promise<string[]> {
  if (isTauriEnvironment()) {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<string[]>("get_whitelist");
  }
  return ["/home/user/work/my-precious-repo"];
}

export async function addToWhitelist(path: string): Promise<string[]> {
  if (isTauriEnvironment()) {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<string[]>("add_to_whitelist", { path });
  }
  return ["/home/user/work/my-precious-repo", path];
}

export async function removeFromWhitelist(path: string): Promise<string[]> {
  if (isTauriEnvironment()) {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<string[]>("remove_from_whitelist", { path });
  }
  return [];
}

export async function getCleanupHistory(): Promise<CleanupHistory> {
  if (isTauriEnvironment()) {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<CleanupHistory>("get_cleanup_history");
  }
  return {
    total_freed_bytes: 4200 * 1024 * 1024,
    total_clean_runs: 5,
    records: [],
  };
}
