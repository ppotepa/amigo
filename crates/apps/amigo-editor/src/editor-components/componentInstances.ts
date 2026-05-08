import type {
  ComponentPlacement,
  EditorComponentDefinition,
  EditorComponentInstance,
  EditorSerializedComponentContext,
} from "./componentTypes";
import {
  AssetsBrowserComponent,
  CachePreviewComponent,
  DiagnosticsProblemsComponent,
  EntityInspectorComponent,
  EventsLogComponent,
  FilesBrowserComponent,
  ScenesBrowserComponent,
  SceneContextComponent,
  ScenePreviewComponent,
  ScriptsBrowserComponent,
  TasksMonitorComponent,
  editorComponentById,
} from "./componentRegistry";

export function singletonComponentInstanceId(componentId: string): string {
  return `${componentId}:singleton`;
}

export function componentInstanceId(
  componentId: string,
  parts: Array<string | null | undefined>,
): string {
  const suffix = parts.filter(Boolean).join(":");
  return suffix ? `${componentId}:${suffix}` : singletonComponentInstanceId(componentId);
}

export function createComponentInstance({
  component,
  componentId,
  context,
  placement,
  resourceUri,
  sessionId,
  titleOverride,
}: {
  component?: EditorComponentDefinition<any>;
  componentId?: string;
  context?: EditorSerializedComponentContext;
  placement?: ComponentPlacement;
  resourceUri?: string;
  sessionId?: string;
  titleOverride?: string;
}): EditorComponentInstance {
  const definition = component ?? (componentId ? editorComponentById(componentId) : undefined);
  const resolvedComponentId = definition?.id ?? componentId;
  if (!resolvedComponentId) {
    throw new Error("createComponentInstance requires a component definition or componentId.");
  }
  const instanceId = definition?.singleton
    ? singletonComponentInstanceId(resolvedComponentId)
    : componentInstanceId(resolvedComponentId, [
        sessionId,
        resourceUri,
        context?.sceneId,
        context?.entityId,
        context?.componentIndex,
      ]);

  return {
    instanceId,
    component: definition ?? {
      id: resolvedComponentId,
      title: resolvedComponentId,
      category: "system",
      domain: "editor",
      icon: "box",
      placement: placement ?? { kind: "centerTab" },
      defaultPlacement: placement ?? { kind: "centerTab" },
      allowedPlacements: [placement?.kind ?? "centerTab"],
      canDock: false,
      canFloat: false,
      canOpenInWindow: false,
      canOpenInCenterTabs: true,
      singleton: false,
      render: () => null,
    },
    componentId: resolvedComponentId,
    context,
    placement: placement ?? definition?.defaultPlacement ?? { kind: "centerTab" },
    resourceUri,
    sessionId,
    titleOverride,
    dirty: false,
  };
}

export const DEFAULT_WORKSPACE_COMPONENT_INSTANCES: EditorComponentInstance[] = [
  createComponentInstance({ component: AssetsBrowserComponent }),
  createComponentInstance({ component: FilesBrowserComponent }),
  createComponentInstance({ component: ScenesBrowserComponent }),
  createComponentInstance({ component: ScriptsBrowserComponent }),
  createComponentInstance({ component: ScenePreviewComponent }),
  createComponentInstance({ component: SceneContextComponent }),
  createComponentInstance({ component: EntityInspectorComponent }),
  createComponentInstance({ component: DiagnosticsProblemsComponent }),
  createComponentInstance({ component: EventsLogComponent }),
  createComponentInstance({ component: TasksMonitorComponent }),
  createComponentInstance({ component: CachePreviewComponent }),
];
