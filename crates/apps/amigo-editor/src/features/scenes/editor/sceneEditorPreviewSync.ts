export type SceneEditorPreviewSyncStatus =
  | "idle"
  | "regenerating"
  | "ready"
  | "failed";

export type SceneEditorPreviewSyncState = {
  status: SceneEditorPreviewSyncStatus;
  sceneId: string | null;
  revision: number;
  message?: string;
};

export function idleSceneEditorPreviewSync(sceneId: string | null = null): SceneEditorPreviewSyncState {
  return {
    status: "idle",
    sceneId,
    revision: 0,
  };
}

export function sceneEditorPreviewRegenerating({
  sceneId,
  revision,
}: {
  sceneId: string;
  revision: number;
}): SceneEditorPreviewSyncState {
  return {
    status: "regenerating",
    sceneId,
    revision,
    message: "Preview is regenerating from the updated scene document.",
  };
}

export function sceneEditorPreviewReady({
  sceneId,
  revision,
}: {
  sceneId: string;
  revision: number;
}): SceneEditorPreviewSyncState {
  return {
    status: "ready",
    sceneId,
    revision,
    message: "Preview is synchronized with the latest editor snapshot.",
  };
}

export function sceneEditorPreviewFailed({
  sceneId,
  revision,
  message,
}: {
  sceneId: string;
  revision: number;
  message: string;
}): SceneEditorPreviewSyncState {
  return {
    status: "failed",
    sceneId,
    revision,
    message,
  };
}
