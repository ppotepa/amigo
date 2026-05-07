import type {
  EditorProjectFileContentDto,
  EditorModDetailsDto,
  EditorProjectFileDto,
  EditorProjectTreeDto,
  EditorSceneEntityDto,
  EditorSceneHierarchyDto,
  EditorSceneSummaryDto,
  EditorUiNodeDto,
  EditorUiNodeObjectDto,
  ManagedAssetDto,
  ScenePreviewDto,
} from "../../api/dto";
import type { EditorSelection } from "../../properties/propertiesTypes";
import { managedAssetFromProjectFile } from "../../assets/assetProjectFiles";
import { findProjectFile } from "../../features/files/fileTreeSelectors";
import type { EditorState } from "./editorState";
import { previewKey } from "./editorState";
import {
  selectedAssetKey,
  selectedEntityId,
  selectedFilePath,
  selectedModId,
  selectedSceneId,
  selectedUiNode as selectedUiNodeRef,
} from "../selectionSelectors";

export function selectedScene(state: EditorState): EditorSceneSummaryDto | null {
  const details = state.modDetails;
  const modId = selectedModId(state.selection);
  const sceneId = selectedSceneId(state.selection);
  if (details && modId && details.id !== modId) {
    return null;
  }
  return sceneId ? details?.scenes.find((scene) => scene.id === sceneId) ?? null : null;
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
  const entityId = selectedEntityId(state.selection);
  if (!entityId) {
    return null;
  }
  return hierarchy?.entities.find((entity) => entity.id === entityId) ?? null;
}

export function findUiNodeInTree(root: EditorUiNodeDto, nodePath: string): EditorUiNodeDto | null {
  if (root.path === nodePath) {
    return root;
  }

  for (const child of root.children) {
    const match = findUiNodeInTree(child, nodePath);
    if (match) {
      return match;
    }
  }

  return null;
}

export function selectedUiNode(
  state: EditorState,
  hierarchy?: EditorSceneHierarchyDto,
): EditorUiNodeDto | null {
  const selection = selectedUiNodeRef(state.selection);
  if (!selection || !hierarchy) {
    return null;
  }

  const document = hierarchy.uiDocuments.find(
    (candidate) =>
      candidate.entityId === selection.entityId &&
      candidate.componentIndex === selection.componentIndex,
  );

  return document ? findUiNodeInTree(document.root, selection.nodePath) : null;
}

export function selectedUiNodeObject(
  state: EditorState,
  snapshot?: { uiNodes?: EditorUiNodeObjectDto[] } | null,
): EditorUiNodeObjectDto | null {
  const selection = selectedUiNodeRef(state.selection);
  if (!selection || !snapshot?.uiNodes) {
    return null;
  }

  return (
    snapshot.uiNodes.find(
      (candidate) =>
        candidate.entityId === selection.entityId &&
        candidate.componentIndex === selection.componentIndex &&
        candidate.nodePath === selection.nodePath,
    ) ?? null
  );
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
  const uiNode = selectedUiNode(state, hierarchy);
  const asset = selectedAsset(state, projectTree);

  if (asset) {
    return { kind: "asset", asset, file };
  }
  const selection = state.selection;
  if (uiNode && selection.kind === "uiNode") {
    const uiEntity = hierarchy?.entities.find((candidate) => candidate.id === selection.entityId) ?? null;
    if (uiEntity) {
      return {
        kind: "uiNode",
        scene,
        entity: uiEntity,
        node: uiNode,
        nodeRef: {
          entityId: selection.entityId,
          componentIndex: selection.componentIndex,
          nodePath: selection.nodePath,
        },
      };
    }
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
