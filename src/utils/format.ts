export function formatBytes(bytes: number, decimals = 1): string {
  if (bytes === 0) return "0 B";
  if (bytes < 0 || !Number.isFinite(bytes)) return "0 B";

  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ["B", "KB", "MB", "GB", "TB"];

  const i = Math.floor(Math.log(bytes) / Math.log(k));
  const safeI = Math.min(i, sizes.length - 1);
  const value = (bytes / Math.pow(k, safeI)).toFixed(dm);

  return `${value} ${sizes[safeI]}`;
}

export function truncatePath(path: string | null, maxLength = 60): string {
  if (!path) return "-";
  if (path.length <= maxLength) return path;

  const parts = path.split("/");
  if (parts.length <= 3) return path;

  const start = parts.slice(0, 2).join("/");
  const end = parts.slice(-2).join("/");
  return `${start}/.../${end}`;
}
