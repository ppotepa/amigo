import type { EditorComponentInstance } from "../editor-components/componentTypes";
import type { WorkspaceTabView } from "./hooks/useWorkspaceTabs";

// @codemap anchor:workspace-tab-strip domain:workspace role:tab-strip priority:P1 layer:app tags:tabs,detached-workspace
export function WorkspaceTabsStrip({
  activeTabId,
  canDetachTab,
  centerComponentTabs,
  closeCenterComponent,
  closeWorkspaceTab,
  detachWorkspaceTab,
  selectWorkspaceTab,
  tabs,
}: {
  activeTabId: string;
  canDetachTab?: (tabId: string) => boolean;
  centerComponentTabs: EditorComponentInstance[];
  closeCenterComponent: (instanceId: string) => void;
  closeWorkspaceTab: (tabId: string) => void;
  detachWorkspaceTab?: (tabId: string) => void;
  selectWorkspaceTab: (tabId: string) => void;
  tabs: WorkspaceTabView[];
}) {
  return (
    <div className="workspace-tabs">
      {tabs.map((tab) => {
        const closeable = tab.id.startsWith("file:") || centerComponentTabs.some((instance) => instance.instanceId === tab.id);
        const detachable = Boolean(detachWorkspaceTab && canDetachTab?.(tab.id));
        const closeTab = () => {
          if (tab.id.startsWith("file:")) {
            closeWorkspaceTab(tab.id);
          } else {
            closeCenterComponent(tab.id);
          }
        };

        return (
          <div
            key={tab.id}
            className={`workspace-tab ${activeTabId === tab.id ? "active" : ""}`}
          >
            <button
              className="workspace-tab-select"
              type="button"
              onClick={() => selectWorkspaceTab(tab.id)}
            >
              <span
                className={`workspace-tab-dirty-dot ${tab.dirty ? "dirty" : "clean"}`}
                aria-label={tab.dirty ? "Unsaved changes" : "Clean"}
              />
              {tab.icon}
              <span className="workspace-tab-title">{tab.title}</span>
            </button>
            {detachable ? (
              /* @codemap anchor:workspace-tab-detach-action domain:workspace role:detached-workspace priority:P1 layer:app tags:tabs,detach,workspace-window */
              <button
                className="workspace-tab-detach"
                type="button"
                title="Detach to workspace window"
                onClick={(event) => {
                  event.stopPropagation();
                  detachWorkspaceTab?.(tab.id);
                }}
              >
                {"\u2197"}
              </button>
            ) : null}
            {closeable ? (
              <button
                className="workspace-tab-close"
                type="button"
                title="Close tab"
                onClick={(event) => {
                  event.stopPropagation();
                  closeTab();
                }}
              >
                {"\u00d7"}
              </button>
            ) : null}
          </div>
        );
      })}
    </div>
  );
}
