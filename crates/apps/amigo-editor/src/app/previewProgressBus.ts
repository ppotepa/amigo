import { listen } from "@tauri-apps/api/event";

export const PREVIEW_PROGRESS_EVENT = "preview-progress";

export interface PreviewProgressPayload {
  modId: string;
  sceneId: string;
  current: number;
  total: number;
  phase: string;
}

export function listenPreviewProgress(
  handler: (payload: PreviewProgressPayload) => void,
): Promise<() => void> {
  return listen<PreviewProgressPayload>(PREVIEW_PROGRESS_EVENT, (event) => {
    handler(event.payload);
  });
}
