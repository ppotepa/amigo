export type EditorEvent =
  | { type: "StartupDialogOpened" }
  | { type: "ModsScanRequested" }
  | { type: "ModsScanStarted" }
  | { type: "ModsScanCompleted"; modCount: number }
  | { type: "ModsScanFailed"; error: string }
  | { type: "ModSelected"; modId: string }
  | { type: "ModDetailsRequested"; modId: string }
  | { type: "ModDetailsLoaded"; modId: string }
  | { type: "ModDetailsFailed"; modId: string; error: string }
  | { type: "SceneSelected"; modId: string; sceneId: string }
  | { type: "ScenePreviewRequested"; modId: string; sceneId: string }
  | { type: "ScenePreviewStarted"; modId: string; sceneId: string }
  | { type: "ScenePreviewFrameGenerated"; modId: string; sceneId: string; current: number; total: number }
  | { type: "ScenePreviewCompleted"; modId: string; sceneId: string }
  | { type: "ScenePreviewFailed"; modId: string; sceneId: string; error: string }
  | { type: "PreviewPlaybackToggled"; playing: boolean }
  | { type: "InspectorSectionToggled"; sectionId: string }
  | { type: "ContentFilterChanged"; filter: string | null }
  | { type: "ModValidationRequested"; modId: string }
  | { type: "ModValidationCompleted"; modId: string }
  | { type: "ModValidationFailed"; modId: string; error: string }
  | { type: "RevealPathRequested"; pathKind: "mod" | "scene"; modId: string; sceneId?: string }
  | { type: "RevealPathCompleted"; pathKind: "mod" | "scene"; path: string }
  | { type: "RevealPathFailed"; pathKind: "mod" | "scene"; error: string }
  | { type: "OpenModRequested"; modId: string }
  | { type: "OpenModCompleted"; modId: string; sessionId: string }
  | { type: "OpenModFailed"; modId: string; error: string }
  | { type: "ThemeControllerOpened" }
  | {
      type: "ThemePreviewStarted";
      themeId: "amigo-dark-navy" | "amigo-light-paper" | "amigo-mexico" | "amigo-mexico-dark";
    }
  | { type: "ThemePreviewCancelled" }
  | {
      type: "ThemeApplyRequested";
      themeId: "amigo-dark-navy" | "amigo-light-paper" | "amigo-mexico" | "amigo-mexico-dark";
    }
  | {
      type: "ThemeApplied";
      themeId: "amigo-dark-navy" | "amigo-light-paper" | "amigo-mexico" | "amigo-mexico-dark";
    }
  | {
      type: "ThemeApplyFailed";
      themeId: "amigo-dark-navy" | "amigo-light-paper" | "amigo-mexico" | "amigo-mexico-dark";
      error: string;
    };
