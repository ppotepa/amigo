import type { EditorComponentDefinition } from "./componentTypes";

export function withComponentDebugSource(
  component: EditorComponentDefinition,
  debugSources: Record<string, string>,
): EditorComponentDefinition {
  return {
    ...component,
    debugSource: debugSources[component.id] ?? "src/editor-components/builtin/index.ts",
  };
}
