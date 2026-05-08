import { editorComponentById } from "./componentRegistry";
import {
  deserializeComponentContext,
  serializeComponentContext,
} from "./componentContextSerialization";
import type {
  EditorComponentContextPayload,
  EditorComponentDefinition,
  EditorSerializedComponentContext,
} from "./componentTypes";

export type SerializedEditorComponentRef = {
  componentId: string;
  context?: EditorSerializedComponentContext;
};

export type ResolvedEditorComponentRef = {
  component: EditorComponentDefinition<any>;
  context?: EditorComponentContextPayload;
};

export function serializeEditorComponentRef(
  component: EditorComponentDefinition<any>,
  context?: EditorComponentContextPayload,
): SerializedEditorComponentRef {
  return {
    componentId: component.id,
    context: serializeComponentContext(context),
  };
}

export function resolveSerializedComponentRef(
  serialized: SerializedEditorComponentRef,
): ResolvedEditorComponentRef | null {
  const component = editorComponentById(serialized.componentId);
  if (!component) return null;

  return {
    component,
    context: deserializeComponentContext(component, serialized.context),
  };
}
