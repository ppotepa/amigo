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
  | { type: "ProjectTreeRequested"; modId: string }
  | { type: "ProjectTreeLoaded"; modId: string; fileCount: number }
  | { type: "ProjectTreeFailed"; modId: string; error: string }
  | { type: "ProjectFileSelected"; modId: string; path: string; kind: string }
  | { type: "ProjectFileReadRequested"; modId: string; path: string }
  | { type: "ProjectFileReadCompleted"; modId: string; path: string }
  | { type: "ProjectFileReadFailed"; modId: string; path: string; error: string }
  | { type: "ProjectFileRevealRequested"; modId: string; path: string }
  | { type: "ProjectFileRevealCompleted"; modId: string; path: string }
  | { type: "ProjectFileRevealFailed"; modId: string; path: string; error: string }
  | { type: "ProjectTreeNodeActivated"; modId: string; nodeId: string; kind: string }
  | { type: "ExpectedProjectFolderCreateRequested"; modId: string; expectedPath: string }
  | { type: "ExpectedProjectFolderCreateCompleted"; modId: string; path: string }
  | { type: "ExpectedProjectFolderCreateFailed"; modId: string; expectedPath: string; error: string }
  | { type: "SceneSelected"; modId: string; sceneId: string }
  | { type: "SceneHierarchyRequested"; modId: string; sceneId: string }
  | { type: "SceneHierarchyLoaded"; modId: string; sceneId: string; entityCount: number }
  | { type: "SceneHierarchyFailed"; modId: string; sceneId: string; error: string }
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
  | { type: "MainEditorWindowRequested"; sessionId: string; modId: string }
  | { type: "EditorSessionLoaded"; sessionId: string; modId: string }
  | { type: "EditorSessionClosed"; sessionId: string }
  | { type: "DockLayoutLoaded"; layoutId: "default" }
  | { type: "WorkspaceReady"; sessionId: string }
  | { type: "DockTabSelected"; dock: "left" | "right" | "bottom"; tabId: string }
  | { type: "WorkspaceTabSelected"; tabId: string }
  | { type: "WorkspaceTabOpened"; tabId: string; resourcePath: string }
  | { type: "WorkspaceTabClosed"; tabId: string }
  | { type: "ComponentOpenRequested"; componentId: string; context?: Record<string, string> }
  | { type: "ComponentOpened"; instanceId: string; componentId: string }
  | { type: "ComponentFocused"; instanceId: string; componentId: string }
  | { type: "ComponentMoved"; instanceId: string; placement: string }
  | { type: "ComponentClosed"; instanceId: string; componentId: string }
  | { type: "FileDirtyStateChanged"; path: string; dirty: boolean }
  | { type: "WorkspaceCloseBlocked"; dirtyFileCount: number }
  | { type: "WorkspaceCloseConfirmed" }
  | { type: "LayoutResetRequested" }
  | { type: "InspectorContextChanged"; contextKind: "mod" | "scene" | "entity" | "asset" | "file"; id: string }
  | { type: "ThemeControllerOpened" }
  | {
      type: "ThemePreviewStarted";
      themeId: "mexico-sand" | "mexico-at-night" | "amigo-light-paper";
    }
  | { type: "ThemePreviewCancelled" }
  | {
      type: "ThemeApplyRequested";
      themeId: "mexico-sand" | "mexico-at-night" | "amigo-light-paper";
    }
  | {
      type: "ThemeApplied";
      themeId: "mexico-sand" | "mexico-at-night" | "amigo-light-paper";
    }
  | {
      type: "ThemeApplyFailed";
      themeId: "mexico-sand" | "mexico-at-night" | "amigo-light-paper";
      error: string;
    };
