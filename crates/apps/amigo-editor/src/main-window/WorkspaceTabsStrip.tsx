import type { EditorComponentInstance } from "../editor-components/componentTypes";
import type { WorkspaceTabView } from "./hooks/useWorkspaceTabs";

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
          <button
            key={tab.id}
            type="button"
            className={`workspace-tab ${activeTabId === tab.id ? "active" : ""}`}
            onClick={() => selectWorkspaceTab(tab.id)}
          >
            <span
              className={`workspace-tab-dirty-dot ${tab.dirty ? "dirty" : "clean"}`}
              aria-label={tab.dirty ? "Unsaved changes" : "Clean"}
            />
            {tab.icon}
            {tab.title}
            {detachable ? (
              <span
                className="workspace-tab-detach"
                role="button"
                tabIndex={0}
                title="Detach to workspace window"
                onClick={(event) => {
                  event.stopPropagation();
                  detachWorkspaceTab?.(tab.id);
                }}
                onKeyDown={(event) => {
                  if (event.key === "Enter" || event.key === " ") {
                    event.preventDefault();
                    event.stopPropagation();
                    detachWorkspaceTab?.(tab.id);
                  }
                }}
              >
                {"\u2197"}
              </span>
            ) : null}
            {closeable ? (
              <span
                className="workspace-tab-close"
                role="button"
                tabIndex={0}
                onClick={(event) => {
                  event.stopPropagation();
                  closeTab();
                }}
                onKeyDown={(event) => {
                  if (event.key === "Enter" || event.key === " ") {
                    event.preventDefault();
                    event.stopPropagation();
                    closeTab();
                  }
                }}
              >
                {"\u00d7"}
              </span>
            ) : null}
          </button>
        );
      })}
    </div>
  );
}
