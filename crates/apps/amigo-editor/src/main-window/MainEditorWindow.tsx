import { useMemo, useState } from "react";
import type React from "react";
import {
  ArrowLeft,
  Box,
  FileCode2,
  Pause,
  Play,
  Settings,
} from "lucide-react";
import { useEditorStore } from "../app/editorStore";
import {
  activePreview as selectActivePreview,
  resolvedSelection as selectResolvedSelection,
  selectedAsset as selectSelectedAsset,
  selectedEntity as selectSelectedEntity,
  selectedFile as selectSelectedFile,
  selectedHierarchy as selectSelectedHierarchy,
  selectedScene as selectSelectedScene,
} from "../app/store/editorSelectors";
import { openModSettingsWindow, openSettingsWindow, openThemeWindow } from "../api/editorApi";
import type { EditorProjectFileDto } from "../api/dto";
import { ComponentToolbar, defaultToolbarState } from "../editor-components/ComponentToolbar";
import { createComponentInstance } from "../editor-components/componentInstances";
import { editorComponentById, iconForEditorComponent } from "../editor-components/componentRegistry";
import type {
  ComponentToolbarState,
  ComponentToolbarValue,
  EditorComponentContext,
  EditorComponentInstance,
} from "../editor-components/componentTypes";
import { ThemeButton } from "../theme/ThemeButton";
import { themeNameForId } from "../theme/themeRegistry";
import { useThemeService } from "../theme/themeService";
import { closeCurrentWindow, toggleFullscreenWindow } from "./windowControls";
import { ComponentMenu } from "./ComponentMenu";
import { DockAreaHost } from "./DockAreaHost";
import { WorkspaceComponentHost } from "./WorkspaceComponentHost";
import { WorkspaceResizeHandle } from "./WorkspaceResizeHandle";
import { fileDiagnosticsFor, findProjectFile, normalizePath } from "../features/files/fileTreeSelectors";
import { PROJECT_NODE_ACTIONS } from "../features/project/projectNodeActions";
import { componentTabs } from "./workspaceTabs";
import { resolveFileWorkspaceDescriptor } from "../features/files/fileWorkspaceRules";
import {
  DEFAULT_WORKSPACE_TOOLBOX_ACTION_IDS,
  WORKSPACE_TOOLBOX_ACTIONS,
  type WorkspaceToolboxAction,
  type WorkspaceToolboxActionId,
} from "./toolboxRegistry";
import { useWorkspaceLayout } from "./useWorkspaceLayout";
import "./main-window.css";

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
    refreshProjectTree,
    selectAsset,
    selectProjectFile,
    selectScene,
    selectSceneEntity,
    selectWorkspaceTab,
    setFileDirty,
    setPreviewPlaying,
    validateSelectedMod,
  } = useEditorStore();
  const { activeThemeId } = useThemeService();
  const [componentMenuOpen, setComponentMenuOpen] = useState(false);
  const {
    bottomInstanceId,
    dockSizes,
    leftInstanceId,
    resetDockSize,
    resetLayout,
    resizeDock,
    rightInstanceId,
    setBottomInstanceId,
    setLeftInstanceId,
    setRightInstanceId,
  } = useWorkspaceLayout();
  const [eventFilter, setEventFilter] = useState<string>("all");
  const [eventSessionFilter, setEventSessionFilter] = useState<string>("all");
  const [eventSourceFilter, setEventSourceFilter] = useState<string>("all");
  const [eventSearch, setEventSearch] = useState("");
  const [centerComponentTabs, setCenterComponentTabs] = useState<EditorComponentInstance[]>([]);
  const [componentToolbarState, setComponentToolbarState] = useState<Record<string, ComponentToolbarState>>({});

  const details = state.modDetails;
  const session = state.activeSession;
  const selectedSceneValue = selectSelectedScene(state);
  const projectTree = details ? state.projectTrees[details.id] : undefined;
  const projectStructureTree = details ? state.projectStructureTrees[details.id] : undefined;
  const projectTreeTask = details ? state.tasks[`project-tree:${details.id}`] : undefined;
  const selectedFileValue = selectSelectedFile(state, projectTree);
  const selectedFileContent = details && selectedFileValue ? state.projectFileContents[`${details.id}:${selectedFileValue.relativePath}`] : undefined;
  const preview = selectActivePreview(details, selectedSceneValue?.id ?? null, state.previews);
  const previewTask = details && selectedSceneValue ? state.tasks[`preview:${details.id}:${selectedSceneValue.id}`] : undefined;
  const runningTasks = Object.values(state.tasks).filter((task) => task.status === "running");
  const eventRows = state.events.slice(0, 8);
  const windowEventRows = state.windowEvents.slice(0, 12);
  const sceneDiagnostics = selectedSceneValue?.diagnostics ?? [];
  const modDiagnostics = details?.diagnostics ?? [];
  const problems = [...modDiagnostics, ...sceneDiagnostics];
  const hierarchy = selectSelectedHierarchy(details, selectedSceneValue, state.sceneHierarchies);
  const hierarchyTask = details && selectedSceneValue ? state.tasks[`scene-hierarchy:${details.id}:${selectedSceneValue.id}`] : undefined;
  const selectedEntityValue = selectSelectedEntity(state, hierarchy);
  const selectedAssetValue = selectSelectedAsset(state, projectTree);
  const resolvedSelection = selectResolvedSelection(state, projectTree, selectedFileContent ?? null);
  const componentContext: EditorComponentContext = {
    sessionId: session?.sessionId ?? null,
    modId: details?.id ?? session?.modId ?? null,
    selectedSceneId: selectedSceneValue?.id ?? null,
    selectedEntityId: selectedEntityValue?.id ?? null,
    selectedAssetId: selectedAssetValue?.assetKey ?? null,
    capabilities: details?.capabilities ?? [],
  };
  const leftDockInstances = useMemo(
    () => [
      createComponentInstance({ componentId: "assets.browser", placement: { kind: "leftDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "files.browser", placement: { kind: "leftDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "scenes.browser", placement: { kind: "leftDock" }, sessionId: session?.sessionId }),
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

  function toolbarStateFor(instance: EditorComponentInstance): ComponentToolbarState {
    const toolbar = editorComponentById(instance.componentId)?.toolbar;
    return {
      ...defaultToolbarState(toolbar),
      ...(componentToolbarState[instance.instanceId] ?? {}),
    };
  }

  function setToolbarValue(instance: EditorComponentInstance, controlId: string, value: ComponentToolbarValue) {
    setComponentToolbarState((current) => ({
      ...current,
      [instance.instanceId]: {
        ...defaultToolbarState(editorComponentById(instance.componentId)?.toolbar),
        ...(current[instance.instanceId] ?? {}),
        [controlId]: value,
      },
    }));
  }

  function runComponentToolbarAction(instance: EditorComponentInstance, controlId: string) {
    if (instance.componentId === "assets.browser" && controlId === "refresh" && details) {
      setToolbarValue(instance, "refreshNonce", String(Date.now()));
      void refreshProjectTree(details.id);
    }
  }

  function renderComponentToolbar(instance: EditorComponentInstance) {
    const toolbar = editorComponentById(instance.componentId)?.toolbar;
    if (!toolbar) return null;
    return (
      <ComponentToolbar
        toolbar={toolbar}
        state={toolbarStateFor(instance)}
        onChange={(controlId, value) => setToolbarValue(instance, controlId, value)}
        onAction={(controlId) => runComponentToolbarAction(instance, controlId)}
      />
    );
  }

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

  const fileDiagnostics = selectedFileValue ? fileDiagnosticsFor(selectedFileValue, selectedFileContent) : [];
  const allProblems = [...problems, ...fileDiagnostics];
  const activeFileTabPath = state.activeWorkspaceTabId.startsWith("file:")
    ? state.activeWorkspaceTabId.slice("file:".length)
    : null;
  const activeFile = activeFileTabPath && projectTree ? findProjectFile(projectTree.root, activeFileTabPath) : selectedFileValue;
  const activeFileContent = details && activeFile ? state.projectFileContents[`${details.id}:${activeFile.relativePath}`] : undefined;
  const activeFileDescriptor = activeFile ? resolveFileWorkspaceDescriptor(activeFile) : null;

  const workspaceTabs = useMemo(() => {
    const tabs: Array<{ id: string; title: string; icon: React.ReactNode }> = selectedSceneValue ? [
      { id: "scene-preview", title: `${selectedSceneValue.label} Preview`, icon: <Play size={13} /> },
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
  }, [centerComponentTabs, projectTree, selectedSceneValue, state.openedFilePaths]);

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

  function runToolboxAction(actionId: WorkspaceToolboxActionId) {
    recordEvent({ type: "WorkspaceToolboxActionTriggered", actionId });

    if (actionId === "preview.toggle") {
      setPreviewPlaying(!state.previewPlaying);
      return;
    }

    if (actionId === "file.reveal") {
      void revealSelectedProjectFile();
      return;
    }

    if (actionId === "layout.reset") {
      resetLayout();
      recordEvent({ type: "LayoutResetRequested" });
      return;
    }

    if (actionId === "panel.problems") {
      setBottomInstanceId("diagnostics.problems:singleton");
      return;
    }

    if (actionId === "panel.events") {
      setBottomInstanceId("events.log:singleton");
      return;
    }

    if (actionId === "window.fullscreen") {
      void toggleFullscreenWindow();
      return;
    }

    if (actionId === "toolbox.configure") {
      recordEvent({ type: "WorkspaceToolboxConfigureRequested" });
      return;
    }

    if (actionId === "preview.regenerate" && details && selectedSceneValue) {
      void regeneratePreview(details.id, selectedSceneValue.id, true);
      return;
    }

    if (actionId === "mod.validate") {
      void validateSelectedMod();
    }
  }

  const toolboxContext = { details, selectedScene: selectedSceneValue, selectedFile: selectedFileValue };
  const pinnedToolboxActions = DEFAULT_WORKSPACE_TOOLBOX_ACTION_IDS
    .map((actionId) => WORKSPACE_TOOLBOX_ACTIONS.find((action) => action.id === actionId))
    .filter((action): action is WorkspaceToolboxAction => Boolean(action));

  return (
    <main className="main-window-shell window-shell workspace-window-shell">
      <header className="main-titlebar window-titlebar">
        <div className="main-brand window-brand">
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
                    sceneId: selectedSceneValue?.id ?? "",
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

      <section className="main-topbar window-topbar">
        <div className="main-toolbar workspace-toolbox" aria-label="Workspace toolbox">
          {pinnedToolboxActions.map((action, index) => {
            const Icon = action.id === "preview.toggle" && state.previewPlaying ? Pause : action.icon;
            const previous = pinnedToolboxActions[index - 1];
            const separated = previous && previous.group !== action.group;
            return (
              <button
                key={action.id}
                className={`workspace-toolbox-button ${separated ? "separated" : ""}`}
                type="button"
                disabled={action.enabled ? !action.enabled(toolboxContext) : false}
                title={action.label}
                aria-label={action.label}
                onClick={() => runToolboxAction(action.id)}
              >
                <Icon size={14} />
              </button>
            );
          })}
        </div>

        <div className="main-toolbar main-toolbar-right" aria-hidden="true" />
      </section>

      <section
        className="workspace-grid"
        style={{
          "--left-dock-width": `${dockSizes.leftWidth}px`,
          "--right-dock-width": `${dockSizes.rightWidth}px`,
          "--bottom-dock-height": `${dockSizes.bottomHeight}px`,
        } as React.CSSProperties}
      >
        <DockAreaHost
          className="dock-left"
          tabs={componentTabs(leftDockInstances)}
          activeTab={activeLeftInstance.instanceId}
          toolbar={renderComponentToolbar(activeLeftInstance)}
          onSelect={(instanceId) => {
            const instance = leftDockInstances.find((candidate) => candidate.instanceId === instanceId);
            if (!instance) return;
            setLeftInstanceId(instanceId);
            focusComponent(instance.instanceId, instance.componentId);
            recordEvent({ type: "DockTabSelected", dock: "left", tabId: instanceId });
          }}
        >
          <WorkspaceComponentHost
            instance={activeLeftInstance}
            context={componentContext}
            services={{
            allProblems,
            details,
            eventFilter,
            eventRows,
            eventSearch,
            eventSessionFilter,
            eventSourceFilter,
            handleSelectProjectFile,
            handleSelectAsset: selectAsset,
            hierarchy,
            hierarchyTask,
            onCreateExpectedFolder: createExpectedFolder,
            onProjectNodeActivated: (node) => {
              if (!details) return;
              recordEvent({ type: "ProjectTreeNodeActivated", modId: details.id, nodeId: node.id, kind: node.kind });
              const action = PROJECT_NODE_ACTIONS.find((candidate) => candidate.canRun(node));
              void action?.run(node, {
                openCenterComponent,
                showBottomPanel: setBottomInstanceId,
                validateSelectedMod,
              });
            },
            preview,
            projectTree,
            projectStructureTree,
            projectTreeTask,
            selection: resolvedSelection,
            selectedEntity: selectedEntityValue,
            selectedAsset: selectedAssetValue,
            selectedFile: selectedFileValue,
            selectedScene: selectedSceneValue,
            selectScene,
            selectSceneEntity,
            setEventFilter,
            setEventSearch,
            setEventSessionFilter,
            setEventSourceFilter,
            windowEventRows,
            toolbarState: toolbarStateFor(activeLeftInstance),
            }}
          />
        </DockAreaHost>

        <WorkspaceResizeHandle
          className="resize-left"
          orientation="vertical"
          title="Resize left dock"
          onDrag={(delta) => resizeDock("leftWidth", delta)}
          onReset={() => resetDockSize("leftWidth")}
        />

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
            <WorkspaceComponentHost
              instance={centerComponentTabs.find((instance) => instance.instanceId === state.activeWorkspaceTabId)!}
              context={componentContext}
              services={{ details, selectedFile: selectedFileValue, selectedFileContent, selection: resolvedSelection }}
            />
          ) : activeFile && activeFileTabPath ? (
            <WorkspaceComponentHost
              instance={createComponentInstance({
                componentId: activeFileDescriptor?.componentId ?? "file.binary",
                context: {
                  fileKind: activeFileDescriptor?.fileKind ?? activeFile.kind,
                  filePath: activeFile.relativePath,
                },
                placement: { kind: "centerTab" },
                resourceUri: activeFile.relativePath,
                sessionId: session?.sessionId,
                titleOverride: activeFile.name,
              })}
              context={componentContext}
              services={{
                details,
                onFileDirtyChange: setFileDirty,
                onProjectTreeRefresh: () => {
                  if (details) {
                    void refreshProjectTree(details.id);
                  }
                },
                onRevealSelectedFile: () => void revealSelectedProjectFile(),
                selection: resolvedSelection,
                selectedFile: activeFile,
                selectedFileContent: activeFileContent,
              }}
            />
          ) : (
            <WorkspaceComponentHost
              instance={createComponentInstance({
                componentId: "scene.preview",
                context: { sceneId: selectedSceneValue?.id ?? "" },
                placement: { kind: "centerTab" },
                sessionId: session?.sessionId,
              })}
              context={componentContext}
              services={{
                details,
                onRevealSelectedFile: () => void revealSelectedProjectFile(),
                preview,
                previewPlaying: state.previewPlaying,
                previewTask,
                selection: resolvedSelection,
                selectedFile: selectedFileValue,
                selectedFileContent,
                selectedScene: selectedSceneValue,
                selectScene,
              }}
            />
          )}
        </section>

        <WorkspaceResizeHandle
          className="resize-right"
          orientation="vertical"
          title="Resize right dock"
          onDrag={(delta) => resizeDock("rightWidth", -delta)}
          onReset={() => resetDockSize("rightWidth")}
        />

        <DockAreaHost
          className="dock-right"
          tabs={componentTabs(rightDockInstances)}
          activeTab={activeRightInstance.instanceId}
          toolbar={renderComponentToolbar(activeRightInstance)}
          onSelect={(instanceId) => {
            const instance = rightDockInstances.find((candidate) => candidate.instanceId === instanceId);
            if (!instance) return;
            setRightInstanceId(instanceId);
            focusComponent(instance.instanceId, instance.componentId);
            recordEvent({ type: "DockTabSelected", dock: "right", tabId: instanceId });
          }}
        >
          <WorkspaceComponentHost
            instance={activeRightInstance}
            context={componentContext}
            services={{
            allProblems,
            details,
            selection: resolvedSelection,
            selectedEntity: selectedEntityValue,
            selectedAsset: selectedAssetValue,
            selectedFile: selectedFileValue,
            selectedScene: selectedSceneValue,
            toolbarState: toolbarStateFor(activeRightInstance),
            }}
          />
        </DockAreaHost>

        <WorkspaceResizeHandle
          className="resize-bottom"
          orientation="horizontal"
          title="Resize bottom dock"
          onDrag={(delta) => resizeDock("bottomHeight", -delta)}
          onReset={() => resetDockSize("bottomHeight")}
        />

        <DockAreaHost
          className="dock-bottom"
          tabs={componentTabs(bottomDockInstances)}
          activeTab={activeBottomInstance.instanceId}
          toolbar={renderComponentToolbar(activeBottomInstance)}
          onSelect={(instanceId) => {
            const instance = bottomDockInstances.find((candidate) => candidate.instanceId === instanceId);
            if (!instance) return;
            setBottomInstanceId(instanceId);
            focusComponent(instance.instanceId, instance.componentId);
            recordEvent({ type: "DockTabSelected", dock: "bottom", tabId: instanceId });
          }}
        >
          <WorkspaceComponentHost
            instance={activeBottomInstance}
            context={componentContext}
            services={{
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
            toolbarState: toolbarStateFor(activeBottomInstance),
            windowEventRows,
            }}
          />
        </DockAreaHost>
      </section>

      <footer className="workspace-statusbar window-statusbar">
        <span><span className="status-dot" />Ready</span>
        <span>{details?.name ?? "No mod"}</span>
        <span>{details?.sceneCount ?? 0} scenes</span>
        <span>{details?.contentSummary.totalFiles ?? 0} files</span>
        <span>Theme: {themeNameForId(activeThemeId)}</span>
        <span>{runningTasks.length} tasks running</span>
      </footer>
    </main>
  );
}
