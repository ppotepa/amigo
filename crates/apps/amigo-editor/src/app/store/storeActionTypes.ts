import type {
  CreateModProjectRequestDto,
  CreateModProjectResultDto,
  EditorProjectFileDto,
  EditorSceneSummaryDto,
  ManagedAssetDto,
} from "../../api/dto";
import type { EditorEvent } from "../editorEvents";
import type { EditorUiNodeSelectionRef } from "../selectionTypes";
import type { EditorState } from "./editorState";
import type { EditorComponentInstance } from "../../editor-components/componentTypes";
import type { WorkspaceDockProfileId } from "../../main-window/workspaceDockProfiles";
import type { WorkspaceDockLayoutState } from "../../main-window/workspaceLayout";

export interface EditorStoreValue {
  state: EditorState;
  scanMods: () => Promise<void>;
  createModProject: (request: CreateModProjectRequestDto) => Promise<CreateModProjectResultDto>;
  deleteModProject: (modId: string) => Promise<void>;
  selectMod: (modId: string) => Promise<void>;
  loadProjectTree: (modId: string) => Promise<void>;
  refreshProjectTree: (modId: string) => Promise<void>;
  loadEditorSession: (sessionId: string) => Promise<void>;
  selectScene: (scene: EditorSceneSummaryDto, workspaceId?: string) => Promise<void>;
  selectSceneEntity: (entityId: string | null, workspaceId?: string) => void;
  selectUiNode: (selection: Omit<EditorUiNodeSelectionRef, "kind" | "modId" | "sceneId"> | null, workspaceId?: string) => void;
  selectAsset: (asset: ManagedAssetDto | null, workspaceId?: string) => void;
  selectProjectFile: (file: EditorProjectFileDto, workspaceId?: string) => void;
  selectWorkspaceTab: (tabId: string, workspaceId?: string) => void;
  closeWorkspaceTab: (tabId: string, workspaceId?: string) => void;
  markWorkspaceTabDetached: (sourceWorkspaceId: string, tabId: string, detachedWorkspaceId: string) => void;
  setWorkspaceDockProfile: (workspaceId: string, dockProfileId: WorkspaceDockProfileId) => void;
  setWorkspaceDockLayout: (workspaceId: string, dockLayout: WorkspaceDockLayoutState) => void;
  openCenterComponentTab: (instance: EditorComponentInstance, workspaceId?: string) => void;
  closeCenterComponentTab: (instanceId: string, workspaceId?: string) => void;
  openComponent: (componentId: string, context?: Record<string, string>) => void;
  focusComponent: (instanceId: string, componentId: string) => void;
  moveComponent: (instanceId: string, placement: string) => void;
  closeComponent: (instanceId: string, componentId: string) => void;
  revealSelectedProjectFile: () => Promise<void>;
  createExpectedFolder: (expectedPath: string) => Promise<void>;
  loadSceneHierarchy: (modId: string, sceneId: string, force?: boolean) => Promise<void>;
  loadEditorModeSceneHierarchy: (sessionId: string, editorModeSessionId: string) => Promise<void>;
  regeneratePreview: (modId: string, sceneId: string, forceRegenerate?: boolean) => Promise<void>;
  validateSelectedMod: () => Promise<void>;
  revealSelectedModFolder: () => Promise<void>;
  revealSelectedSceneDocument: () => Promise<void>;
  openSelectedMod: (modIdOverride?: string) => Promise<void>;
  recordEvent: (event: EditorEvent) => void;
  returnToStartup: () => Promise<void>;
  toggleInspectorSection: (sectionId: string) => void;
  setPreviewPlaying: (playing: boolean) => void;
  setContentFilter: (filter: string | null) => void;
  setFileDirty: (path: string, dirty: boolean) => void;
}
