import { createComponentInstance } from "../editor-components/componentInstances";
import type { ComponentPlacementKind, EditorComponentInstance } from "../editor-components/componentTypes";
import type { ResolvedEditorTarget } from "../editor-targets";
import type { WorkspaceRuntimeServices } from "./workspaceRuntimeServices";
import type { WorkspaceDockProfile, WorkspaceDockSlot } from "./workspaceDockProfiles";

export type WorkspaceDockInstances = {
  left: EditorComponentInstance[];
  rightTop: EditorComponentInstance[];
  rightBottom: EditorComponentInstance[];
  bottom: EditorComponentInstance[];
};

export function buildWorkspaceDockInstances({
  profile,
  target,
  services,
  sessionId,
}: {
  profile: WorkspaceDockProfile;
  target: ResolvedEditorTarget | null;
  services: WorkspaceRuntimeServices;
  sessionId?: string;
}): WorkspaceDockInstances {
  return {
    left: buildForSlots(profile.left, "leftDock", target, services, sessionId),
    rightTop: buildForSlots(profile.rightTop, "rightDock", target, services, sessionId),
    rightBottom: buildForSlots(profile.rightBottom, "rightDock", target, services, sessionId),
    bottom: buildForSlots(profile.bottom, "bottomDock", target, services, sessionId),
  };
}

function buildForSlots(
  slots: readonly WorkspaceDockSlot[],
  placementKind: ComponentPlacementKind,
  target: ResolvedEditorTarget | null,
  services: WorkspaceRuntimeServices,
  sessionId?: string,
): EditorComponentInstance[] {
  const instances: EditorComponentInstance[] = [];
  for (const slot of slots) {
    const context = target && slot.component.contextFromTarget
      ? slot.component.contextFromTarget(target, services)
      : undefined;
    const presentation = target && slot.component.presentationFromTarget
      ? slot.component.presentationFromTarget(target, services)
      : undefined;
    instances.push(
      createComponentInstance({
        component: slot.component,
        context: context ?? undefined,
        placement: { kind: placementKind },
        sessionId,
        titleOverride: presentation?.title,
        iconOverride: presentation?.icon,
      }),
    );
  }
  return dedupeById(instances);
}

function dedupeById(instances: EditorComponentInstance[]): EditorComponentInstance[] {
  const seen = new Set<string>();
  return instances.filter((instance) => {
    if (seen.has(instance.instanceId)) return false;
    seen.add(instance.instanceId);
    return true;
  });
}
