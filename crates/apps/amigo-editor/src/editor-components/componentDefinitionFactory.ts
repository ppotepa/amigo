import type {
  EditorComponentContextPayload,
  EditorComponentDefinition,
  EditorComponentContextOf,
} from "./componentTypes";

export function defineEditorComponent<TContext extends EditorComponentContextPayload = any>() {
  return function defineTypedEditorComponent(
    component: EditorComponentDefinition<TContext>,
  ): EditorComponentDefinition<TContext> {
    return component;
  };
}

export function componentIdOf(component: EditorComponentDefinition<any>): string {
  return component.id;
}

export function componentContextOf<TComponent extends EditorComponentDefinition<any>>(
  _component: TComponent,
): EditorComponentContextOf<TComponent> | undefined {
  return undefined;
}
