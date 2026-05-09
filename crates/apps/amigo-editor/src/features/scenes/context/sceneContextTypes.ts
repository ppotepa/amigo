// Legacy scene context implementation used by features/scene/target adapters.
// New target-facing code should import from features/scene/target/*.
import type {
  EditorDiagnosticDto,
  EditorProjectFileDto,
  EditorSceneEntityDto,
  EditorSceneObjectDto,
  EditorSceneSummaryDto,
  ManagedAssetDto,
  RawAssetFileDto,
} from "../../../api/dto";
import type { ScriptSourceRef } from "../../files/scriptSourceRefs";
import type { YamlSourceRef } from "../../files/yamlSourceRefs";
import type { EditorTargetRef } from "../../../editor-targets";

export type SceneScriptRole =
  | "primary"
  | "component"
  | "mod"
  | "package"
  | "related";

export type SceneScriptRef = {
  id: string;
  role: SceneScriptRole;
  file: EditorProjectFileDto;
};

export type SceneAssetGroupId =
  | "spritesheet"
  | "tilemap"
  | "audio"
  | "font"
  | "scene"
  | "script"
  | "raw"
  | "other";

export type SceneAssetGroup = {
  id: string;
  label: string;
  count: number;
  managedAssets: ManagedAssetDto[];
  rawFiles: RawAssetFileDto[];
};

export type SceneEntityNode = {
  entity: EditorSceneEntityDto;
  selected: boolean;
};

export type SceneSourceModel = {
  yaml: YamlSourceRef | null;
  script: ScriptSourceRef | null;
  folderPath: string | null;
};

export type SceneContextModel = {
  scene: EditorSceneSummaryDto;
  status: string;
  selectedObject: EditorSceneObjectDto | null;
  scripts: SceneScriptRef[];
  assetGroups: SceneAssetGroup[];
  entities: SceneEntityNode[];
  diagnostics: EditorDiagnosticDto[];
  source: SceneSourceModel;
  header: SceneHeaderModel;
  navigation: SceneNavigationModel;
  components: SceneComponentsModel;
  entitiesInfo: SceneEntitiesModel;
  sceneInfo: SceneInfoModel;
  fileInfo: SceneFileInfoModel;
  diagnosticsInfo: SceneDiagnosticsModel;
  changes: SceneChangesModel;
};

export type SceneHeaderModel = {
  scene: EditorSceneSummaryDto;
  status: "ok" | "warning" | "error" | "neutral";
  displayName: string;
  canRename: boolean;
  badges: SceneHeaderBadge[];
  foldedHint: string;
};

export type SceneHeaderBadge = {
  id: string;
  label: string;
  tone: "ok" | "warning" | "error" | "info" | "neutral";
};

export type SceneNavigationModel = {
  incoming: SceneNavigationLink[];
  outgoing: SceneNavigationLink[];
  entries: SceneNavigationLink[];
  triggers: SceneNavigationLink[];
  foldedHint: string;
  entryScript: SceneScriptRef | null;
  scripts: SceneScriptRef[];
  yaml: YamlSourceRef | null;
};

export type SceneNavigationLink = {
  id: string;
  label: string;
  subtitle?: string;
  targetSceneId?: string;
  targetEntityId?: string;
};

export type SceneComponentTreeItem = {
  id: string;
  label: string;
  typeName: string;
  componentIndex: number;
  ownerKind: "scene" | "entity";
  entityId?: string;
  summary?: string;
  status: "ok" | "warning" | "error" | "neutral";
  target: EditorTargetRef;
};

export type SceneComponentGroup = {
  id: string;
  label: string;
  count: number;
  items: SceneComponentTreeItem[];
  status?: "ok" | "warning" | "error" | "info" | "neutral";
};

export type SceneComponentsModel = {
  groups: SceneComponentGroup[];
  total: number;
  warningCount: number;
  foldedHint: string;
};

export type SceneEntitiesModel = {
  entities: SceneEntityNode[];
  total: number;
  visibleCount: number;
  warningCount: number;
  groups: SceneEntityGroup[];
};

export type SceneEntityGroup = {
  id: string;
  label: string;
  count: number;
  entityIds: string[];
};

export type SceneInfoModel = {
  status: string;
  launcherVisible: boolean;
  assetGroups: SceneAssetGroup[];
  assetCount: number;
  scriptCount: number;
  entityCount: number;
};

export type SceneFileInfoModel = {
  source: SceneSourceModel;
};

export type SceneDiagnosticsModel = {
  diagnostics: EditorDiagnosticDto[];
  errorCount: number;
  warningCount: number;
};

export type SceneChangesModel = {
  dirty: boolean;
  summary: string;
};

export type BuildSceneContextModelInput = {
  scene: EditorSceneSummaryDto;
  projectTreeRoot?: EditorProjectFileDto;
  selectedEntityId?: string | null;
  editorObjects?: EditorSceneObjectDto[];
  entities?: EditorSceneEntityDto[];
  diagnostics?: EditorDiagnosticDto[];
  managedAssets?: ManagedAssetDto[];
  rawFiles?: RawAssetFileDto[];
  sceneChanges?: { dirty: boolean } | null;
};
