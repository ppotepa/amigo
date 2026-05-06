import { useCallback, useMemo, useState } from "react";
import { createComponentInstance } from "../../editor-components/componentInstances";
import type { EditorComponentInstance } from "../../editor-components/componentTypes";

export type OpenCenterComponentOptions = {
  context?: Record<string, string>;
  resourceUri?: string;
  titleOverride?: string;
};

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
    (componentId: string, options: OpenCenterComponentOptions = {}) => {
      if (componentId === scenePreviewComponentId) {
        selectWorkspaceTab(scenePreviewTabId);
        focusComponent(scenePreviewInstanceId, scenePreviewComponentId);
        return;
      }

      const context = {
        modId: detailsId ?? "",
        sessionId: sessionId ?? "",
        ...(options.context ?? {}),
      };

      const instance = createComponentInstance({
        componentId,
        context,
        placement: { kind: "centerTab" },
        resourceUri: options.resourceUri,
        sessionId: sessionId ?? undefined,
        titleOverride: options.titleOverride,
      });
      setCenterComponentTabs((current) =>
        current.some((candidate) => candidate.instanceId === instance.instanceId)
          ? current
          : [...current, instance],
      );
      selectWorkspaceTab(instance.instanceId);
      focusComponent(instance.instanceId, componentId);
      openComponent(componentId, context);
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
