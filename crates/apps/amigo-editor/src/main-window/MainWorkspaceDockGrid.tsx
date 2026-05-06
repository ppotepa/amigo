import type React from "react";
import { DockAreaHost } from "./DockAreaHost";
import { WorkspaceComponentHost } from "./WorkspaceComponentHost";
import { WorkspaceResizeHandle } from "./WorkspaceResizeHandle";
import type { EditorComponentContext, EditorComponentInstance } from "../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "./workspaceRuntimeServices";

type DockKey = "left" | "right" | "bottom";

export function MainWorkspaceDockGrid({
  activeBottomInstance,
  activeLeftInstance,
  activeRightInstance,
  bottomDockInstances,
  bottomTabs,
  componentContext,
  leftDockInstances,
  leftTabs,
  onFocusComponent,
  onRecordDockTabSelected,
  onResizeDock,
  onResetDockSize,
  onSelectBottomInstance,
  onSelectLeftInstance,
  onSelectRightInstance,
  renderComponentToolbar,
  rightDockInstances,
  rightTabs,
  showComponentSources,
  toolbarStateFor,
  workspaceRuntimeServices,
}: {
  activeBottomInstance: EditorComponentInstance;
  activeLeftInstance: EditorComponentInstance;
  activeRightInstance: EditorComponentInstance;
  bottomDockInstances: EditorComponentInstance[];
  bottomTabs: Array<{ id: string; title: string; icon: React.ReactNode; dirty?: boolean }>;
  componentContext: EditorComponentContext;
  leftDockInstances: EditorComponentInstance[];
  leftTabs: Array<{ id: string; title: string; icon: React.ReactNode; dirty?: boolean }>;
  onFocusComponent: (instanceId: string, componentId: string) => void;
  onRecordDockTabSelected: (dock: DockKey, tabId: string) => void;
  onResizeDock: (key: "leftWidth" | "rightWidth" | "bottomHeight", delta: number) => void;
  onResetDockSize: (key: "leftWidth" | "rightWidth" | "bottomHeight") => void;
  onSelectBottomInstance: (instanceId: string) => void;
  onSelectLeftInstance: (instanceId: string) => void;
  onSelectRightInstance: (instanceId: string) => void;
  renderComponentToolbar: (instance: EditorComponentInstance) => React.ReactNode;
  rightDockInstances: EditorComponentInstance[];
  rightTabs: Array<{ id: string; title: string; icon: React.ReactNode; dirty?: boolean }>;
  showComponentSources: boolean;
  toolbarStateFor: (instance: EditorComponentInstance) => WorkspaceRuntimeServices["toolbarState"];
  workspaceRuntimeServices: WorkspaceRuntimeServices;
}) {
  return (
    <>
      <DockAreaHost
        className="dock-left"
        tabs={leftTabs}
        activeTab={activeLeftInstance.instanceId}
        toolbar={renderComponentToolbar(activeLeftInstance)}
        onSelect={(instanceId) => {
          const instance = leftDockInstances.find((candidate) => candidate.instanceId === instanceId);
          if (!instance) return;
          onSelectLeftInstance(instanceId);
          onFocusComponent(instance.instanceId, instance.componentId);
          onRecordDockTabSelected("left", instanceId);
        }}
      >
        <WorkspaceComponentHost
          instance={activeLeftInstance}
          context={componentContext}
          showDebugSource={showComponentSources}
          services={{
            ...workspaceRuntimeServices,
            toolbarState: toolbarStateFor(activeLeftInstance),
          }}
        />
      </DockAreaHost>

      <WorkspaceResizeHandle
        className="resize-left"
        orientation="vertical"
        title="Resize left dock"
        onDrag={(delta) => onResizeDock("leftWidth", delta)}
        onReset={() => onResetDockSize("leftWidth")}
      />

      <WorkspaceResizeHandle
        className="resize-right"
        orientation="vertical"
        title="Resize right dock"
        onDrag={(delta) => onResizeDock("rightWidth", -delta)}
        onReset={() => onResetDockSize("rightWidth")}
      />

      <DockAreaHost
        className="dock-right"
        tabs={rightTabs}
        activeTab={activeRightInstance.instanceId}
        toolbar={renderComponentToolbar(activeRightInstance)}
        onSelect={(instanceId) => {
          const instance = rightDockInstances.find((candidate) => candidate.instanceId === instanceId);
          if (!instance) return;
          onSelectRightInstance(instanceId);
          onFocusComponent(instance.instanceId, instance.componentId);
          onRecordDockTabSelected("right", instanceId);
        }}
      >
        <WorkspaceComponentHost
          instance={activeRightInstance}
          context={componentContext}
          showDebugSource={showComponentSources}
          services={{
            ...workspaceRuntimeServices,
            toolbarState: toolbarStateFor(activeRightInstance),
          }}
        />
      </DockAreaHost>

      <WorkspaceResizeHandle
        className="resize-bottom"
        orientation="horizontal"
        title="Resize bottom dock"
        onDrag={(delta) => onResizeDock("bottomHeight", -delta)}
        onReset={() => onResetDockSize("bottomHeight")}
      />

      <DockAreaHost
        className="dock-bottom"
        tabs={bottomTabs}
        activeTab={activeBottomInstance.instanceId}
        toolbar={renderComponentToolbar(activeBottomInstance)}
        onSelect={(instanceId) => {
          const instance = bottomDockInstances.find((candidate) => candidate.instanceId === instanceId);
          if (!instance) return;
          onSelectBottomInstance(instanceId);
          onFocusComponent(instance.instanceId, instance.componentId);
          onRecordDockTabSelected("bottom", instanceId);
        }}
      >
        <WorkspaceComponentHost
          instance={activeBottomInstance}
          context={componentContext}
          showDebugSource={showComponentSources}
          services={{
            ...workspaceRuntimeServices,
            toolbarState: toolbarStateFor(activeBottomInstance),
          }}
        />
      </DockAreaHost>
    </>
  );
}
