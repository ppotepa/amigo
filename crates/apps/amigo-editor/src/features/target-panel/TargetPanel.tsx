import type { EditorComponentProps } from "../../editor-components/componentTypes";
import { TargetViewFallback } from "../../workbench/target-view/TargetViewFallback";
import { TargetViewHost } from "../../workbench/target-view/TargetViewHost";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";

export function TargetPanel({
  context,
  instance,
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  const target = services.currentEditorTarget ?? null;

  return (
    <TargetViewHost
      context={context}
      instance={instance}
      target={target}
      services={services}
      fallback={<TargetViewFallback context={context} instance={instance} services={services} />}
    />
  );
}