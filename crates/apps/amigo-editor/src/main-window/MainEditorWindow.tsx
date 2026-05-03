import { useEffect, useMemo, useState } from "react";
import type React from "react";
import {
  AlertTriangle,
  ArrowLeft,
  Box,
  CheckCircle2,
  FileCode2,
  FolderOpen,
  Layers3,
  ListTree,
  Maximize2,
  Package,
  PanelsTopLeft,
  Play,
  RefreshCcw,
  Settings,
  ShieldCheck,
  Terminal,
} from "lucide-react";
import { useEditorStore } from "../app/editorStore";
import { openModSettingsWindow, openSettingsWindow, openThemeWindow } from "../api/editorApi";
import type { EditorModDetailsDto, EditorProjectFileDto, EditorSceneSummaryDto, ScenePreviewDto } from "../api/dto";
import { componentMenuGroups } from "../editor-components/componentMenu";
import { createComponentInstance } from "../editor-components/componentInstances";
import { editorComponentById, iconForEditorComponent } from "../editor-components/componentRegistry";
import type { EditorComponentContext, EditorComponentInstance } from "../editor-components/componentTypes";
import { ThemeButton } from "../theme/ThemeButton";
import { useThemeService } from "../theme/themeService";
import { closeCurrentWindow, toggleFullscreenWindow } from "./windowControls";
import { DockAreaHost } from "./DockAreaHost";
import { FileWorkspace, componentTabs, fileDiagnosticsFor, findProjectFile, normalizePath, renderWorkspaceComponent } from "./workspacePanels";
import "./main-window.css";

const WORKSPACE_LAYOUT_STORAGE_KEY = "amigo-editor.workspace.component-layout.v1";

type PersistedWorkspaceComponentLayout = {
  leftInstanceId?: string;
  rightInstanceId?: string;
  bottomInstanceId?: string;
};

function previewKey(modId: string, sceneId: string): string {
  return `${modId}:${sceneId}`;
}

function activePreview(details: EditorModDetailsDto | null, selectedSceneId: string | null, previews: Record<string, ScenePreviewDto>) {
  if (!details || !selectedSceneId) {
    return undefined;
  }
  return previews[previewKey(details.id, selectedSceneId)];
}

function formatTaskTime(value: number): string {
  return new Date(value).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
}

