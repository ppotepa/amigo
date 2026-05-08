import type {
  EditorComponentContextOf,
  EditorComponentContextPayload,
  EditorComponentDefinition,
  EditorSerializedComponentContext,
} from "./componentTypes";
import { UiDocumentEditorComponent } from "./builtin";

export function serializeComponentContext(
  context?: EditorComponentContextPayload | null,
): EditorSerializedComponentContext | undefined {
  if (!context) return undefined;

  const serialized: EditorSerializedComponentContext = {};
  for (const [key, value] of Object.entries(context)) {
    if (value === undefined || value === null) continue;
    serialized[key] = String(value);
  }

  return serialized;
}

export function deserializeComponentContext<TComponent extends EditorComponentDefinition<any>>(
  component: TComponent,
  context?: EditorSerializedComponentContext | null,
): EditorComponentContextOf<TComponent> | undefined {
  if (!context) return undefined;

  if (component === UiDocumentEditorComponent) {
    return {
      ...context,
      componentIndex: context.componentIndex === undefined ? undefined : Number(context.componentIndex),
      initialTemplate: context.initialTemplate ?? "empty",
    } as EditorComponentContextOf<TComponent>;
  }

  return { ...context } as EditorComponentContextOf<TComponent>;
}
