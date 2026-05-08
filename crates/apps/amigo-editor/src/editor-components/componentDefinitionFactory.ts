import type {
  EditorComponentId,
  EditorComponentContextPayload,
  EditorComponentDefinition,
} from "./componentTypes";

export type EditorComponentDebugSources = Partial<Record<EditorComponentId, string>>;

export function defineEditorComponent<TContext extends EditorComponentContextPayload = any>(
  component: EditorComponentDefinition<TContext>,
): EditorComponentDefinition<TContext> {
  return component;
}

export function withComponentDebugSource(
  component: EditorComponentDefinition,
  debugSources: EditorComponentDebugSources,
): EditorComponentDefinition {
  const componentId = component.id as EditorComponentId;
  return {
    ...component,
    debugSource: debugSources[componentId] ?? "src/editor-components/builtin/index.ts",
  };
}
