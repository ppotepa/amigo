import type {
  AssetRegistryDto,
  EditorCommandDto,
  EditorCommandResultDto,
  EditorDiagnosticDto,
  EditorFrameDto,
  EditorFrameResultDto,
  EditorModeDto,
  EditorModeSessionDto,
  EditorPointerEventDto,
  EditorToolDto,
  EditorViewportDto,
  EditorModDetailsDto,
  EditorProjectFileContentDto,
  EditorProjectFileDto,
  EditorProjectStructureNodeDto,
  EditorProjectStructureTreeDto,
  EditorProjectTreeDto,
  EditorSceneEntityDto,
  EditorSceneHierarchyDto,
  EditorSceneSnapshotDto,
  EditorSceneSummaryDto,
  EditorUiNodeDto,
  EditorUiNodeObjectDto,
  ManagedAssetDto,
  ScenePreviewDto,
} from "../api/dto";
import type { EditorSelection } from "../properties/propertiesTypes";
import type { EditorEvent } from "../app/editorEvents";
import type { WindowBusEvent } from "../app/windowBusTypes";
import type { ComponentToolbarState } from "../editor-components/componentTypes";
import type { YamlSourceRef } from "../features/files/yamlSourceRefs";
import type { SceneEditorPreviewSyncState } from "../features/scenes/editor/sceneEditorPreviewSync";
import type { OpenWorkspaceEditorRequest } from "./workspaceOpenTypes";

export type WorkspaceProjectNodeRef = EditorProjectStructureNodeDto | {
  id: string;
  kind: string;
};

export type WorkspaceUiNodeSelectionRef = {
  entityId: string;
  componentIndex: number;
  nodePath: string;
};

export type WorkspaceProjectItemOpenResult = {
  itemKind: string;
  itemId: string;
  selectedFilePath?: string | null;
  selectedSceneId?: string | null;
};

export type WorkspaceRuntimeServices = {
  allProblems?: EditorDiagnosticDto[];
  assetRegistry?: AssetRegistryDto | null;
  details?: EditorModDetailsDto | null;
  eventFilter?: string;
  eventRows?: EditorEvent[];
  eventSearch?: string;
  eventSessionFilter?: string;
  eventSourceFilter?: string;
  editorSnapshot?: EditorSceneSnapshotDto | null;
  editorPreviewSync?: SceneEditorPreviewSyncState;
  editorModeSession?: EditorModeSessionDto | null;
  editorFrame?: EditorFrameDto | null;
  applyEditorCommand?: (command: EditorCommandDto) => Promise<EditorCommandResultDto | null>;
  recordEvent?: (event: EditorEvent) => void;
  openEditorModeSession?: () => Promise<void>;
  closeEditorModeSession?: () => Promise<void>;
  resizeEditorModeViewport?: (viewport: EditorViewportDto) => Promise<EditorFrameResultDto | null>;
  setEditorMode?: (mode: EditorModeDto) => Promise<EditorFrameResultDto | null>;
  setEditorTool?: (tool: EditorToolDto) => Promise<EditorFrameResultDto | null>;
  saveEditorModeSession?: () => Promise<void>;
  discardEditorModeSessionChanges?: () => Promise<void>;
  undoEditorModeTransaction?: () => Promise<void>;
  redoEditorModeTransaction?: () => Promise<void>;
  sendEditorPointerEvent?: (event: EditorPointerEventDto) => Promise<EditorFrameResultDto | null>;
  refreshEditorSnapshot?: () => Promise<void>;
  refreshSceneHierarchy?: () => Promise<void>;
  reloadModDetails?: () => Promise<void>;
  handleSelectProjectFile?: (file: EditorProjectFileDto) => void;
  openWorkspaceEditor?: (request: OpenWorkspaceEditorRequest) => void;
  openProjectItemResult?: (result: WorkspaceProjectItemOpenResult) => Promise<void>;
  openProjectFileEditor?: (file: EditorProjectFileDto) => void;
  openSceneEditor?: (scene: EditorSceneSummaryDto) => Promise<void>;
  openUiDocumentEditor?: (target?: {
    sceneId?: string;
    entityId?: string;
    componentIndex?: number;
    focusPath?: string;
    preferredEntityId?: string;
    initialTemplate?: string;
    titleOverride?: string;
  }) => void;
  showYamlView?: (source: YamlSourceRef) => void;
  openSceneScript?: (scene: EditorSceneSummaryDto) => void;
  handleSelectAsset?: (asset: ManagedAssetDto) => void;
  hierarchy?: EditorSceneHierarchyDto;
  hierarchyTask?: { status: string } | undefined;
  onRevealSelectedFile?: () => void;
  preview?: ScenePreviewDto;
  previewPlaying?: boolean;
  previewTask?: { progress?: number; status: string } | undefined;
  projectTree?: EditorProjectTreeDto;
  projectStructureTree?: EditorProjectStructureTreeDto;
  projectTreeTask?: { status: string } | undefined;
  selection?: EditorSelection;
  selectedAsset?: ManagedAssetDto | null;
  selectedEntity?: EditorSceneEntityDto | null;
  selectedUiNode?: EditorUiNodeDto | null;
  selectedUiNodeObject?: EditorUiNodeObjectDto | null;
  selectedFile?: EditorProjectFileDto | null;
  selectedFileContent?: EditorProjectFileContentDto | null;
  selectedScene?: EditorSceneSummaryDto | null;
  onCreateExpectedFolder?: (expectedPath: string) => Promise<void>;
  onFileDirtyChange?: (path: string, dirty: boolean) => void;
  onProjectTreeRefresh?: () => void | Promise<void>;
  onProjectNodeActivated?: (node: WorkspaceProjectNodeRef) => void;
  activateSceneContext?: (scene: EditorSceneSummaryDto) => Promise<void>;
  selectScene?: (scene: EditorSceneSummaryDto) => Promise<void>;
  selectSceneEntity?: (entityId: string | null) => void;
  selectUiNode?: (selection: WorkspaceUiNodeSelectionRef | null) => void;
  openComponent?: (componentId: string, context?: Record<string, string>) => void;
  setEventFilter?: (filter: string) => void;
  setEventSearch?: (value: string) => void;
  setEventSessionFilter?: (filter: string) => void;
  setEventSourceFilter?: (filter: string) => void;
  tasks?: Array<{ id: string; label: string; status: string; startedAt: number; progress?: number }>;
  toolbarState?: ComponentToolbarState;
  windowEventRows?: WindowBusEvent[];
};
