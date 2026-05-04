import {
  canOpenEditorComponent,
  EDITOR_COMPONENTS,
  editorComponentById,
  iconForEditorComponent,
} from "../editor-components/componentRegistry";
import type { ComponentPlacementKind } from "../editor-components/componentTypes";
import type { DockAreaId } from "../main-window/workspaceLayout";
import type { DockPlugin, EditorDockContext } from "./dockTypes";

const DOCK_COMPONENT_TO_LEGACY_ID: Record<string, string> = {
  "project.explorer": "project-explorer",
  "assets.browser": "asset-browser",
  "files.browser": "files-browser",
  "scenes.browser": "scene-browser",
  "scripts.browser": "scripts-browser",
  "scene.hierarchy": "scene-hierarchy",
  "scene.preview": "scene-preview",
  "entity.inspector": "inspector",
  "diagnostics.problems": "problems",
  "events.log": "event-log",
  "tasks.monitor": "tasks",
  "cache.preview": "preview-cache",
};

const EXTRA_LEGACY_PLUGINS: DockPlugin[] = [
  {
    id: "diagnostics",
    componentId: "diagnostics.problems",
    title: "Diagnostics",
    icon: iconForEditorComponent("alert-triangle"),
    defaultDock: "right",
    canOpen: (context) => Boolean(context.modId),
  },
  {
    id: "properties",
    componentId: "entity.inspector",
    title: "Properties",
    icon: iconForEditorComponent("layers"),
    defaultDock: "right",
    canOpen: (context) => Boolean(context.modId),
  },
  {
    id: "console",
    componentId: "scripting.console",
    title: "Console",
    icon: iconForEditorComponent("terminal"),
    defaultDock: "bottom",
    canOpen: () => true,
  },
];

function dockAreaForPlacement(placement: ComponentPlacementKind): DockAreaId | null {
  switch (placement) {
    case "leftDock":
      return "left";
    case "rightDock":
      return "right";
    case "bottomDock":
      return "bottom";
    case "centerTab":
      return "center";
    default:
      return null;
  }
}

function componentContextFromDockContext(context: EditorDockContext) {
  return {
    sessionId: context.sessionId,
    modId: context.modId,
    selectedSceneId: context.selectedSceneId,
  };
}

export const DOCK_PLUGINS: DockPlugin[] = [
  ...EDITOR_COMPONENTS.flatMap((component) => {
    const id = DOCK_COMPONENT_TO_LEGACY_ID[component.id];
    const defaultDock = dockAreaForPlacement(component.defaultPlacement.kind);
    if (!id || !defaultDock) {
      return [];
    }

    return [
      {
        id,
        componentId: component.id,
        title: component.title.replace(" Explorer", "").replace(" Browser", ""),
        icon: iconForEditorComponent(component.icon),
        defaultDock,
        canOpen: (context: EditorDockContext) => canOpenEditorComponent(component, componentContextFromDockContext(context)),
      },
    ];
  }),
  ...EXTRA_LEGACY_PLUGINS,
];

export function dockPluginById(pluginId: string): DockPlugin | undefined {
  return DOCK_PLUGINS.find((plugin) => plugin.id === pluginId);
}

export function dockPluginByComponentId(componentId: string): DockPlugin | undefined {
  return DOCK_PLUGINS.find((plugin) => plugin.componentId === componentId);
}

export function dockComponentDefinition(pluginId: string) {
  const plugin = dockPluginById(pluginId);
  return plugin ? editorComponentById(plugin.componentId) : undefined;
}
