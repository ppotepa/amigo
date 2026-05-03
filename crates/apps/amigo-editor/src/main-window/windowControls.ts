import { getCurrentWindow } from "@tauri-apps/api/window";
import { emitWindowCloseRequested, emitWorkspaceClosed } from "../app/windowBus";

export async function toggleFullscreenWindow(): Promise<void> {
  const currentWindow = getCurrentWindow();
  const fullscreen = await currentWindow.isFullscreen();
  await currentWindow.setFullscreen(!fullscreen);
}

export async function closeCurrentWindow(sessionId?: string | null): Promise<void> {
  await emitWindowCloseRequested(sessionId);
  if (sessionId) {
    await emitWorkspaceClosed(sessionId);
  }
  await getCurrentWindow().destroy();
}
