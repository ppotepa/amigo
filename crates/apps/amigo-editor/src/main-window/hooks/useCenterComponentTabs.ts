import { useCallback, useMemo } from "react";
import { createComponentInstance } from "../../editor-components/componentInstances";
import type { EditorComponentInstance } from "../../editor-components/componentTypes";

export type OpenCenterComponentOptions = {
  context?: Record<string, string>;
  resourceUri?: string;
  titleOverride?: string;
};

export function useCenterComponentTabs({
  activeWorkspaceTabId,
  centerComponentTabs,
  detailsId,
  focusComponent,
  openCenterComponentTab,
  closeCenterComponentTab,
  openComponent,
  scenePreviewComponentId,
  scenePreviewInstanceId,
  scenePreviewTabId,
  selectWorkspaceTab,
  sessionId,
}: {
  activeWorkspaceTabId: string;
  centerComponentTabs: EditorComponentInstance[];
  detailsId?: string | null;
  focusComponent: (instanceId: string, componentId: string) => void;
  openCenterComponentTab: (instance: EditorComponentInstance) => void;
  closeCenterComponentTab: (instanceId: string) => void;
  openComponent: (componentId: string, context?: Record<string, string>) => void;
  scenePreviewComponentId: string;
  scenePreviewInstanceId: string;
  scenePreviewTabId: string;
  selectWorkspaceTab: (tabId: string) => void;
  sessionId?: string | null;
}) {
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
      openCenterComponentTab(instance);
      selectWorkspaceTab(instance.instanceId);
      focusComponent(instance.instanceId, componentId);
      openComponent(componentId, context);
    },
    [
      detailsId,
      focusComponent,
      openCenterComponentTab,
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
      closeCenterComponentTab(instanceId);
      if (activeWorkspaceTabId === instanceId) {
        selectWorkspaceTab(scenePreviewTabId);
      }
    },
    [activeWorkspaceTabId, closeCenterComponentTab, scenePreviewTabId, selectWorkspaceTab],
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
