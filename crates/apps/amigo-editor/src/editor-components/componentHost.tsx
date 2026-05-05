import type { EditorComponentContext, EditorComponentInstance } from "./componentTypes";
import { editorComponentById } from "./componentRegistry";
import type { WorkspaceRuntimeServices } from "../main-window/workspaceRuntimeServices";

export function ComponentHost({
  context,
  instance,
  services,
}: {
  context: EditorComponentContext;
  instance: EditorComponentInstance;
  services: WorkspaceRuntimeServices;
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
  return <Component context={context} instance={instance} services={services} />;
}
