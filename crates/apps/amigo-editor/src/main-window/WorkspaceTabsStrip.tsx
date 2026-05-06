import type { EditorComponentInstance } from "../editor-components/componentTypes";
import type { WorkspaceTabView } from "./hooks/useWorkspaceTabs";

export function WorkspaceTabsStrip({
  activeTabId,
  centerComponentTabs,
  closeCenterComponent,
  closeWorkspaceTab,
  selectWorkspaceTab,
  tabs,
}: {
  activeTabId: string;
  centerComponentTabs: EditorComponentInstance[];
  closeCenterComponent: (instanceId: string) => void;
  closeWorkspaceTab: (tabId: string) => void;
  selectWorkspaceTab: (tabId: string) => void;
  tabs: WorkspaceTabView[];
}) {
  return (
    <div className="workspace-tabs">
      {tabs.map((tab) => {
        const closeable = tab.id.startsWith("file:") || centerComponentTabs.some((instance) => instance.instanceId === tab.id);
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
