import { convertFileSrc } from "@tauri-apps/api/core";

export function fileSrc(path: string): string {
  const normalizedPath = path.startsWith("\\\\?\\") ? path.slice(4) : path;
  return convertFileSrc(normalizedPath);
}
