import { useEffect, useMemo, useRef, useState } from "react";
import type React from "react";
import {
  AlertTriangle,
  ArrowLeft,
  Box,
  CheckCircle2,
  Command,
  Crosshair,
  Eye,
  FileCode2,
  FolderOpen,
  Layers3,
  ListTree,
  Maximize2,
  Package,
  PanelsTopLeft,
  Pause,
  Play,
  RefreshCcw,
  Save,
  Settings,
  ShieldCheck,
  SlidersHorizontal,
  Terminal,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { useEditorStore } from "../app/editorStore";
import { openModSettingsWindow, openSettingsWindow, openThemeWindow } from "../api/editorApi";
import type { EditorModDetailsDto, EditorProjectFileDto, EditorSceneSummaryDto, ScenePreviewDto } from "../api/dto";
import { componentMenuGroups } from "../editor-components/componentMenu";
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
import { useThemeService } from "../theme/themeService";
import { closeCurrentWindow, toggleFullscreenWindow } from "./windowControls";
import { DockAreaHost } from "./DockAreaHost";
import { componentTabs, fileDiagnosticsFor, findProjectFile, normalizePath, renderWorkspaceComponent } from "./workspacePanels";
import { resolveFileWorkspaceDescriptor } from "./workspaceResources";
import "./main-window.css";

const WORKSPACE_LAYOUT_STORAGE_KEY = "amigo-editor.workspace.component-layout.v1";

type PersistedWorkspaceComponentLayout = {
  leftInstanceId?: string;
  rightInstanceId?: string;
  bottomInstanceId?: string;
  sizes?: WorkspaceDockSizes;
};

type WorkspaceDockSizes = {
  leftWidth: number;
  rightWidth: number;
  bottomHeight: number;
};

const DEFAULT_WORKSPACE_DOCK_SIZES: WorkspaceDockSizes = {
  leftWidth: 360,
  rightWidth: 380,
  bottomHeight: 260,
};

const WORKSPACE_DOCK_SIZE_LIMITS = {
  leftWidth: { min: 240, max: 620 },
  rightWidth: { min: 280, max: 680 },
  bottomHeight: { min: 160, max: 520 },
} as const;

type WorkspaceToolboxActionId =
  | "preview.toggle"
  | "preview.fit"
  | "file.reveal"
  | "file.save"
  | "command.palette"
  | "layout.reset"
  | "panel.problems"
  | "panel.events"
  | "window.fullscreen"
  | "toolbox.configure"
  | "preview.regenerate"
  | "mod.validate";

type WorkspaceToolboxAction = {
  id: WorkspaceToolboxActionId;
  label: string;
  icon: LucideIcon;
  group: "preview" | "file" | "workspace" | "panels" | "system";
  enabled?: (context: {
    details: EditorModDetailsDto | null;
    selectedScene: EditorSceneSummaryDto | null;
    selectedFile: EditorProjectFileDto | null;
  }) => boolean;
};

const WORKSPACE_TOOLBOX_ACTIONS: WorkspaceToolboxAction[] = [
  {
    id: "preview.toggle",
    label: "Play / pause preview",
    icon: Play,
    group: "preview",
    enabled: ({ details, selectedScene }) => Boolean(details && selectedScene),
  },
  {
    id: "preview.fit",
    label: "Fit preview to workspace",
    icon: Crosshair,
    group: "preview",
    enabled: ({ selectedScene }) => Boolean(selectedScene),
  },
  {
    id: "file.reveal",
    label: "Reveal selected file",
    icon: Eye,
    group: "file",
    enabled: ({ selectedFile }) => Boolean(selectedFile),
  },
  {
    id: "file.save",
    label: "Save current file",
    icon: Save,
    group: "file",
    enabled: ({ selectedFile }) => Boolean(selectedFile),
  },
  {
    id: "command.palette",
    label: "Open command palette",
    icon: Command,
    group: "workspace",
  },
  {
    id: "layout.reset",
    label: "Reset layout",
    icon: PanelsTopLeft,
    group: "workspace",
  },
  {
    id: "panel.problems",
    label: "Show problems",
    icon: AlertTriangle,
    group: "panels",
  },
  {
    id: "panel.events",
    label: "Show event log",
    icon: Terminal,
    group: "panels",
  },
  {
    id: "window.fullscreen",
    label: "Toggle fullscreen",
    icon: Maximize2,
    group: "system",
  },
  {
    id: "toolbox.configure",
    label: "Configure toolbox",
    icon: SlidersHorizontal,
    group: "system",
  },
  {
    id: "preview.regenerate",
    label: "Regenerate scene preview",
    icon: RefreshCcw,
    group: "preview",
    enabled: ({ details, selectedScene }) => Boolean(details && selectedScene),
  },
  {
    id: "mod.validate",
    label: "Validate mod",
    icon: ShieldCheck,
    group: "workspace",
    enabled: ({ details }) => Boolean(details),
  },
];

const DEFAULT_WORKSPACE_TOOLBOX_ACTION_IDS: WorkspaceToolboxActionId[] = [
  "preview.toggle",
  "preview.fit",
  "file.reveal",
  "command.palette",
  "panel.problems",
  "panel.events",
  "window.fullscreen",
  "toolbox.configure",
];

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
    refreshProjectTree,
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
  const persistedLayout = readPersistedWorkspaceComponentLayout();
  const [leftInstanceId, setLeftInstanceId] = useState(persistedLayout.leftInstanceId ?? "assets.browser:singleton");
  const [rightInstanceId, setRightInstanceId] = useState(persistedLayout.rightInstanceId ?? "entity.inspector:singleton");
  const [bottomInstanceId, setBottomInstanceId] = useState(persistedLayout.bottomInstanceId ?? "diagnostics.problems:singleton");
  const [eventFilter, setEventFilter] = useState<string>("all");
  const [eventSessionFilter, setEventSessionFilter] = useState<string>("all");
  const [eventSourceFilter, setEventSourceFilter] = useState<string>("all");
  const [eventSearch, setEventSearch] = useState("");
  const [centerComponentTabs, setCenterComponentTabs] = useState<EditorComponentInstance[]>([]);
  const [componentToolbarState, setComponentToolbarState] = useState<Record<string, ComponentToolbarState>>({});
  const [dockSizes, setDockSizes] = useState<WorkspaceDockSizes>(() => normalizeDockSizes(persistedLayout.sizes));

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

  useEffect(() => {
    persistWorkspaceComponentLayout({ leftInstanceId, rightInstanceId, bottomInstanceId, sizes: dockSizes });
  }, [bottomInstanceId, dockSizes, leftInstanceId, rightInstanceId]);

  function resizeDock(sizeKey: keyof WorkspaceDockSizes, delta: number) {
    setDockSizes((current) => ({
      ...current,
      [sizeKey]: clampDockSize(sizeKey, current[sizeKey] + delta),
    }));
  }

  function resetDockSize(sizeKey: keyof WorkspaceDockSizes) {
    setDockSizes((current) => ({
      ...current,
      [sizeKey]: DEFAULT_WORKSPACE_DOCK_SIZES[sizeKey],
    }));
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

  const fileDiagnostics = selectedFile ? fileDiagnosticsFor(selectedFile, selectedFileContent) : [];
  const allProblems = [...problems, ...fileDiagnostics];
  const activeFileTabPath = state.activeWorkspaceTabId.startsWith("file:")
    ? state.activeWorkspaceTabId.slice("file:".length)
    : null;
  const activeFile = activeFileTabPath && projectTree ? findProjectFile(projectTree.root, activeFileTabPath) : selectedFile;
  const activeFileContent = details && activeFile ? state.projectFileContents[`${details.id}:${activeFile.relativePath}`] : undefined;
  const activeFileDescriptor = activeFile ? resolveFileWorkspaceDescriptor(activeFile) : null;

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
      setDockSizes(DEFAULT_WORKSPACE_DOCK_SIZES);
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

    if (actionId === "preview.regenerate" && details && selectedScene) {
      void regeneratePreview(details.id, selectedScene.id, true);
      return;
    }

    if (actionId === "mod.validate") {
      void validateSelectedMod();
    }
  }

  const toolboxContext = { details, selectedScene, selectedFile };
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
            toolbarState: toolbarStateFor(activeLeftInstance),
          })}
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
            renderWorkspaceComponent(
              centerComponentTabs.find((instance) => instance.instanceId === state.activeWorkspaceTabId)!,
              componentContext,
              { details, selectedFile, selectedFileContent },
            )
          ) : activeFile && activeFileTabPath ? (
            renderWorkspaceComponent(
              createComponentInstance({
                componentId: activeFileDescriptor?.componentId ?? "file.binary",
                context: {
                  fileKind: activeFileDescriptor?.fileKind ?? activeFile.kind,
                  filePath: activeFile.relativePath,
                },
                placement: { kind: "centerTab" },
                resourceUri: activeFile.relativePath,
                sessionId: session?.sessionId,
                titleOverride: activeFile.name,
              }),
              componentContext,
              {
                details,
                onFileDirtyChange: setFileDirty,
                onProjectTreeRefresh: () => {
                  if (details) {
                    void refreshProjectTree(details.id);
                  }
                },
                onRevealSelectedFile: () => void revealSelectedProjectFile(),
                selectedFile: activeFile,
                selectedFileContent: activeFileContent,
              },
            )
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
                onRevealSelectedFile: () => void revealSelectedProjectFile(),
                preview,
                previewPlaying: state.previewPlaying,
                previewTask,
                selectedFile,
                selectedFileContent,
                selectedScene,
                selectScene,
              },
            )
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
          {renderWorkspaceComponent(activeRightInstance, componentContext, {
            allProblems,
            details,
            selectedEntity,
            selectedFile,
            selectedScene,
            toolbarState: toolbarStateFor(activeRightInstance),
          })}
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
            toolbarState: toolbarStateFor(activeBottomInstance),
            windowEventRows,
          })}
        </DockAreaHost>
      </section>

      <footer className="workspace-statusbar window-statusbar">
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

