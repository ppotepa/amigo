import { useEffect, useMemo, useState } from "react";
import type { AssetRegistryDto } from "../../../api/dto";
import { getAssetRegistry } from "../../../api/editorApi";
import type { EditorComponentContext } from "../../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../../main-window/workspaceRuntimeServices";
import { resolveSceneScriptFile, resolveSceneYamlFile } from "./sceneContextActions";
import { buildSceneContextModel } from "./sceneContextModel";

export function useSceneContextRuntime({
  context,
  services,
}: {
  context: EditorComponentContext;
  services: WorkspaceRuntimeServices;
}) {
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
      sceneChanges: services.sceneChanges ?? null,
    });
  }, [
    registry,
    scene,
    services.sceneChanges,
    services.editorSnapshot?.objects,
    services.hierarchy,
    services.projectTree?.root,
    services.selectedEntity?.id,
  ]);

  const yamlFile = scene ? resolveSceneYamlFile(services.projectTree, scene) : null;
  const scriptFile = scene ? resolveSceneScriptFile(services.projectTree, scene) : null;

  function showYaml() {
    if (model?.source.yaml) {
      services.targetBridge?.showYamlView?.(model.source.yaml);
    }
  }

  function openScript() {
    if (scene) {
      services.targetBridge?.openSceneScript?.(scene);
    }
  }

  function revealSourceFolder() {
    services.onRevealSelectedFile?.();
  }

  return {
    model,
    openScript,
    registry,
    registryError,
    revealSourceFolder,
    scene,
    scriptFile,
    showYaml,
    yamlFile,
  };
}
