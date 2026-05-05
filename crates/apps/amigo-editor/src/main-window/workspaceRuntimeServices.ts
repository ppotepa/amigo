import type {
  AssetRegistryDto,
  EditorCommandDto,
  EditorCommandResultDto,
  EditorDiagnosticDto,
  EditorLiveCommandResultDto,
  EditorLiveSceneSessionDto,
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
  EditorTransform2Dto,
  ManagedAssetDto,
  ScenePreviewDto,
} from "../api/dto";
import type { EditorSelection } from "../properties/propertiesTypes";
import type { WindowBusEvent } from "../app/windowBusTypes";
import type { ComponentToolbarState } from "../editor-components/componentTypes";
import type { YamlSourceRef } from "../features/files/yamlSourceRefs";
import type { SceneEditorPreviewSyncState } from "../features/scenes/editor/sceneEditorPreviewSync";
import type { SceneEditorModeKind } from "../features/scenes/editor/sceneEditorMode";

export type WorkspaceProjectNodeRef = EditorProjectStructureNodeDto | {
  id: string;
  kind: string;
};

export type WorkspaceRuntimeServices = {
  allProblems?: EditorDiagnosticDto[];
  assetRegistry?: AssetRegistryDto | null;
  details?: EditorModDetailsDto | null;
  eventFilter?: string;
  eventRows?: Array<{ type: string }>;
  eventSearch?: string;
  eventSessionFilter?: string;
  eventSourceFilter?: string;
  editorSnapshot?: EditorSceneSnapshotDto | null;
  editorPreviewSync?: SceneEditorPreviewSyncState;
  sceneEditorMode?: SceneEditorModeKind;
  setSceneEditorMode?: (mode: SceneEditorModeKind) => void;
  editorLiveSession?: EditorLiveSceneSessionDto | null;
  editorLiveSessionOpening?: boolean;
  editorLiveError?: string | null;
  openEditorLiveSession?: () => Promise<void>;
  closeEditorLiveSession?: () => Promise<void>;
  commitEditorLiveSession?: () => Promise<void>;
  discardEditorLiveSession?: () => Promise<void>;
  applyEditorCommand?: (command: EditorCommandDto) => Promise<EditorCommandResultDto | null>;
  applyEditorLiveTransform?: (entityId: string, transform: EditorTransform2Dto) => Promise<EditorLiveCommandResultDto | null>;
  refreshEditorSnapshot?: () => Promise<void>;
  handleSelectProjectFile?: (file: EditorProjectFileDto) => void;
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
  selectedFile?: EditorProjectFileDto | null;
  selectedFileContent?: EditorProjectFileContentDto | null;
  selectedScene?: EditorSceneSummaryDto | null;
  onCreateExpectedFolder?: (expectedPath: string) => Promise<void>;
  onFileDirtyChange?: (path: string, dirty: boolean) => void;
  onProjectTreeRefresh?: () => void;
  onProjectNodeActivated?: (node: WorkspaceProjectNodeRef) => void;
  activateSceneContext?: (scene: EditorSceneSummaryDto) => Promise<void>;
  selectScene?: (scene: EditorSceneSummaryDto) => Promise<void>;
  selectSceneEntity?: (entityId: string) => void;
  setEventFilter?: (filter: string) => void;
  setEventSearch?: (value: string) => void;
  setEventSessionFilter?: (filter: string) => void;
  setEventSourceFilter?: (filter: string) => void;
  tasks?: Array<{ id: string; label: string; status: string; startedAt: number; progress?: number }>;
  toolbarState?: ComponentToolbarState;
  windowEventRows?: WindowBusEvent[];
};
