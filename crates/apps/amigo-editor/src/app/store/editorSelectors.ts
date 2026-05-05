import type {
  EditorProjectFileContentDto,
  EditorModDetailsDto,
  EditorProjectFileDto,
  EditorProjectTreeDto,
  EditorSceneEntityDto,
  EditorSceneHierarchyDto,
  EditorSceneSummaryDto,
  ManagedAssetDto,
  ScenePreviewDto,
} from "../../api/dto";
import type { EditorSelection } from "../../properties/propertiesTypes";
import { managedAssetFromProjectFile } from "../../assets/assetProjectFiles";
import { findProjectFile } from "../../features/files/fileTreeSelectors";
import type { EditorState } from "./editorState";
import { previewKey } from "./editorState";
import { selectedAssetKey, selectedFilePath, selectedSceneId } from "../selectionSelectors";

export function selectedScene(state: EditorState): EditorSceneSummaryDto | null {
  const details = state.modDetails;
  const sceneId = selectedSceneId(state.selection);
  return details?.scenes.find((scene) => scene.id === sceneId) ?? details?.scenes[0] ?? null;
}

export function selectedFile(state: EditorState, projectTree?: EditorProjectTreeDto): EditorProjectFileDto | null {
  const path = selectedFilePath(state.selection);
  return projectTree && path ? findProjectFile(projectTree.root, path) : null;
}

export function selectedAsset(state: EditorState, projectTree?: EditorProjectTreeDto): ManagedAssetDto | null {
  const assetKey = selectedAssetKey(state.selection);
  if (!assetKey || !projectTree || state.selection.kind !== "asset") {
    return null;
  }
  const file = state.selection.filePath ? findProjectFile(projectTree.root, state.selection.filePath) : null;
  if (!file) return null;
  const asset = managedAssetFromProjectFile(state.selection.modId, file);
  return asset.assetKey === assetKey ? asset : null;
}

export function activePreview(
  details: EditorModDetailsDto | null,
  sceneId: string | null,
  previews: Record<string, ScenePreviewDto>,
): ScenePreviewDto | undefined {
  if (!details || !sceneId) {
    return undefined;
  }
  return previews[previewKey(details.id, sceneId)];
}

export function selectedHierarchy(
  details: EditorModDetailsDto | null,
  scene: EditorSceneSummaryDto | null,
  sceneHierarchies: Record<string, EditorSceneHierarchyDto>,
): EditorSceneHierarchyDto | undefined {
  return details && scene ? sceneHierarchies[previewKey(details.id, scene.id)] : undefined;
}

export function selectedEntity(
  state: EditorState,
  hierarchy?: EditorSceneHierarchyDto,
): EditorSceneEntityDto | null {
  const entityId = state.selection.kind === "entity" ? state.selection.entityId : null;
  return hierarchy?.entities.find((entity) => entity.id === entityId) ?? hierarchy?.entities[0] ?? null;
}

export function resolvedSelection(
  state: EditorState,
  projectTree?: EditorProjectTreeDto,
  _selectedFileContent?: EditorProjectFileContentDto | null,
): EditorSelection {
  const details = state.modDetails;
  const scene = selectedScene(state);
  const file = selectedFile(state, projectTree);
  const hierarchy = selectedHierarchy(details, scene, state.sceneHierarchies);
  const entity = selectedEntity(state, hierarchy);
  const asset = selectedAsset(state, projectTree);

  if (asset) {
    return { kind: "asset", asset, file };
  }
  if (entity) {
    return { kind: "entity", entity, scene };
  }
  if (file) {
    return { kind: "projectFile", file };
  }
  if (scene) {
    return { kind: "scene", scene };
  }
  if (details) {
    return { kind: "mod", details };
  }
  return { kind: "empty" };
}
