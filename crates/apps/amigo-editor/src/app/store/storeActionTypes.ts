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

export interface EditorStoreValue {
  state: EditorState;
  scanMods: () => Promise<void>;
  createModProject: (request: CreateModProjectRequestDto) => Promise<CreateModProjectResultDto>;
  deleteModProject: (modId: string) => Promise<void>;
  selectMod: (modId: string) => Promise<void>;
  loadProjectTree: (modId: string) => Promise<void>;
  refreshProjectTree: (modId: string) => Promise<void>;
  loadEditorSession: (sessionId: string) => Promise<void>;
  selectScene: (scene: EditorSceneSummaryDto) => Promise<void>;
  selectSceneEntity: (entityId: string | null) => void;
  selectUiNode: (selection: Omit<EditorUiNodeSelectionRef, "kind" | "modId" | "sceneId"> | null) => void;
  selectAsset: (asset: ManagedAssetDto | null) => void;
  selectProjectFile: (file: EditorProjectFileDto) => void;
  selectWorkspaceTab: (tabId: string) => void;
  closeWorkspaceTab: (tabId: string) => void;
  openComponent: (componentId: string, context?: Record<string, string>) => void;
  focusComponent: (instanceId: string, componentId: string) => void;
  moveComponent: (instanceId: string, placement: string) => void;
  closeComponent: (instanceId: string, componentId: string) => void;
  revealSelectedProjectFile: () => Promise<void>;
  createExpectedFolder: (expectedPath: string) => Promise<void>;
  loadSceneHierarchy: (modId: string, sceneId: string, force?: boolean) => Promise<void>;
  regeneratePreview: (modId: string, sceneId: string, forceRegenerate?: boolean) => Promise<void>;
  validateSelectedMod: () => Promise<void>;
  revealSelectedModFolder: () => Promise<void>;
  revealSelectedSceneDocument: () => Promise<void>;
  openSelectedMod: () => Promise<void>;
  recordEvent: (event: EditorEvent) => void;
  returnToStartup: () => Promise<void>;
  toggleInspectorSection: (sectionId: string) => void;
  setPreviewPlaying: (playing: boolean) => void;
  setContentFilter: (filter: string | null) => void;
  setFileDirty: (path: string, dirty: boolean) => void;
}
