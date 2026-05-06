import { ComponentHost } from "../editor-components/componentHost";
import { DebugSourceOverlay } from "../debug/debugSource";
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
  showDebugSource?: boolean;
};

export function WorkspaceComponentHost({
  context,
  instance,
  services,
  showDebugSource = false,
}: WorkspaceComponentHostProps) {
  const definition = editorComponentById(instance.componentId);

  if (!definition) {
    return (
      <section className="workspace-empty">
        Unknown component: <code>{instance.componentId}</code>
      </section>
    );
  }

  return (
    <DebugSourceOverlay enabled={showDebugSource && Boolean(definition.debugSource)} source={definition.debugSource ?? ""}>
      <div className="workspace-component-body">
        <ComponentHost context={context} instance={instance} services={services} />
      </div>
    </DebugSourceOverlay>
  );
}
