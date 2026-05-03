import { getCurrentWindow } from "@tauri-apps/api/window";

export async function toggleFullscreenWindow(): Promise<void> {
  const currentWindow = getCurrentWindow();
  const fullscreen = await currentWindow.isFullscreen();
  await currentWindow.setFullscreen(!fullscreen);
}

export async function closeCurrentWindow(): Promise<void> {
  await getCurrentWindow().close();
}
