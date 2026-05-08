import type {
  EditorComponentContextPayload,
  EditorSerializedComponentContext,
} from "./componentTypes";

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

export function deserializeComponentContext(
  context?: EditorSerializedComponentContext | null,
): EditorSerializedComponentContext | undefined {
  return context ? { ...context } : undefined;
}
