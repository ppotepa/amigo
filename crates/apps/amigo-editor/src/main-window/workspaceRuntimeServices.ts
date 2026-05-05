import type {
  AssetRegistryDto,
  EditorDiagnosticDto,
  EditorModDetailsDto,
  EditorProjectFileContentDto,
  EditorProjectFileDto,
  EditorProjectStructureNodeDto,
  EditorProjectStructureTreeDto,
  EditorProjectTreeDto,
  EditorSceneEntityDto,
  EditorSceneHierarchyDto,
  EditorSceneSummaryDto,
  ManagedAssetDto,
  ScenePreviewDto,
} from "../api/dto";
import type { EditorSelection } from "../properties/propertiesTypes";
import type { WindowBusEvent } from "../app/windowBusTypes";
import type { ComponentToolbarState } from "../editor-components/componentTypes";

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
  handleSelectProjectFile?: (file: EditorProjectFileDto) => void;
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
