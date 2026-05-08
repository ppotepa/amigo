import type {
  EditorComponentContextPayload,
  EditorComponentDefinition,
} from "./componentTypes";

export function defineEditorComponent<TContext extends EditorComponentContextPayload = EditorComponentContextPayload>(
  component: EditorComponentDefinition<TContext>,
): EditorComponentDefinition<TContext> {
  return component;
}

export function withComponentDebugSource(
  component: EditorComponentDefinition,
  debugSources: Record<string, string>,
): EditorComponentDefinition {
  return {
    ...component,
    debugSource: debugSources[component.id] ?? "src/editor-components/builtin/index.ts",
  };
}
