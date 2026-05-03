import type { ComponentPlacement, EditorComponentInstance } from "./componentTypes";
import { editorComponentById } from "./componentRegistry";

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
  componentId,
  context,
  placement,
  resourceUri,
  sessionId,
  titleOverride,
}: {
  componentId: string;
  context?: Record<string, string>;
  placement?: ComponentPlacement;
  resourceUri?: string;
  sessionId?: string;
  titleOverride?: string;
}): EditorComponentInstance {
  const definition = editorComponentById(componentId);
  const instanceId = definition?.singleton
    ? singletonComponentInstanceId(componentId)
    : componentInstanceId(componentId, [sessionId, resourceUri, context?.sceneId, context?.entityId]);

  return {
    instanceId,
    componentId,
    context,
    placement: placement ?? definition?.defaultPlacement ?? { kind: "centerTab" },
    resourceUri,
    sessionId,
    titleOverride,
    dirty: false,
  };
}

export const DEFAULT_WORKSPACE_COMPONENT_INSTANCES: EditorComponentInstance[] = [
  createComponentInstance({ componentId: "project.explorer" }),
  createComponentInstance({ componentId: "assets.browser" }),
  createComponentInstance({ componentId: "scene.hierarchy" }),
  createComponentInstance({ componentId: "scene.preview" }),
  createComponentInstance({ componentId: "entity.inspector" }),
  createComponentInstance({ componentId: "diagnostics.problems" }),
  createComponentInstance({ componentId: "events.log" }),
  createComponentInstance({ componentId: "tasks.monitor" }),
  createComponentInstance({ componentId: "cache.preview" }),
];
