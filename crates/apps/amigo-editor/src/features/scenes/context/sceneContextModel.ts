import {
  selectSceneAssetGroups,
  selectSceneDiagnostics,
  selectSceneEntityNodes,
  selectSceneScripts,
  selectSceneSourceModel,
} from "./sceneContextSelectors";
import type {
  BuildSceneContextModelInput,
  SceneContextModel,
} from "./sceneContextTypes";

export function buildSceneContextModel(input: BuildSceneContextModelInput): SceneContextModel {
  const {
    diagnostics = [],
    entities = [],
    editorObjects = [],
    managedAssets = [],
    projectTreeRoot,
    rawFiles = [],
    scene,
    selectedEntityId,
  } = input;

  return {
    scene,
    status: scene.status,
    selectedObject: editorObjects.find((object) => object.entityId === selectedEntityId) ?? null,
    scripts: selectSceneScripts(projectTreeRoot, scene),
    assetGroups: selectSceneAssetGroups(scene, managedAssets, rawFiles),
    entities: selectSceneEntityNodes(entities, selectedEntityId),
    diagnostics: selectSceneDiagnostics(scene, diagnostics),
    source: selectSceneSourceModel(scene),
  };
}
