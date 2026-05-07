import { WorkspaceComponentHost } from "./WorkspaceComponentHost";
import { WorkspaceTabsStrip } from "./WorkspaceTabsStrip";
import type { EditorComponentContext, EditorComponentInstance } from "../editor-components/componentTypes";
import type { EditorProjectFileContentDto, EditorProjectFileDto } from "../api/dto";
import type { WorkspaceRuntimeServices } from "./workspaceRuntimeServices";
import type { WorkspaceTabView } from "./hooks/useWorkspaceTabs";

export function MainWorkspaceCenter({
  activeCenterComponent,
  activeFile,
  activeFileComponent,
  activeFileContent,
  activeTabId,
  canDetachTab,
  centerComponentTabs,
  closeCenterComponent,
  closeWorkspaceTab,
  componentContext,
  detachWorkspaceTab,
  scenePreviewComponent,
  selectWorkspaceTab,
  showComponentSources,
  tabs,
  workspaceRuntimeServices,
}: {
  activeCenterComponent: EditorComponentInstance | null;
  activeFile: EditorProjectFileDto | null;
  activeFileComponent: EditorComponentInstance | null;
  activeFileContent?: EditorProjectFileContentDto | null;
  activeTabId: string;
  canDetachTab?: (tabId: string) => boolean;
  centerComponentTabs: EditorComponentInstance[];
  closeCenterComponent: (instanceId: string) => void;
  closeWorkspaceTab: (tabId: string) => void;
  componentContext: EditorComponentContext;
  detachWorkspaceTab?: (tabId: string) => void;
  scenePreviewComponent: EditorComponentInstance;
  selectWorkspaceTab: (tabId: string) => void;
  showComponentSources: boolean;
  tabs: WorkspaceTabView[];
  workspaceRuntimeServices: WorkspaceRuntimeServices;
}) {
  return (
    <section className="workspace-center">
      <WorkspaceTabsStrip
        activeTabId={activeTabId}
        canDetachTab={canDetachTab}
        centerComponentTabs={centerComponentTabs}
        closeCenterComponent={closeCenterComponent}
        closeWorkspaceTab={closeWorkspaceTab}
        detachWorkspaceTab={detachWorkspaceTab}
        selectWorkspaceTab={selectWorkspaceTab}
        tabs={tabs}
      />

      <div className="workspace-center-body">
        {activeCenterComponent ? (
          <WorkspaceComponentHost
            instance={activeCenterComponent}
            context={componentContext}
            showDebugSource={showComponentSources}
            services={workspaceRuntimeServices}
          />
        ) : activeFileComponent ? (
          <WorkspaceComponentHost
            instance={activeFileComponent}
            context={componentContext}
            showDebugSource={showComponentSources}
            services={{
              ...workspaceRuntimeServices,
              selectedFile: activeFile,
              selectedFileContent: activeFileContent ?? null,
            }}
          />
        ) : (
          <WorkspaceComponentHost
            instance={scenePreviewComponent}
            context={componentContext}
            showDebugSource={showComponentSources}
            services={workspaceRuntimeServices}
          />
        )}
      </div>
    </section>
  );
}
