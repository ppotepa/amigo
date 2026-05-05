import type { ReactNode } from "react";
import type {
  AssetRegistryDto,
  EditorModDetailsDto,
  EditorProjectFileDto,
  EditorSceneEntityDto,
  EditorSceneSummaryDto,
  ManagedAssetDto,
} from "../api/dto";

export type AssetSelection = {
  kind: "asset";
  asset: ManagedAssetDto;
  file: EditorProjectFileDto | null;
};

export type ProjectFileSelection = {
  kind: "projectFile";
  file: EditorProjectFileDto;
};

export type EntitySelection = {
  kind: "entity";
  entity: EditorSceneEntityDto;
  scene: EditorSceneSummaryDto | null;
};

export type SceneSelection = {
  kind: "scene";
  scene: EditorSceneSummaryDto;
};

export type ModSelection = {
  kind: "mod";
  details: EditorModDetailsDto | null;
};

export type EmptySelection = {
  kind: "empty";
};

export type EditorSelection =
  | AssetSelection
  | ProjectFileSelection
  | EntitySelection
  | SceneSelection
  | ModSelection
  | EmptySelection;

export type PropertiesContext = {
  assetRegistry: AssetRegistryDto | null;
  assetRegistryError: string | null;
  details: EditorModDetailsDto | null;
  rulesetBusy: boolean;
  rulesetError: string | null;
  sessionId?: string;
  onAddSpritesheetRuleset?: (asset: ManagedAssetDto) => Promise<void>;
  onSelectAsset?: (asset: ManagedAssetDto) => void;
  onSelectFile?: (file: EditorProjectFileDto) => void;
};

export type TypedPropertiesRenderer<TSelection extends EditorSelection> = {
  kind: TSelection["kind"];
  canRender: (selection: EditorSelection) => selection is TSelection;
  render: (selection: TSelection, context: PropertiesContext) => ReactNode;
};

export type PropertiesRenderer = {
  kind: EditorSelection["kind"];
  canRender: (selection: EditorSelection) => boolean;
  render: (selection: EditorSelection, context: PropertiesContext) => ReactNode;
};

export function propertiesRenderer<TSelection extends EditorSelection>(
  renderer: TypedPropertiesRenderer<TSelection>,
): PropertiesRenderer {
  return {
    kind: renderer.kind,
    canRender: renderer.canRender,
    render: (selection, context) => renderer.render(selection as TSelection, context),
  };
}

export function createEditorSelection({
  details,
  selectedAsset,
  selectedEntity,
  selectedFile,
  selectedScene,
}: {
  details: EditorModDetailsDto | null;
  selectedAsset: ManagedAssetDto | null;
  selectedEntity: EditorSceneEntityDto | null;
  selectedFile: EditorProjectFileDto | null;
  selectedScene: EditorSceneSummaryDto | null;
}): EditorSelection {
  if (selectedAsset) return { kind: "asset", asset: selectedAsset, file: selectedFile };
  if (selectedEntity) return { kind: "entity", entity: selectedEntity, scene: selectedScene };
  if (selectedFile) return { kind: "projectFile", file: selectedFile };
  if (selectedScene) return { kind: "scene", scene: selectedScene };
  if (details) return { kind: "mod", details };
  return { kind: "empty" };
}
