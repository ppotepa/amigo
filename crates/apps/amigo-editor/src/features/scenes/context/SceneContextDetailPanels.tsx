// Legacy scene-specific tab composition.
// TargetContract entrypoint lives in features/scene/target/sceneTargetView.tsx.
import type React from "react";
import type { EditorComponentProps } from "../../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../../main-window/workspaceRuntimeServices";
import type { TabSpec } from "../../../workbench/layout/layoutTypes";
import { SceneChangesTab } from "./details/SceneChangesTab";
import { SceneDiagnosticsTab } from "./details/SceneDiagnosticsTab";
import { SceneFileInfoTab } from "./details/SceneFileInfoTab";
import { SceneInfoTab } from "./details/SceneInfoTab";
import { ScenePropertiesTab } from "./details/ScenePropertiesTab";
import { sceneContextIcon } from "./sceneContextIcons";
import type { SceneContextModel } from "./sceneContextTypes";
import { useSceneContextRuntime } from "./useSceneContextRuntime";

type SceneDetailPanelKind =
  | "info"
  | "file"
  | "properties"
  | "diagnostics"
  | "changes";

type SceneDetailRenderContext = {
  model: SceneContextModel;
  openScript: () => void;
  revealSourceFolder: () => void;
  scriptFile: ReturnType<typeof useSceneContextRuntime>["scriptFile"];
  services: WorkspaceRuntimeServices;
  showYaml: () => void;
  yamlFile: ReturnType<typeof useSceneContextRuntime>["yamlFile"];
};

const SCENE_DETAIL_RENDERERS = {
  info: ({ model }) => <SceneInfoTab model={model} />,
  file: ({ model, openScript, revealSourceFolder, scriptFile, showYaml, yamlFile }) => (
    <SceneFileInfoTab
      model={model}
      yamlFile={yamlFile}
      scriptFile={scriptFile}
      onShowYaml={showYaml}
      onOpenScript={openScript}
      onReveal={revealSourceFolder}
    />
  ),
  properties: ({ model, services }) => <ScenePropertiesTab model={model} services={services} />,
  diagnostics: ({ model, services }) => <SceneDiagnosticsTab model={model} services={services} />,
  changes: ({ model, services }) => <SceneChangesTab model={model} services={services} />,
} satisfies Record<SceneDetailPanelKind, (context: SceneDetailRenderContext) => React.ReactElement>;

export function createSceneContextBottomTabs(context: SceneDetailRenderContext): TabSpec[] {
  return [
    {
      id: "scene-info",
      title: "Scene Info",
      icon: sceneContextIcon("scene"),
      content: SCENE_DETAIL_RENDERERS.info(context),
    },
    {
      id: "file-info",
      title: "File Info",
      icon: sceneContextIcon("script"),
      content: SCENE_DETAIL_RENDERERS.file(context),
    },
    {
      id: "properties",
      title: "Properties",
      icon: sceneContextIcon("component"),
      content: SCENE_DETAIL_RENDERERS.properties(context),
    },
    {
      id: "diagnostics",
      title: "Diagnostics",
      icon: sceneContextIcon("diagnostic"),
      content: SCENE_DETAIL_RENDERERS.diagnostics(context),
    },
    {
      id: "changes",
      title: "Changes",
      icon: sceneContextIcon("diagnostic"),
      content: SCENE_DETAIL_RENDERERS.changes(context),
    },
  ];
}

export function SceneInfoPanel(props: EditorComponentProps<WorkspaceRuntimeServices>) {
  return <SceneDetailPanel {...props} panel="info" />;
}

export function SceneFileInfoPanel(props: EditorComponentProps<WorkspaceRuntimeServices>) {
  return <SceneDetailPanel {...props} panel="file" />;
}

export function ScenePropertiesPanel(props: EditorComponentProps<WorkspaceRuntimeServices>) {
  return <SceneDetailPanel {...props} panel="properties" />;
}

export function SceneDiagnosticsPanel(props: EditorComponentProps<WorkspaceRuntimeServices>) {
  return <SceneDetailPanel {...props} panel="diagnostics" />;
}

export function SceneChangesPanel(props: EditorComponentProps<WorkspaceRuntimeServices>) {
  return <SceneDetailPanel {...props} panel="changes" />;
}

function SceneDetailPanel({
  context,
  panel,
  services,
}: EditorComponentProps<WorkspaceRuntimeServices> & {
  panel: SceneDetailPanelKind;
}) {
  const runtime = useSceneContextRuntime({ context, services });
  const { model, openScript, revealSourceFolder, scene, scriptFile, showYaml, yamlFile } = runtime;

  if (!scene || !model) {
    return <p className="muted workspace-empty">Select a scene to inspect details.</p>;
  }

  return (
    <div className="context-dock scene-detail-panel">
      {SCENE_DETAIL_RENDERERS[panel]({
        model,
        openScript,
        revealSourceFolder,
        scriptFile,
        services,
        showYaml,
        yamlFile,
      })}
    </div>
  );
}