export function MainEditorWindow() {
  const {
    state,
    closeWorkspaceTab,
    createExpectedFolder,
    focusComponent,
    openComponent,
    returnToStartup,
    regeneratePreview,
    recordEvent,
    revealSelectedProjectFile,
    selectProjectFile,
    selectScene,
    selectSceneEntity,
    selectWorkspaceTab,
    setPreviewPlaying,
    validateSelectedMod,
  } = useEditorStore();
  const { activeThemeId } = useThemeService();
  const [componentMenuOpen, setComponentMenuOpen] = useState(false);
  const persistedLayout = readPersistedWorkspaceComponentLayout();
  const [leftInstanceId, setLeftInstanceId] = useState(persistedLayout.leftInstanceId ?? "project.explorer:singleton");
  const [rightInstanceId, setRightInstanceId] = useState(persistedLayout.rightInstanceId ?? "entity.inspector:singleton");
  const [bottomInstanceId, setBottomInstanceId] = useState(persistedLayout.bottomInstanceId ?? "diagnostics.problems:singleton");
  const [eventFilter, setEventFilter] = useState<string>("all");
  const [eventSessionFilter, setEventSessionFilter] = useState<string>("all");
  const [eventSourceFilter, setEventSourceFilter] = useState<string>("all");
  const [eventSearch, setEventSearch] = useState("");
  const [centerComponentTabs, setCenterComponentTabs] = useState<EditorComponentInstance[]>([]);

  const details = state.modDetails;
  const session = state.activeSession;
  const selectedScene = details?.scenes.find((scene) => scene.id === state.selectedSceneId) ?? details?.scenes[0] ?? null;
  const projectTree = details ? state.projectTrees[details.id] : undefined;
  const projectStructureTree = details ? state.projectStructureTrees[details.id] : undefined;
  const projectTreeTask = details ? state.tasks[`project-tree:${details.id}`] : undefined;
  const selectedFile = projectTree && state.selectedFilePath ? findProjectFile(projectTree.root, state.selectedFilePath) : null;
  const selectedFileContent = details && selectedFile ? state.projectFileContents[`${details.id}:${selectedFile.relativePath}`] : undefined;
  const preview = activePreview(details, selectedScene?.id ?? null, state.previews);
  const previewTask = details && selectedScene ? state.tasks[`preview:${details.id}:${selectedScene.id}`] : undefined;
  const runningTasks = Object.values(state.tasks).filter((task) => task.status === "running");
  const eventRows = state.events.slice(0, 8);
  const windowEventRows = state.windowEvents.slice(0, 12);
  const sceneDiagnostics = selectedScene?.diagnostics ?? [];
  const modDiagnostics = details?.diagnostics ?? [];
  const problems = [...modDiagnostics, ...sceneDiagnostics];
  const hierarchy = details && selectedScene ? state.sceneHierarchies[previewKey(details.id, selectedScene.id)] : undefined;
  const hierarchyTask = details && selectedScene ? state.tasks[`scene-hierarchy:${details.id}:${selectedScene.id}`] : undefined;
  const selectedEntity = hierarchy?.entities.find((entity) => entity.id === state.selectedEntityId) ?? hierarchy?.entities[0] ?? null;
  const componentContext: EditorComponentContext = {
    sessionId: session?.sessionId ?? null,
    modId: details?.id ?? session?.modId ?? null,
    selectedSceneId: selectedScene?.id ?? null,
    selectedEntityId: selectedEntity?.id ?? null,
    capabilities: details?.capabilities ?? [],
  };
  const leftDockInstances = useMemo(
    () => [
      createComponentInstance({ componentId: "project.explorer", placement: { kind: "leftDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "scenes.browser", placement: { kind: "leftDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "assets.browser", placement: { kind: "leftDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "scripts.browser", placement: { kind: "leftDock" }, sessionId: session?.sessionId }),
    ],
    [session?.sessionId],
  );
  const rightDockInstances = useMemo(
    () => [
      createComponentInstance({ componentId: "entity.inspector", placement: { kind: "rightDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "diagnostics.panel", placement: { kind: "rightDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "entity.properties", placement: { kind: "rightDock" }, sessionId: session?.sessionId }),
    ],
    [session?.sessionId],
  );
  const bottomDockInstances = useMemo(
    () => [
      createComponentInstance({ componentId: "diagnostics.problems", placement: { kind: "bottomDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "events.log", placement: { kind: "bottomDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "tasks.monitor", placement: { kind: "bottomDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "scripting.console", placement: { kind: "bottomDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "cache.preview", placement: { kind: "bottomDock" }, sessionId: session?.sessionId }),
    ],
    [session?.sessionId],
  );
  const activeLeftInstance = leftDockInstances.find((instance) => instance.instanceId === leftInstanceId) ?? leftDockInstances[0];
  const activeRightInstance = rightDockInstances.find((instance) => instance.instanceId === rightInstanceId) ?? rightDockInstances[0];
  const activeBottomInstance = bottomDockInstances.find((instance) => instance.instanceId === bottomInstanceId) ?? bottomDockInstances[0];

  useEffect(() => {
    persistWorkspaceComponentLayout({ leftInstanceId, rightInstanceId, bottomInstanceId });
  }, [bottomInstanceId, leftInstanceId, rightInstanceId]);

  function reportWindowOpenError(error: unknown) {
    window.alert(`Failed to open window: ${error instanceof Error ? error.message : String(error)}`);
  }

  const handleSelectProjectFile = (file: EditorProjectFileDto) => {
    const matchingScene = details?.scenes.find((scene) => {
      const normalizedDocument = normalizePath(scene.documentPath);
      const normalizedScript = normalizePath(scene.scriptPath);
      return normalizedDocument.endsWith(file.relativePath) || normalizedScript.endsWith(file.relativePath);
    });
    if (matchingScene) {
      void selectScene(matchingScene);
    }
    selectProjectFile(file);
  };

  const fileDiagnostics = selectedFile ? fileDiagnosticsFor(selectedFile, selectedFileContent) : [];
  const allProblems = [...problems, ...fileDiagnostics];

  const workspaceTabs = useMemo(() => {
    const tabs: Array<{ id: string; title: string; icon: React.ReactNode }> = selectedScene ? [
      { id: "scene-preview", title: `${selectedScene.label} Preview`, icon: <Play size={13} /> },
    ] : [{ id: "scene-preview", title: "Scene Preview", icon: <Play size={13} /> }];
    centerComponentTabs.forEach((instance) => {
      const definition = editorComponentById(instance.componentId);
      tabs.push({
        id: instance.instanceId,
        title: instance.titleOverride ?? definition?.title ?? instance.componentId,
        icon: definition ? iconForEditorComponent(definition.icon, 13) : <Box size={13} />,
      });
    });
    state.openedFilePaths.forEach((relativePath) => {
      const file = projectTree ? findProjectFile(projectTree.root, relativePath) : null;
      if (file) {
        tabs.push({ id: `file:${file.relativePath}`, title: file.name, icon: <FileCode2 size={13} /> });
      }
    });
    return tabs;
  }, [centerComponentTabs, projectTree, selectedScene, state.openedFilePaths]);

  function openCenterComponent(componentId: string) {
    const instance = createComponentInstance({
      componentId,
      placement: { kind: "centerTab" },
      sessionId: session?.sessionId,
    });
    setCenterComponentTabs((current) => current.some((candidate) => candidate.instanceId === instance.instanceId) ? current : [...current, instance]);
    selectWorkspaceTab(instance.instanceId);
    openComponent(componentId, { modId: details?.id ?? "", sessionId: session?.sessionId ?? "" });
  }

  function closeCenterComponent(instanceId: string) {
    setCenterComponentTabs((current) => current.filter((instance) => instance.instanceId !== instanceId));
    if (state.activeWorkspaceTabId === instanceId) {
      selectWorkspaceTab("scene-preview");
    }
  }

  return (
    <main className="main-window-shell">
      <header className="main-titlebar">
        <div className="main-brand">
          <div className="brand-mark">A</div>
          <strong>Amigo Editor</strong>
          <span>{session ? `workspace session ${session.sessionId}` : "workspace"}</span>
        </div>
        <nav className="main-menu" aria-label="Application menu">
          <button type="button">File</button>
          <button type="button">Edit</button>
          <button type="button">View</button>
          <span className="main-menu-popover-anchor">
            <button type="button" onClick={() => setComponentMenuOpen((open) => !open)}>Window</button>
            {componentMenuOpen ? (
              <ComponentMenu
                onOpen={(componentId) => {
                  openComponent(componentId, {
                    modId: details?.id ?? "",
                    sceneId: selectedScene?.id ?? "",
                    sessionId: session?.sessionId ?? "",
                  });
                  setComponentMenuOpen(false);
                }}
              />
            ) : null}
          </span>
          <button type="button" onClick={() => setComponentMenuOpen((open) => !open)}>Tools</button>
        </nav>
        <div className="titlebar-project-context">
          <span className="titlebar-project-summary">
            <strong>{details?.name ?? session?.modId ?? "No mod"}</strong>
            <small>{details ? `${details.id} · ${details.version}` : session?.rootPath ?? "No active session"}</small>
            <span className={`titlebar-status-dot status-${details?.status ?? "warning"}`} aria-label={details?.status ?? "session"} />
          </span>
          <span className="titlebar-separator" aria-hidden="true" />
          <button
            className="titlebar-action-button"
            type="button"
            onClick={async () => {
              if (state.hasDirtyState) {
                recordEvent({ type: "WorkspaceCloseBlocked", dirtyFileCount: Object.keys(state.dirtyFiles).length });
                const shouldClose = window.confirm("This workspace has unsaved changes. Discard changes and close?");
                if (!shouldClose) {
                  return;
                }
                recordEvent({ type: "WorkspaceCloseConfirmed" });
              }
              await returnToStartup();
              await closeCurrentWindow(session?.sessionId);
            }}
          >
            <ArrowLeft size={15} />
            Close Workspace
          </button>
        </div>
      </header>

      <section className="main-topbar">
        <div className="main-toolbar">
          <button
            className="button button-tool"
            type="button"
            disabled={!details || !selectedScene}
            onClick={() => setPreviewPlaying(!state.previewPlaying)}
          >
            <Play size={14} />
            {state.previewPlaying ? "Pause Preview" : "Play Preview"}
          </button>
          <button
            className="button button-tool"
            type="button"
            disabled={!details || !selectedScene}
            onClick={() => details && selectedScene ? void regeneratePreview(details.id, selectedScene.id, true) : undefined}
          >
            <RefreshCcw size={14} />
            Regenerate
          </button>
          <button className="button button-tool" type="button">
            <ShieldCheck size={14} />
            Validate
          </button>
          <button className="button button-tool" type="button" onClick={() => recordEvent({ type: "LayoutResetRequested" })}>
            <PanelsTopLeft size={14} />
            Reset Layout
          </button>
          <button className="button button-tool" type="button" onClick={() => void toggleFullscreenWindow()}>
            <Maximize2 size={14} />
            Fullscreen
          </button>
        </div>

        <div className="main-toolbar main-toolbar-right">
          <ThemeButton onClick={() => void openThemeWindow().catch(reportWindowOpenError)} />
          <button
            className="button button-ghost"
            type="button"
            onClick={() =>
              void (session ? openModSettingsWindow(session.sessionId) : openSettingsWindow()).catch(reportWindowOpenError)
            }
          >
            <Settings size={15} />
            Settings
          </button>
        </div>
      </section>

      <section className="workspace-grid">
        <DockAreaHost
          className="dock-left"
          tabs={componentTabs(leftDockInstances)}
          activeTab={activeLeftInstance.instanceId}
          onSelect={(instanceId) => {
            const instance = leftDockInstances.find((candidate) => candidate.instanceId === instanceId);
            if (!instance) return;
            setLeftInstanceId(instanceId);
            focusComponent(instance.instanceId, instance.componentId);
            recordEvent({ type: "DockTabSelected", dock: "left", tabId: instanceId });
          }}
        >
          {renderWorkspaceComponent(activeLeftInstance, componentContext, {
            allProblems,
            details,
            eventFilter,
            eventRows,
            eventSearch,
            eventSessionFilter,
            eventSourceFilter,
            handleSelectProjectFile,
            hierarchy,
            hierarchyTask,
            onCreateExpectedFolder: createExpectedFolder,
            onProjectNodeActivated: (node) => {
              if (!details) return;
              recordEvent({ type: "ProjectTreeNodeActivated", modId: details.id, nodeId: node.id, kind: node.kind });
              if (node.kind === "overview") {
                openCenterComponent("project.overview");
              }
              if (node.kind === "capabilities") {
                openCenterComponent("project.capabilities");
              }
              if (node.kind === "dependencies") {
                openCenterComponent("project.dependencies");
              }
              if (node.kind === "diagnostics") {
                setBottomInstanceId("diagnostics.problems:singleton");
              }
              if (node.kind === "manifest") {
                void validateSelectedMod();
              }
            },
            preview,
            projectTree,
            projectStructureTree,
            projectTreeTask,
            selectedEntity,
            selectedFile,
            selectedScene,
            selectScene,
            selectSceneEntity,
            setEventFilter,
            setEventSearch,
            setEventSessionFilter,
            setEventSourceFilter,
            windowEventRows,
          })}
        </DockAreaHost>

        <section className="workspace-center">
          <div className="workspace-tabs">
            {workspaceTabs.map((tab, index) => (
              <button
                key={tab.id}
                type="button"
                className={`workspace-tab ${state.activeWorkspaceTabId === tab.id ? "active" : ""}`}
                onClick={() => selectWorkspaceTab(tab.id)}
              >
                {tab.icon}
                {tab.title}
                {tab.id.startsWith("file:") || centerComponentTabs.some((instance) => instance.instanceId === tab.id) ? (
                  <span
                    className="workspace-tab-close"
                    role="button"
                    tabIndex={0}
                    onClick={(event) => {
                      event.stopPropagation();
                      if (tab.id.startsWith("file:")) {
                        closeWorkspaceTab(tab.id);
                      } else {
                        closeCenterComponent(tab.id);
                      }
                    }}
                    onKeyDown={(event) => {
                      if (event.key === "Enter" || event.key === " ") {
                        event.preventDefault();
                        event.stopPropagation();
                        if (tab.id.startsWith("file:")) {
                          closeWorkspaceTab(tab.id);
                        } else {
                          closeCenterComponent(tab.id);
                        }
                      }
                    }}
                  >
                    ×
                  </span>
                ) : null}
              </button>
            ))}
          </div>

          {centerComponentTabs.some((instance) => instance.instanceId === state.activeWorkspaceTabId) ? (
            renderWorkspaceComponent(
              centerComponentTabs.find((instance) => instance.instanceId === state.activeWorkspaceTabId)!,
              componentContext,
              { details },
            )
          ) : selectedFile && state.activeWorkspaceTabId === `file:${selectedFile.relativePath}` ? (
            <FileWorkspace file={selectedFile} content={selectedFileContent} onReveal={() => void revealSelectedProjectFile()} />
          ) : (
            renderWorkspaceComponent(
              createComponentInstance({
                componentId: "scene.preview",
                context: { sceneId: selectedScene?.id ?? "" },
                placement: { kind: "centerTab" },
                sessionId: session?.sessionId,
              }),
              componentContext,
              {
                details,
                preview,
                previewPlaying: state.previewPlaying,
                previewTask,
                selectedScene,
                selectScene,
              },
            )
          )}
        </section>

        <DockAreaHost
          className="dock-right"
          tabs={componentTabs(rightDockInstances)}
          activeTab={activeRightInstance.instanceId}
          onSelect={(instanceId) => {
            const instance = rightDockInstances.find((candidate) => candidate.instanceId === instanceId);
            if (!instance) return;
            setRightInstanceId(instanceId);
            focusComponent(instance.instanceId, instance.componentId);
            recordEvent({ type: "DockTabSelected", dock: "right", tabId: instanceId });
          }}
        >
          {renderWorkspaceComponent(activeRightInstance, componentContext, {
            allProblems,
            details,
            selectedEntity,
            selectedFile,
            selectedScene,
          })}
        </DockAreaHost>

        <DockAreaHost
          className="dock-bottom"
          tabs={componentTabs(bottomDockInstances)}
          activeTab={activeBottomInstance.instanceId}
          onSelect={(instanceId) => {
            const instance = bottomDockInstances.find((candidate) => candidate.instanceId === instanceId);
            if (!instance) return;
            setBottomInstanceId(instanceId);
            focusComponent(instance.instanceId, instance.componentId);
            recordEvent({ type: "DockTabSelected", dock: "bottom", tabId: instanceId });
          }}
        >
          {renderWorkspaceComponent(activeBottomInstance, componentContext, {
            allProblems,
            details,
            eventFilter,
            eventRows,
            eventSearch,
            eventSessionFilter,
            eventSourceFilter,
            preview,
            tasks: Object.values(state.tasks),
            setEventFilter,
            setEventSearch,
            setEventSessionFilter,
            setEventSourceFilter,
            windowEventRows,
          })}
        </DockAreaHost>
      </section>

      <footer className="workspace-statusbar">
        <span><span className="status-dot" />Ready</span>
        <span>{details?.name ?? "No mod"}</span>
        <span>{details?.sceneCount ?? 0} scenes</span>
        <span>{details?.contentSummary.totalFiles ?? 0} files</span>
        <span>Theme: {activeThemeId}</span>
        <span>{runningTasks.length} tasks running</span>
      </footer>
    </main>
  );
}

function readPersistedWorkspaceComponentLayout(): PersistedWorkspaceComponentLayout {
  try {
    const text = window.localStorage.getItem(WORKSPACE_LAYOUT_STORAGE_KEY);
    return text ? (JSON.parse(text) as PersistedWorkspaceComponentLayout) : {};
  } catch {
    return {};
  }
}

function persistWorkspaceComponentLayout(layout: PersistedWorkspaceComponentLayout) {
  window.localStorage.setItem(WORKSPACE_LAYOUT_STORAGE_KEY, JSON.stringify(layout));
}

function ComponentMenu({ onOpen }: { onOpen: (componentId: string) => void }) {
  const groups = componentMenuGroups().filter((group) =>
    group.components.some((component) => component.canDock || component.canOpenInWindow || component.canOpenInCenterTabs),
  );

  return (
    <div className="component-menu-popover">
      {groups.map((group) => (
        <section key={group.category}>
          <h3>{group.category}</h3>
          {group.components.map((component) => (
            <button key={component.id} type="button" onClick={() => onOpen(component.id)}>
              {iconForEditorComponent(component.icon, 13)}
              <span>
                <strong>{component.title}</strong>
                <small>{component.domain}{component.subdomain ? ` · ${component.subdomain}` : ""}</small>
              </span>
            </button>
          ))}
        </section>
      ))}
    </div>
  );
}
