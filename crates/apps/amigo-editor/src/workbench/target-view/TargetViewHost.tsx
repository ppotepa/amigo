import type { ReactNode } from "react";
import type {
  EditorComponentContext,
  EditorComponentInstance,
} from "../../editor-components/componentTypes";
import type { ResolvedEditorTarget } from "../../editor-targets";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { resolveTargetContract } from "./resolveTargetContract";
import { WorkbenchSlotContent } from "./WorkbenchSlotContent";

export type TargetViewHostProps = {
  context: EditorComponentContext;
  instance: EditorComponentInstance;
  target: ResolvedEditorTarget | null;
  services: WorkspaceRuntimeServices;
  fallback?: ReactNode;
};

export function TargetViewHost({
  fallback = null,
  context,
  instance,
  services,
  target,
}: TargetViewHostProps) {
  if (!target) return <>{fallback}</>;

  const contract = resolveTargetContract(target);
  if (!contract) return <>{fallback}</>;

  const model = contract.buildModel({
    context,
    instance,
    ref: target.ref,
    resolved: target,
    services,
  });
  const actions = contract.buildActions({
    context,
    instance,
    ref: target.ref,
    resolved: target,
    services,
  }, model);
  const slots = contract.renderSlots({
    context,
    instance,
    model,
    actions,
    services,
  });

  return <WorkbenchSlotContent slots={slots} />;
}
