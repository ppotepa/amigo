import { useEffect, useMemo, useState } from "react";
import type { AssetRegistryDto } from "../../../api/dto";
import { getAssetRegistry } from "../../../api/editorApi";
import type { EditorComponentProps } from "../../../editor-components/componentTypes";
import { assetToTarget } from "../../../editor-targets/adapters/assetTargetAdapter";
import { projectFileToTarget } from "../../../editor-targets/adapters/fileTargetAdapter";
import { sceneEntityIdToTarget } from "../../../editor-targets/adapters/sceneTargetAdapter";
import type { WorkspaceRuntimeServices } from "../../../main-window/workspaceRuntimeServices";
import { ContextDock } from "../../../ui/context-dock/ContextDock";
import { EntityTransformWidget } from "../editor/EntityTransformWidget";
import { SelectedEntityWidget } from "../editor/SelectedEntityWidget";
import { resolveSceneScriptFile, resolveSceneYamlFile } from "./sceneContextActions";
import { buildSceneContextModel } from "./sceneContextModel";
import { SceneAssetsWidget } from "./SceneAssetsWidget";
import { SceneDiagnosticsWidget } from "./SceneDiagnosticsWidget";
import { SceneEntitiesWidget } from "./SceneEntitiesWidget";
import { SceneScriptsWidget } from "./SceneScriptsWidget";
import { SceneSourceWidget } from "./SceneSourceWidget";
import { SceneSummaryWidget } from "./SceneSummaryWidget";

export function SceneContextContent({
  context,
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  const [registry, setRegistry] = useState<AssetRegistryDto | null>(services.assetRegistry ?? null);
  const [registryError, setRegistryError] = useState<string | null>(null);

  const scene = services.selectedScene ?? null;
  const sessionId = context.sessionId ?? undefined;

  useEffect(() => {
    if (services.assetRegistry && services.assetRegistry.modId === services.details?.id) {
      setRegistry(services.assetRegistry);
      setRegistryError(null);
      return;
    }

    if (!sessionId || !scene) {
      setRegistry(null);
      setRegistryError(null);
      return;
    }

    let alive = true;
    void getAssetRegistry(sessionId)
      .then((next) => {
        if (!alive) return;
        setRegistry(next.modId === services.details?.id ? next : null);
        setRegistryError(null);
      })
      .catch((reason: unknown) => {
        if (!alive) return;
        setRegistryError(reason instanceof Error ? reason.message : String(reason));
      });

    return () => {
      alive = false;
    };
  }, [services.assetRegistry, services.details?.id, sessionId, scene?.id]);

  const model = useMemo(() => {
    if (!scene) return null;
    return buildSceneContextModel({
      scene,
      projectTreeRoot: services.projectTree?.root,
      selectedEntityId: services.selectedEntity?.id ?? null,
      editorObjects: services.editorSnapshot?.objects ?? [],
      entities: services.hierarchy?.entities ?? [],
      diagnostics: services.hierarchy?.diagnostics ?? [],
      managedAssets: registry?.managedAssets ?? [],
      rawFiles: registry?.rawFiles ?? [],
    });
  }, [
    registry,
    scene,
    services.editorSnapshot?.objects,
    services.hierarchy,
    services.projectTree?.root,
    services.selectedEntity?.id,
  ]);

  if (!scene || !model) {
    return (
      <ContextDock empty={<p className="muted workspace-empty">Select a scene to inspect its context.</p>}>
        {null}
      </ContextDock>
    );
  }

  const yamlFile = resolveSceneYamlFile(services.projectTree, scene);
  const scriptFile = resolveSceneScriptFile(services.projectTree, scene);

  function showYaml() {
    if (model?.source.yaml) {
      services.targetBridge?.showYamlView?.(model.source.yaml);
    }
  }

  function openScript() {
    if (scriptFile) {
      services.activateEditorTarget?.(projectFileToTarget(scriptFile), "open");
      return;
    }

    if (scene) {
      services.targetBridge?.openSceneScript?.(scene);
    }
  }

  function revealSourceFolder() {
    if (yamlFile) {
      services.activateEditorTarget?.(projectFileToTarget(yamlFile), "reveal");
      services.onRevealSelectedFile?.();
    }
  }

  return (
    <ContextDock>
      <SceneSummaryWidget
        scene={scene}
        onShowYaml={showYaml}
        onOpenScript={openScript}
        onReveal={revealSourceFolder}
      />
      <SceneScriptsWidget
        scripts={model.scripts}
        onOpenFile={(file) => services.activateEditorTarget?.(projectFileToTarget(file), "open")}
      />
      {registryError ? <p className="muted workspace-note">{registryError}</p> : null}
      <SceneAssetsWidget
        groups={model.assetGroups}
        onSelectAsset={(asset) => services.activateEditorTarget?.(assetToTarget(asset), "select")}
        onShowYaml={services.targetBridge?.showYamlView}
      />
      <SceneEntitiesWidget
        entities={model.entities}
        loading={services.hierarchyTask?.status === "running"}
        onSelectEntity={(entityId) => services.activateEditorTarget?.(sceneEntityIdToTarget(scene.id, entityId), "select")}
      />
      <SelectedEntityWidget
        entity={services.selectedEntity ?? null}
        object={model.selectedObject}
      />
      <EntityTransformWidget
        editorModeSession={services.editorModeSession}
        sceneId={scene.id}
        object={model.selectedObject}
        onApplyCommand={services.applyEditorCommand}
      />
      <SceneDiagnosticsWidget diagnostics={model.diagnostics} />
      <SceneSourceWidget
        source={model.source}
        yamlFile={yamlFile}
        scriptFile={scriptFile}
        onShowYaml={showYaml}
        onOpenScript={openScript}
        onReveal={revealSourceFolder}
      />
    </ContextDock>
  );
}
