import type React from "react";
import { DockAreaHost } from "./DockAreaHost";
import { WorkspaceComponentHost } from "./WorkspaceComponentHost";
import { WorkspaceResizeHandle } from "./WorkspaceResizeHandle";
import type { EditorComponentContext, EditorComponentInstance } from "../editor-components/componentTypes";
import type { WorkspaceDockAreaId } from "./workspaceLayout";
import type { WorkspaceRuntimeServices } from "./workspaceRuntimeServices";

type DockKey = Exclude<WorkspaceDockAreaId, "center">;
type DockSizeKey = "leftWidth" | "rightWidth" | "rightBottomHeight" | "bottomHeight";

// @codemap anchor:workspace-dock-grid domain:workspace role:right-dock-split priority:P1 layer:app tags:dock-layout,rightTop,rightBottom
export function MainWorkspaceDockGrid({
  activeBottomInstance,
  activeLeftInstance,
  activeRightBottomInstance,
  activeRightTopInstance,
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
  onSelectRightBottomInstance,
  onSelectRightTopInstance,
  renderComponentToolbar,
  rightBottomDockInstances,
  rightBottomTabs,
  rightTopDockInstances,
  rightTopTabs,
  showComponentSources,
  toolbarStateFor,
  workspaceRuntimeServices,
}: {
  activeBottomInstance: EditorComponentInstance | null;
  activeLeftInstance: EditorComponentInstance | null;
  activeRightBottomInstance: EditorComponentInstance | null;
  activeRightTopInstance: EditorComponentInstance | null;
  bottomDockInstances: EditorComponentInstance[];
  bottomTabs: Array<{ id: string; title: string; icon: React.ReactNode; dirty?: boolean }>;
  componentContext: EditorComponentContext;
  leftDockInstances: EditorComponentInstance[];
  leftTabs: Array<{ id: string; title: string; icon: React.ReactNode; dirty?: boolean }>;
  onFocusComponent: (instanceId: string, componentId: string) => void;
  onRecordDockTabSelected: (dock: DockKey, tabId: string) => void;
  onResizeDock: (key: DockSizeKey, delta: number) => void;
  onResetDockSize: (key: DockSizeKey) => void;
  onSelectBottomInstance: (instanceId: string) => void;
  onSelectLeftInstance: (instanceId: string) => void;
  onSelectRightBottomInstance: (instanceId: string) => void;
  onSelectRightTopInstance: (instanceId: string) => void;
  renderComponentToolbar: (instance: EditorComponentInstance) => React.ReactNode;
  rightBottomDockInstances: EditorComponentInstance[];
  rightBottomTabs: Array<{ id: string; title: string; icon: React.ReactNode; dirty?: boolean }>;
  rightTopDockInstances: EditorComponentInstance[];
  rightTopTabs: Array<{ id: string; title: string; icon: React.ReactNode; dirty?: boolean }>;
  showComponentSources: boolean;
  toolbarStateFor: (instance: EditorComponentInstance) => WorkspaceRuntimeServices["toolbarState"];
  workspaceRuntimeServices: WorkspaceRuntimeServices;
}) {
  const emptyDockContent = <p className="muted workspace-note">No components in this dock.</p>;

  return (
    <>
      <DockAreaHost
        className="dock-left"
        tabs={leftTabs}
        activeTab={activeLeftInstance?.instanceId ?? ""}
        toolbar={activeLeftInstance ? renderComponentToolbar(activeLeftInstance) : null}
        onSelect={(instanceId) => {
          const instance = leftDockInstances.find((candidate) => candidate.instanceId === instanceId);
          if (!instance) return;
          onSelectLeftInstance(instanceId);
          onFocusComponent(instance.instanceId, instance.componentId);
          onRecordDockTabSelected("left", instanceId);
        }}
      >
        {activeLeftInstance ? (
          <WorkspaceComponentHost
            instance={activeLeftInstance}
            context={componentContext}
            showDebugSource={showComponentSources}
            services={{
              ...workspaceRuntimeServices,
              toolbarState: toolbarStateFor(activeLeftInstance),
            }}
          />
        ) : (
          emptyDockContent
        )}
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

      <div className="workspace-right-split">
        <DockAreaHost
          className="dock-right-top"
          tabs={rightTopTabs}
          activeTab={activeRightTopInstance?.instanceId ?? ""}
          toolbar={activeRightTopInstance ? renderComponentToolbar(activeRightTopInstance) : null}
          onSelect={(instanceId) => {
            const instance = rightTopDockInstances.find((candidate) => candidate.instanceId === instanceId);
            if (!instance) return;
            onSelectRightTopInstance(instanceId);
            onFocusComponent(instance.instanceId, instance.componentId);
            onRecordDockTabSelected("rightTop", instanceId);
          }}
        >
          {activeRightTopInstance ? (
            <WorkspaceComponentHost
              instance={activeRightTopInstance}
              context={componentContext}
              showDebugSource={showComponentSources}
              services={{
                ...workspaceRuntimeServices,
                toolbarState: toolbarStateFor(activeRightTopInstance),
              }}
            />
          ) : (
            emptyDockContent
          )}
        </DockAreaHost>

        <WorkspaceResizeHandle
          className="resize-right-bottom"
          orientation="horizontal"
          title="Resize right bottom dock"
          onDrag={(delta) => onResizeDock("rightBottomHeight", -delta)}
          onReset={() => onResetDockSize("rightBottomHeight")}
        />

        <DockAreaHost
          className="dock-right-bottom"
          tabs={rightBottomTabs}
          activeTab={activeRightBottomInstance?.instanceId ?? ""}
          toolbar={activeRightBottomInstance ? renderComponentToolbar(activeRightBottomInstance) : null}
          onSelect={(instanceId) => {
            const instance = rightBottomDockInstances.find((candidate) => candidate.instanceId === instanceId);
            if (!instance) return;
            onSelectRightBottomInstance(instanceId);
            onFocusComponent(instance.instanceId, instance.componentId);
            onRecordDockTabSelected("rightBottom", instanceId);
          }}
        >
          {activeRightBottomInstance ? (
            <WorkspaceComponentHost
              instance={activeRightBottomInstance}
              context={componentContext}
              showDebugSource={showComponentSources}
              services={{
                ...workspaceRuntimeServices,
                toolbarState: toolbarStateFor(activeRightBottomInstance),
              }}
            />
          ) : (
            emptyDockContent
          )}
        </DockAreaHost>
      </div>

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
        activeTab={activeBottomInstance?.instanceId ?? ""}
        toolbar={activeBottomInstance ? renderComponentToolbar(activeBottomInstance) : null}
        onSelect={(instanceId) => {
          const instance = bottomDockInstances.find((candidate) => candidate.instanceId === instanceId);
          if (!instance) return;
          onSelectBottomInstance(instanceId);
          onFocusComponent(instance.instanceId, instance.componentId);
          onRecordDockTabSelected("bottom", instanceId);
        }}
      >
        {activeBottomInstance ? (
          <WorkspaceComponentHost
            instance={activeBottomInstance}
            context={componentContext}
            showDebugSource={showComponentSources}
            services={{
              ...workspaceRuntimeServices,
              toolbarState: toolbarStateFor(activeBottomInstance),
            }}
          />
        ) : (
          emptyDockContent
        )}
      </DockAreaHost>
    </>
  );
}
