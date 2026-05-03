import type { EditorComponentContext, EditorComponentInstance } from "./componentTypes";
import { editorComponentById } from "./componentRegistry";

export function ComponentHost({
  context,
  instance,
}: {
  context: EditorComponentContext;
  instance: EditorComponentInstance;
}) {
  const definition = editorComponentById(instance.componentId);
  if (!definition) {
    return (
      <section className="workspace-empty">
        Unknown component: <code>{instance.componentId}</code>
      </section>
    );
  }

  const Component = definition.render;
  return <Component context={context} instance={instance} />;
}

export function RegisteredComponentPlaceholder({
  instance,
}: {
  instance: EditorComponentInstance;
}) {
  return (
    <section className="workspace-empty">
      <p>Registered component</p>
      <code>{instance.componentId}</code>
    </section>
  );
}
