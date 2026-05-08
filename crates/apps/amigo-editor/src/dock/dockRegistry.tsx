import {
  canOpenEditorComponent,
  EDITOR_COMPONENTS,
  iconForEditorComponent,
} from "../editor-components/componentRegistry";
import type { ComponentPlacementKind, EditorComponentDefinition } from "../editor-components/componentTypes";
import type { DockAreaId } from "../main-window/workspaceLayout";
import { toneForComponentDomain } from "../theme/semanticColorRegistry";
import type { DockPlugin, EditorDockContext } from "./dockTypes";

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
    const defaultDock = dockAreaForPlacement(component.defaultPlacement.kind);
    if (!defaultDock) {
      return [];
    }

    return [
      {
        id: component.id,
        component,
        title: component.title.replace(" Explorer", "").replace(" Browser", ""),
        icon: iconForEditorComponent(component.icon, 14, toneForComponentDomain(component.domain)),
        defaultDock,
        canOpen: (context: EditorDockContext) => canOpenEditorComponent(component, componentContextFromDockContext(context)),
      },
    ];
  }),
];

export function dockPluginByComponent(component: EditorComponentDefinition<any>): DockPlugin | undefined {
  return DOCK_PLUGINS.find((plugin) => plugin.component.id === component.id);
}
