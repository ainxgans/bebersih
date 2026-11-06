import { formatBytes } from "./format";

export interface AvailableUpdate {
  version: string;
  currentVersion: string;
  body?: string;
  date?: string;
  downloadAndInstall: (cb: (event: any) => void) => Promise<void>;
}

/**
 * Check for application updates safely without crashing non-Tauri / browser dev environments.
 *
 * ponytail: Basic update check covers GitHub Releases latest.json.
 * Upgrade with staged rollouts or private channels if commercial distribution is needed.
 */
export async function checkForUpdateSafe(): Promise<AvailableUpdate | null> {
  if (typeof window === "undefined" || !(window as any).__TAURI_INTERNALS__) {
    return null;
  }

  try {
    const { check } = await import("@tauri-apps/plugin-updater");
    const update = await check();
    if (!update || !update.available) {
      return null;
    }
    return update as unknown as AvailableUpdate;
  } catch (err) {
    console.warn("Update check skipped or failed:", err);
    return null;
  }
}

/**
 * Download, verify signature, install update, and relaunch app.
 */
export async function installUpdateAndRelaunch(
  update: AvailableUpdate,
  onProgress?: (status: string) => void
): Promise<void> {
  const { relaunch } = await import("@tauri-apps/plugin-process");

  await update.downloadAndInstall((event: any) => {
    switch (event.event) {
      case "Started":
        onProgress?.(
          `Downloading update (${formatBytes(event.data?.contentLength ?? 0)})...`
        );
        break;
      case "Progress":
        onProgress?.("Downloading update...");
        break;
      case "Finished":
        onProgress?.("Download complete. Relaunching application...");
        break;
    }
  });

  await relaunch();
}