function normalizeDockSizes(sizes?: Partial<WorkspaceDockSizes>): WorkspaceDockSizes {
  return {
    leftWidth: clampDockSize("leftWidth", sizes?.leftWidth ?? DEFAULT_WORKSPACE_DOCK_SIZES.leftWidth),
    rightWidth: clampDockSize("rightWidth", sizes?.rightWidth ?? DEFAULT_WORKSPACE_DOCK_SIZES.rightWidth),
    bottomHeight: clampDockSize("bottomHeight", sizes?.bottomHeight ?? DEFAULT_WORKSPACE_DOCK_SIZES.bottomHeight),
  };
}

function clampDockSize(sizeKey: keyof WorkspaceDockSizes, value: number): number {
  const limits = WORKSPACE_DOCK_SIZE_LIMITS[sizeKey];
  return Math.min(limits.max, Math.max(limits.min, Math.round(value)));
}

function WorkspaceResizeHandle({
  className,
  onDrag,
  onReset,
  orientation,
  title,
}: {
  className: string;
  onDrag: (delta: number) => void;
  onReset: () => void;
  orientation: "vertical" | "horizontal";
  title: string;
}) {
  const dragRef = useRef<{ pointerId: number; x: number; y: number } | null>(null);

  function handlePointerDown(event: React.PointerEvent<HTMLButtonElement>) {
    event.currentTarget.setPointerCapture(event.pointerId);
    dragRef.current = { pointerId: event.pointerId, x: event.clientX, y: event.clientY };
    document.body.classList.add("workspace-resizing");
  }

  function handlePointerMove(event: React.PointerEvent<HTMLButtonElement>) {
    const drag = dragRef.current;
    if (!drag || drag.pointerId !== event.pointerId) return;
    const delta = orientation === "vertical" ? event.clientX - drag.x : event.clientY - drag.y;
    if (delta === 0) return;
    onDrag(delta);
    dragRef.current = { pointerId: event.pointerId, x: event.clientX, y: event.clientY };
  }

  function endDrag(event: React.PointerEvent<HTMLButtonElement>) {
    if (dragRef.current?.pointerId === event.pointerId) {
      dragRef.current = null;
      document.body.classList.remove("workspace-resizing");
    }
  }

  return (
    <button
      aria-label={title}
      className={`workspace-resize-handle ${orientation} ${className}`}
      title={`${title}. Double click to reset.`}
      type="button"
      onDoubleClick={onReset}
      onPointerCancel={endDrag}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={endDrag}
    />
  );
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
