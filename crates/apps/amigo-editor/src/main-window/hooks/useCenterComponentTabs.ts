import { useCallback, useMemo, useState } from "react";
import { createComponentInstance } from "../../editor-components/componentInstances";
import type { EditorComponentInstance } from "../../editor-components/componentTypes";

export function useCenterComponentTabs({
  activeWorkspaceTabId,
  detailsId,
  focusComponent,
  openComponent,
  scenePreviewComponentId,
  scenePreviewInstanceId,
  scenePreviewTabId,
  selectWorkspaceTab,
  sessionId,
}: {
  activeWorkspaceTabId: string;
  detailsId?: string | null;
  focusComponent: (instanceId: string, componentId: string) => void;
  openComponent: (componentId: string, context?: Record<string, string>) => void;
  scenePreviewComponentId: string;
  scenePreviewInstanceId: string;
  scenePreviewTabId: string;
  selectWorkspaceTab: (tabId: string) => void;
  sessionId?: string | null;
}) {
  const [centerComponentTabs, setCenterComponentTabs] = useState<EditorComponentInstance[]>([]);

  const openCenterComponent = useCallback(
    (componentId: string, context: Record<string, string> = {}) => {
      if (componentId === scenePreviewComponentId) {
        selectWorkspaceTab(scenePreviewTabId);
        focusComponent(scenePreviewInstanceId, scenePreviewComponentId);
        return;
      }

      const instance = createComponentInstance({
        componentId,
        context,
        placement: { kind: "centerTab" },
        sessionId: sessionId ?? undefined,
      });
      setCenterComponentTabs((current) =>
        current.some((candidate) => candidate.instanceId === instance.instanceId)
          ? current
          : [...current, instance],
      );
      selectWorkspaceTab(instance.instanceId);
      openComponent(componentId, { modId: detailsId ?? "", sessionId: sessionId ?? "", ...context });
    },
    [
      detailsId,
      focusComponent,
      openComponent,
      scenePreviewComponentId,
      scenePreviewInstanceId,
      scenePreviewTabId,
      selectWorkspaceTab,
      sessionId,
    ],
  );

  const closeCenterComponent = useCallback(
    (instanceId: string) => {
      setCenterComponentTabs((current) => current.filter((instance) => instance.instanceId !== instanceId));
      if (activeWorkspaceTabId === instanceId) {
        selectWorkspaceTab(scenePreviewTabId);
      }
    },
    [activeWorkspaceTabId, scenePreviewTabId, selectWorkspaceTab],
  );

  const activeCenterComponent = useMemo(
    () => centerComponentTabs.find((instance) => instance.instanceId === activeWorkspaceTabId) ?? null,
    [activeWorkspaceTabId, centerComponentTabs],
  );

  return {
    activeCenterComponent,
    centerComponentTabs,
    closeCenterComponent,
    openCenterComponent,
  };
}
