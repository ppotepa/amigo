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
};
