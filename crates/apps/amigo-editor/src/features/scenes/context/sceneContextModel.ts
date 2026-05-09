// Legacy scene context implementation used by features/scene/target adapters.
// New target-facing code should import from features/scene/target/*.
import {
  selectSceneAssetGroups,
  selectSceneComponentsModel,
  selectSceneDiagnostics,
  selectSceneEntityNodes,
  selectSceneEntitiesModel,
  selectSceneHeaderModel,
  selectSceneNavigationModel,
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
    sceneChanges,
  } = input;

  const scripts = selectSceneScripts(projectTreeRoot, scene);
  const assetGroups = selectSceneAssetGroups(scene, managedAssets, rawFiles);
  const sceneEntities = selectSceneEntityNodes(entities, selectedEntityId);
  const sceneDiagnostics = selectSceneDiagnostics(scene, diagnostics);
  const source = selectSceneSourceModel(scene);
  const header = selectSceneHeaderModel(scene, sceneDiagnostics, sceneChanges ? {
    dirty: sceneChanges.dirty,
    summary: "Changes tracked in workspace state.",
  } : null);
  const navigation = selectSceneNavigationModel(scene, scripts, source, sceneEntities);
  const components = selectSceneComponentsModel(scene.id, sceneEntities, sceneDiagnostics);
  const entitiesInfo = selectSceneEntitiesModel(sceneEntities, sceneDiagnostics);

  return {
    scene,
    status: scene.status,
    selectedObject: editorObjects.find((object) => object.entityId === selectedEntityId) ?? null,
    scripts,
    assetGroups,
    entities: sceneEntities,
    diagnostics: sceneDiagnostics,
    source,
    header,
    navigation,
    components,
    entitiesInfo,
    sceneInfo: {
      status: scene.status,
      launcherVisible: scene.launcherVisible,
      assetGroups,
      assetCount: assetGroups.reduce((sum, group) => sum + group.count, 0),
      scriptCount: scripts.length,
      entityCount: sceneEntities.length,
    },
    fileInfo: {
      source,
    },
    diagnosticsInfo: {
      diagnostics: sceneDiagnostics,
      errorCount: sceneDiagnostics.filter((diagnostic) => diagnostic.level === "error").length,
      warningCount: sceneDiagnostics.filter((diagnostic) => diagnostic.level === "warning").length,
    },
    changes: {
      dirty: sceneChanges?.dirty ?? false,
      summary: sceneChanges?.dirty ? "Scene has unsaved changes." : "No unsaved scene changes.",
    },
  };
}
