import type {
  ComponentPlacement,
  EditorComponentContextPayload,
  EditorComponentDefinition,
  EditorComponentInstance,
} from "./componentTypes";
import { deserializeComponentContext, serializeComponentContext } from "./componentContextSerialization";
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
} from "./componentRegistry";

export function singletonComponentInstanceId(component: EditorComponentDefinition<any>): string {
  return `${component.id}:singleton`;
}

export function componentInstanceId(
  component: EditorComponentDefinition<any>,
  parts: Array<string | null | undefined>,
): string {
  const suffix = parts.filter(Boolean).join(":");
  return suffix ? `${component.id}:${suffix}` : singletonComponentInstanceId(component);
}
export function createComponentInstance<TContext extends EditorComponentContextPayload>({
  component,
  context,
  placement,
  resourceUri,
  sessionId,
  titleOverride,
}: {
  component: EditorComponentDefinition<TContext>;
  context?: TContext;
  placement?: ComponentPlacement;
  resourceUri?: string;
  sessionId?: string;
  titleOverride?: string;
}): EditorComponentInstance<TContext> {
  const serializedContext = serializeComponentContext(context);
  const resolvedContext = serializedContext
    ? deserializeComponentContext(component, serializedContext)
    : context;
  const instanceId = component.singleton
    ? singletonComponentInstanceId(component)
    : componentInstanceId(component, [
        sessionId,
        resourceUri,
        serializedContext?.sceneId,
        serializedContext?.entityId,
        serializedContext?.componentIndex,
      ]);

  return {
    instanceId,
    component,
    componentId: component.id,
    context: resolvedContext,
    placement: placement ?? component.defaultPlacement ?? { kind: "centerTab" },
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
