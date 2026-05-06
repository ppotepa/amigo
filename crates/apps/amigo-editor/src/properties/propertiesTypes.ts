import type { ReactNode } from "react";
import type {
  AssetRegistryDto,
  EditorModDetailsDto,
  EditorProjectFileDto,
  EditorSceneEntityDto,
  EditorSceneObjectDto,
  EditorSceneSummaryDto,
  EditorUiNodeDto,
  ManagedAssetDto,
} from "../api/dto";
import type { YamlSourceRef } from "../features/files/yamlSourceRefs";

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
  selectedObject?: EditorSceneObjectDto | null;
};

export type UiNodeSelection = {
  kind: "uiNode";
  scene: EditorSceneSummaryDto | null;
  entity: EditorSceneEntityDto;
  node: EditorUiNodeDto;
  nodeRef: {
    entityId: string;
    componentIndex: number;
    nodePath: string;
  };
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
  | UiNodeSelection
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
  onShowYaml?: (source: YamlSourceRef) => void;
};

export type PropertiesRenderer = {
  kind: EditorSelection["kind"];
  canRender: (selection: EditorSelection) => boolean;
  render: (selection: EditorSelection, context: PropertiesContext) => ReactNode;
};

export function propertyPanel<K extends EditorSelection["kind"]>(
  kind: K,
  render: (
    selection: Extract<EditorSelection, { kind: K }>,
    context: PropertiesContext,
  ) => ReactNode,
): PropertiesRenderer {
  return {
    kind,
    canRender: (selection): selection is Extract<EditorSelection, { kind: K }> =>
      selection.kind === kind,
    render: (selection, context) =>
      render(selection as Extract<EditorSelection, { kind: K }>, context),
  };
}
