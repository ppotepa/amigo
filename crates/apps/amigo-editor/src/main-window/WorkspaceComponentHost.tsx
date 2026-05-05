import { ComponentHost } from "../editor-components/componentHost";
import { editorComponentById } from "../editor-components/componentRegistry";
import type {
  EditorComponentContext,
  EditorComponentInstance,
} from "../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "./workspaceRuntimeServices";

export type WorkspaceComponentHostProps = {
  context: EditorComponentContext;
  instance: EditorComponentInstance;
  services: WorkspaceRuntimeServices;
};

export function WorkspaceComponentHost({
  context,
  instance,
  services,
}: WorkspaceComponentHostProps) {
  const definition = editorComponentById(instance.componentId);

  if (!definition) {
    return (
      <section className="workspace-empty">
        Unknown component: <code>{instance.componentId}</code>
      </section>
    );
  }

  return <ComponentHost context={context} instance={instance} services={services} />;
}
