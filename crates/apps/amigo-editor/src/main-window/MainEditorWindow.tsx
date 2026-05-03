import { useMemo, useState } from "react";
import type React from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import {
  AlertTriangle,
  ArrowLeft,
  Box,
  CheckCircle2,
  FileCode2,
  Folder,
  FolderOpen,
  Gauge,
  Layers3,
  ListTree,
  Package,
  PanelsTopLeft,
  Play,
  RefreshCcw,
  Settings,
  ShieldCheck,
  Terminal,
} from "lucide-react";
import { useEditorStore } from "../app/editorStore";
import type { EditorModDetailsDto, EditorProjectFileDto, EditorProjectTreeDto, EditorSceneEntityDto, EditorSceneHierarchyDto, EditorSceneSummaryDto, ScenePreviewDto } from "../api/dto";
import type { DockPlugin } from "../dock/dockTypes";
import { dockPluginById } from "../dock/dockRegistry";
import { SettingsDialog } from "../settings/SettingsDialog";
import { DiagnosticsList } from "../startup/DiagnosticsList";
import { EngineSlideshowPreview } from "../startup/EngineSlideshowPreview";
import { ThemeButton } from "../theme/ThemeButton";
import { ThemeControllerDialog } from "../theme/ThemeControllerDialog";
import { useThemeService } from "../theme/themeService";
import { DEFAULT_WORKSPACE_LAYOUT } from "./workspaceLayout";
import "./main-window.css";

type LeftDockTab = "project-explorer" | "asset-browser" | "scene-hierarchy";
type RightDockTab = "inspector" | "diagnostics" | "properties";
type BottomDockTab = "problems" | "event-log" | "tasks" | "console" | "preview-cache";

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

function isDockPlugin(plugin: DockPlugin | undefined): plugin is DockPlugin {
  return Boolean(plugin);
}

export function MainEditorWindow() {
  const {
    state,
    closeWorkspaceTab,
    returnToStartup,
    regeneratePreview,
    recordEvent,
    revealSelectedProjectFile,
    selectProjectFile,
    selectScene,
    selectSceneEntity,
    selectWorkspaceTab,
    setPreviewPlaying,
  } = useEditorStore();
  const { activeThemeId } = useThemeService();
  const [leftTab, setLeftTab] = useState<LeftDockTab>("project-explorer");
  const [rightTab, setRightTab] = useState<RightDockTab>("inspector");
  const [bottomTab, setBottomTab] = useState<BottomDockTab>("problems");
  const [themeDialogOpen, setThemeDialogOpen] = useState(false);
  const [settingsDialogOpen, setSettingsDialogOpen] = useState(false);

  const details = state.modDetails;
  const session = state.activeSession;
  const selectedScene = details?.scenes.find((scene) => scene.id === state.selectedSceneId) ?? details?.scenes[0] ?? null;
  const projectTree = details ? state.projectTrees[details.id] : undefined;
  const projectTreeTask = details ? state.tasks[`project-tree:${details.id}`] : undefined;
  const selectedFile = projectTree && state.selectedFilePath ? findProjectFile(projectTree.root, state.selectedFilePath) : null;
  const selectedFileContent = details && selectedFile ? state.projectFileContents[`${details.id}:${selectedFile.relativePath}`] : undefined;
  const preview = activePreview(details, selectedScene?.id ?? null, state.previews);
  const previewTask = details && selectedScene ? state.tasks[`preview:${details.id}:${selectedScene.id}`] : undefined;
  const runningTasks = Object.values(state.tasks).filter((task) => task.status === "running");
  const eventRows = state.events.slice(0, 8);
  const sceneDiagnostics = selectedScene?.diagnostics ?? [];
  const modDiagnostics = details?.diagnostics ?? [];
  const problems = [...modDiagnostics, ...sceneDiagnostics];
  const hierarchy = details && selectedScene ? state.sceneHierarchies[previewKey(details.id, selectedScene.id)] : undefined;
  const hierarchyTask = details && selectedScene ? state.tasks[`scene-hierarchy:${details.id}:${selectedScene.id}`] : undefined;
  const selectedEntity = hierarchy?.entities.find((entity) => entity.id === state.selectedEntityId) ?? hierarchy?.entities[0] ?? null;
  const leftDockPlugins = DEFAULT_WORKSPACE_LAYOUT.leftDock.tabs.map(dockPluginById).filter(isDockPlugin);
  const rightDockPlugins = DEFAULT_WORKSPACE_LAYOUT.rightDock.tabs.map(dockPluginById).filter(isDockPlugin);
  const bottomDockPlugins = DEFAULT_WORKSPACE_LAYOUT.bottomDock.tabs.map(dockPluginById).filter(isDockPlugin);

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
    const tabs = selectedScene ? [
      { id: "scene-preview", title: `${selectedScene.label} Preview`, icon: <Play size={13} /> },
    ] : [{ id: "scene-preview", title: "Scene Preview", icon: <Play size={13} /> }];
    state.openedFilePaths.forEach((relativePath) => {
      const file = projectTree ? findProjectFile(projectTree.root, relativePath) : null;
      if (file) {
        tabs.push({ id: `file:${file.relativePath}`, title: file.name, icon: <FileCode2 size={13} /> });
      }
    });
    return tabs;
  }, [projectTree, selectedScene, state.openedFilePaths]);

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
          <button type="button">Window</button>
          <button type="button">Tools</button>
        </nav>
        <button className="button button-ghost" type="button" onClick={() => void returnToStartup()}>
          <ArrowLeft size={15} />
          Startup
        </button>
      </header>

      <section className="main-topbar">
        <div className="project-pill">
          <span className="dock-icon dock-icon-blue"><FolderOpen size={14} /></span>
          <span>
            <strong>{details?.name ?? session?.modId ?? "No mod"}</strong>
            <small>{details ? `${details.id} · ${details.version}` : session?.rootPath ?? "No active session"}</small>
          </span>
          <span className={`badge status-${details?.status ?? "warning"}`}>{details?.status ?? "session"}</span>
        </div>

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
        </div>

        <div className="main-toolbar main-toolbar-right">
          <ThemeButton onClick={() => setThemeDialogOpen(true)} />
          <button className="button button-ghost" type="button" onClick={() => setSettingsDialogOpen(true)}>
            <Settings size={15} />
            Settings
          </button>
        </div>
      </section>

      <section className="workspace-grid">
        <DockArea
          className="dock-left"
          tabs={[
            ...leftDockPlugins.map((plugin) => ({
              id: plugin.id,
              title: plugin.title,
              icon: plugin.icon,
            })),
          ]}
          activeTab={leftTab}
          onSelect={(tab) => {
            setLeftTab(tab as LeftDockTab);
            recordEvent({ type: "DockTabSelected", dock: "left", tabId: tab });
          }}
        >
          {leftTab === "project-explorer" ? (
            <ProjectExplorer
              details={details}
              projectTree={projectTree}
              loading={projectTreeTask?.status === "running"}
              selectedScene={selectedScene}
              selectedFilePath={selectedFile?.relativePath ?? null}
              onSelectScene={selectScene}
              onSelectFile={handleSelectProjectFile}
            />
          ) : null}
          {leftTab === "asset-browser" ? (
            <AssetBrowser
              details={details}
              projectTree={projectTree}
              loading={projectTreeTask?.status === "running"}
              selectedFilePath={selectedFile?.relativePath ?? null}
              onSelectFile={handleSelectProjectFile}
            />
          ) : null}
          {leftTab === "scene-hierarchy" ? (
            <SceneHierarchy
              selectedScene={selectedScene}
              hierarchy={hierarchy}
              loading={hierarchyTask?.status === "running"}
              selectedEntityId={selectedEntity?.id ?? null}
              onSelectEntity={selectSceneEntity}
            />
          ) : null}
        </DockArea>

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
                {tab.id.startsWith("file:") ? (
                  <span
                    className="workspace-tab-close"
                    role="button"
                    tabIndex={0}
                    onClick={(event) => {
                      event.stopPropagation();
                      closeWorkspaceTab(tab.id);
                    }}
                    onKeyDown={(event) => {
                      if (event.key === "Enter" || event.key === " ") {
                        event.preventDefault();
                        event.stopPropagation();
                        closeWorkspaceTab(tab.id);
                      }
                    }}
                  >
                    ×
                  </span>
                ) : null}
              </button>
            ))}
          </div>

          {selectedFile && state.activeWorkspaceTabId === `file:${selectedFile.relativePath}` ? (
            <FileWorkspace file={selectedFile} content={selectedFileContent} onReveal={() => void revealSelectedProjectFile()} />
          ) : (
          <div className="scene-workbench">
            <div className="scene-workbench-toolbar">
              <div className="scene-heading">
                <span className="dock-icon dock-icon-cyan"><Play size={14} /></span>
                <strong>{selectedScene?.label ?? "Scene Preview"}</strong>
                <span>{selectedScene?.documentPath ?? "No scene selected"}</span>
                <span className="badge badge-info">engine preview</span>
              </div>
              <div className="scene-heading-actions">
                <button className="button button-tool" type="button">Fit</button>
                <button className="button button-tool" type="button">1:1</button>
              </div>
            </div>

            <div className="main-preview-stage">
              {previewTask?.status === "running" ? (
                <div className="preview-canvas preview-loading">
                  <div className="spinner" />
                  <strong>Rendering preview...</strong>
                  <span>{Math.round((previewTask.progress ?? 0) * 100)}%</span>
                </div>
              ) : preview?.status === "ready" && preview.frameUrls.length > 0 ? (
                <EngineSlideshowPreview preview={preview} playing={state.previewPlaying} />
              ) : (
                <div className="preview-canvas preview-empty">
                  <Gauge size={38} />
                  <strong>No workspace preview yet</strong>
                  <span>Select a scene or regenerate preview.</span>
                </div>
              )}
            </div>

            <div className="workspace-scene-strip">
              {details?.scenes.map((scene) => (
                <button
                  key={scene.id}
                  type="button"
                  className={`workspace-scene-thumb ${scene.id === selectedScene?.id ? "active" : ""}`}
                  onClick={() => void selectScene(scene)}
                >
                  <span className="dock-icon dock-icon-cyan"><Play size={13} /></span>
                  <span>
                    <strong>{scene.label}</strong>
                    <small>{scene.launcherVisible ? "launcher visible" : "hidden"} · {scene.status}</small>
                  </span>
                </button>
              ))}
            </div>
          </div>
          )}
        </section>

        <DockArea
          className="dock-right"
          tabs={[
            ...rightDockPlugins.map((plugin) => ({
              id: plugin.id,
              title: plugin.title,
              icon: plugin.icon,
            })),
          ]}
          activeTab={rightTab}
          onSelect={(tab) => {
            setRightTab(tab as RightDockTab);
            recordEvent({ type: "DockTabSelected", dock: "right", tabId: tab });
          }}
        >
          {rightTab === "inspector" ? <Inspector details={details} selectedScene={selectedScene} selectedEntity={selectedEntity} selectedFile={selectedFile} /> : null}
          {rightTab === "diagnostics" ? <DiagnosticsList diagnostics={allProblems} /> : null}
          {rightTab === "properties" ? <PropertiesPanel details={details} selectedScene={selectedScene} selectedEntity={selectedEntity} selectedFile={selectedFile} /> : null}
        </DockArea>

        <DockArea
          className="dock-bottom"
          tabs={[
            ...bottomDockPlugins.map((plugin) => ({
              id: plugin.id,
              title: plugin.title,
              icon: plugin.icon,
            })),
          ]}
          activeTab={bottomTab}
          onSelect={(tab) => {
            setBottomTab(tab as BottomDockTab);
            recordEvent({ type: "DockTabSelected", dock: "bottom", tabId: tab });
          }}
        >
          {bottomTab === "problems" ? <ProblemsTable diagnostics={allProblems} /> : null}
          {bottomTab === "event-log" ? <EventTable events={eventRows} /> : null}
          {bottomTab === "tasks" ? <TaskTable tasks={Object.values(state.tasks)} /> : null}
          {bottomTab === "console" ? <p className="muted workspace-empty">Script console output will appear here.</p> : null}
          {bottomTab === "preview-cache" ? <CachePanel details={details} preview={preview} /> : null}
        </DockArea>
      </section>

      <footer className="workspace-statusbar">
        <span><span className="status-dot" />Ready</span>
        <span>{details?.name ?? "No mod"}</span>
        <span>{details?.sceneCount ?? 0} scenes</span>
        <span>{details?.contentSummary.totalFiles ?? 0} files</span>
        <span>Theme: {activeThemeId}</span>
        <span>{runningTasks.length} tasks running</span>
      </footer>

      <ThemeControllerDialog open={themeDialogOpen} onClose={() => setThemeDialogOpen(false)} />
      <SettingsDialog
        open={settingsDialogOpen}
        onClose={() => setSettingsDialogOpen(false)}
        onOpenTheme={() => {
          setSettingsDialogOpen(false);
          setThemeDialogOpen(true);
        }}
      />
    </main>
  );
}

function DockArea({
  className,
  tabs,
  activeTab,
  onSelect,
  children,
}: {
  className: string;
  tabs: Array<{ id: string; title: string; icon: React.ReactNode }>;
  activeTab: string;
  onSelect: (tabId: string) => void;
  children: React.ReactNode;
}) {
  return (
    <section className={`workspace-dock ${className}`}>
      <div className="workspace-dock-tabs">
        {tabs.map((tab) => (
          <button key={tab.id} type="button" className={`workspace-dock-tab ${activeTab === tab.id ? "active" : ""}`} onClick={() => onSelect(tab.id)}>
            {tab.icon}
            {tab.title}
          </button>
        ))}
      </div>
      <div className="workspace-dock-body">{children}</div>
    </section>
  );
}

function ProjectExplorer({
  details,
  projectTree,
  loading,
  selectedScene,
  selectedFilePath,
  onSelectScene,
  onSelectFile,
}: {
  details: EditorModDetailsDto | null;
  projectTree?: EditorProjectTreeDto;
  loading: boolean;
  selectedScene: EditorSceneSummaryDto | null;
  selectedFilePath: string | null;
  onSelectScene: (scene: EditorSceneSummaryDto) => Promise<void>;
  onSelectFile: (file: EditorProjectFileDto) => void;
}) {
  if (!details) {
    return <p className="muted workspace-empty">No mod details loaded.</p>;
  }

  return (
    <div className="dock-scroll">
      <label className="workspace-search">
        <span>Search</span>
        <input placeholder="Project files..." />
      </label>
      <SectionTitle title={`Mod Root ${projectTree ? `(${projectTree.totalFiles})` : ""}`} />
      {loading ? <p className="muted workspace-note">Indexing project files...</p> : null}
      {projectTree ? <ProjectFileTree node={projectTree.root} depth={0} maxDepth={3} selectedFilePath={selectedFilePath} onSelectFile={onSelectFile} /> : null}
      <SectionTitle title="Scenes" />
      {details.scenes.map((scene) => (
        <button key={scene.id} type="button" className={`workspace-row ${selectedScene?.id === scene.id ? "selected" : ""}`} onClick={() => void onSelectScene(scene)}>
          <span className="dock-icon dock-icon-cyan"><Play size={13} /></span>
          <span>
            <strong>{scene.label}</strong>
            <small>{scene.documentPath}</small>
          </span>
          <em className={`badge status-${scene.status}`}>{scene.status}</em>
        </button>
      ))}
    </div>
  );
}

function AssetBrowser({
  details,
  projectTree,
  loading,
  selectedFilePath,
  onSelectFile,
}: {
  details: EditorModDetailsDto | null;
  projectTree?: EditorProjectTreeDto;
  loading: boolean;
  selectedFilePath: string | null;
  onSelectFile: (file: EditorProjectFileDto) => void;
}) {
  const summary = details?.contentSummary;
  if (!summary) {
    return <p className="muted workspace-empty">No assets loaded.</p>;
  }
  const assets = projectTree ? flattenProjectFiles(projectTree.root).filter((file) => isAssetFileKind(file.kind)) : [];
  return (
    <div className="dock-scroll">
      <label className="workspace-search">
        <span>Filter</span>
        <input placeholder="Assets..." />
      </label>
      {loading ? <p className="muted workspace-note">Indexing assets...</p> : null}
      <div className="workspace-count-list">
        <CountRow label="Textures" value={summary.textures} />
        <CountRow label="Spritesheets" value={summary.spritesheets} />
        <CountRow label="Audio" value={summary.audio} />
        <CountRow label="Tilemaps" value={summary.tilemaps} />
        <CountRow label="Tilesets" value={summary.tilesets} />
        <CountRow label="Scripts" value={summary.scripts} />
        <CountRow label="Packages" value={summary.packages} />
        <CountRow label="Unknown" value={summary.unknownFiles} />
      </div>
      <SectionTitle title={`Asset Files ${assets.length ? `(${assets.length})` : ""}`} />
      {assets.length ? (
        assets.slice(0, 80).map((file) => (
          <button
            key={file.relativePath}
            type="button"
            className={`workspace-row ${selectedFilePath === file.relativePath ? "selected" : ""}`}
            onClick={() => onSelectFile(file)}
          >
            <span className="dock-icon dock-icon-cyan">{fileIcon(file)}</span>
            <span>
              <strong>{file.name}</strong>
              <small>{file.relativePath}</small>
            </span>
            <em className="badge badge-muted">{file.kind}</em>
          </button>
        ))
      ) : (
        <p className="muted workspace-note">No indexed asset files.</p>
      )}
    </div>
  );
}

function ProjectFileTree({
  node,
  depth,
  maxDepth,
  selectedFilePath,
  onSelectFile,
}: {
  node: EditorProjectFileDto;
  depth: number;
  maxDepth: number;
  selectedFilePath: string | null;
  onSelectFile: (file: EditorProjectFileDto) => void;
}) {
  if (depth > maxDepth || (depth === 0 && node.children.length === 0)) {
    return null;
  }

  const children = depth === 0 ? node.children : node.children.slice(0, 24);

  return (
    <>
      {depth > 0 ? (
        <button
          type="button"
          className={`workspace-row ${selectedFilePath === node.relativePath ? "selected" : ""}`}
          style={{ paddingLeft: 7 + depth * 12 }}
          disabled={node.isDir}
          onClick={() => onSelectFile(node)}
        >
          <span className={`dock-icon ${node.isDir ? "dock-icon-blue" : "dock-icon-cyan"}`}>{fileIcon(node)}</span>
          <span>
            <strong>{node.name}</strong>
            <small>{node.isDir ? `${node.children.length} entries` : node.relativePath}</small>
          </span>
          <em className="badge badge-muted">{node.kind}</em>
        </button>
      ) : null}
      {children.map((child) => (
        <ProjectFileTree
          key={child.relativePath || child.path}
          node={child}
          depth={depth + 1}
          maxDepth={maxDepth}
          selectedFilePath={selectedFilePath}
          onSelectFile={onSelectFile}
        />
      ))}
    </>
  );
}

function CountRow({ label, value }: { label: string; value: number }) {
  return (
    <div className="workspace-count-row">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function flattenProjectFiles(root: EditorProjectFileDto): EditorProjectFileDto[] {
  return root.children.flatMap((child) => [child, ...flattenProjectFiles(child)]).filter((file) => !file.isDir);
}

function isAssetFileKind(kind: string): boolean {
  return ["texture", "spritesheet", "audio", "tilemap", "tileset", "script", "sceneDocument", "manifest", "yaml"].includes(kind);
}

function isReadableTextFile(file: EditorProjectFileDto): boolean {
  return ["manifest", "sceneDocument", "script", "yaml"].includes(file.kind);
}

function fileIcon(file: EditorProjectFileDto): string {
  if (file.isDir) return "Dir";
  if (file.kind === "manifest") return "T";
  if (file.kind === "sceneDocument") return "Y";
  if (file.kind === "script") return "Rh";
  if (file.kind === "texture") return "Tx";
  if (file.kind === "spritesheet") return "Sp";
  if (file.kind === "audio") return "Au";
  if (file.kind === "tilemap") return "Tm";
  if (file.kind === "tileset") return "Ts";
  return "F";
}

function normalizePath(path: string): string {
  return path.replace(/\\/g, "/");
}

function findProjectFile(root: EditorProjectFileDto, relativePath: string): EditorProjectFileDto | null {
  if (root.relativePath === relativePath) {
    return root;
  }

  for (const child of root.children) {
    const match = findProjectFile(child, relativePath);
    if (match) {
      return match;
    }
  }

  return null;
}

function formatBytes(sizeBytes: number): string {
  if (sizeBytes < 1024) {
    return `${sizeBytes} B`;
  }
  if (sizeBytes < 1024 * 1024) {
    return `${(sizeBytes / 1024).toFixed(1)} KB`;
  }
  return `${(sizeBytes / (1024 * 1024)).toFixed(1)} MB`;
}

function isImageFile(file: EditorProjectFileDto): boolean {
  return file.kind === "texture" || file.kind === "spritesheet";
}

function fileDiagnosticsFor(file: EditorProjectFileDto, content?: { diagnostics: Array<{ level: "info" | "warning" | "error"; code: string; message: string; path?: string | null }> }) {
  const diagnostics = [...(content?.diagnostics ?? [])];
  if (file.kind === "unknown") {
    diagnostics.push({
      level: "warning" as const,
      code: "unknown_project_file",
      message: `File type for ${file.relativePath} is not recognized by the editor yet.`,
      path: file.relativePath,
    });
  }
  if (isReadableTextFile(file) && !content) {
    diagnostics.push({
      level: "info" as const,
      code: "text_preview_pending",
      message: `Text preview for ${file.relativePath} is not loaded yet.`,
      path: file.relativePath,
    });
  }
  return diagnostics;
}

function FileWorkspace({
  file,
  content,
  onReveal,
}: {
  file: EditorProjectFileDto;
  content?: { content: string; language: string };
  onReveal: () => void;
}) {
  return (
    <div className="file-workbench">
      <div className="scene-workbench-toolbar">
        <div className="scene-heading">
          <span className="dock-icon dock-icon-cyan">{fileIcon(file)}</span>
          <strong>{file.name}</strong>
          <span>{file.relativePath}</span>
          <span className="badge badge-info">{file.kind}</span>
        </div>
        <div className="scene-heading-actions">
          <button className="button button-tool" type="button" onClick={onReveal}>
            <FolderOpen size={14} />
            Reveal
          </button>
        </div>
      </div>

      <div className="file-preview-stage">
        {isImageFile(file) ? (
          <img className="file-image-preview" src={convertFileSrc(file.path)} alt={file.name} draggable={false} />
        ) : content ? (
          <pre className="file-code-preview" data-language={content.language}>
            <code>{content.content}</code>
          </pre>
        ) : (
          <div className="file-preview-empty">
            <FileCode2 size={40} />
            <strong>{file.kind}</strong>
            <span>{isReadableTextFile(file) ? "Loading text preview..." : file.relativePath}</span>
          </div>
        )}
      </div>

      <div className="file-metadata-strip">
        <span>{file.kind}</span>
        <span>{formatBytes(file.sizeBytes)}</span>
        <span>{file.path}</span>
      </div>
    </div>
  );
}

function SceneHierarchy({
  selectedScene,
  hierarchy,
  loading,
  selectedEntityId,
  onSelectEntity,
}: {
  selectedScene: EditorSceneSummaryDto | null;
  hierarchy?: EditorSceneHierarchyDto;
  loading: boolean;
  selectedEntityId: string | null;
  onSelectEntity: (entityId: string) => void;
}) {
  if (!selectedScene) {
    return <p className="muted workspace-empty">No scene selected.</p>;
  }

  return (
    <div className="dock-scroll">
      <SectionTitle title="Scene Context" />
      <Row icon="Sc" title={selectedScene.label} detail={selectedScene.id} badge={selectedScene.status} selected />
      <Row icon="Y" title="Document" detail={selectedScene.documentPath} badge="yaml" />
      <Row icon="Rh" title="Script" detail={selectedScene.scriptPath} badge="rhai" />
      <SectionTitle title={`Entities ${hierarchy ? `(${hierarchy.entityCount})` : ""}`} />
      {loading ? (
        <p className="muted workspace-note">Indexing scene entities...</p>
      ) : hierarchy?.entities.length ? (
        hierarchy.entities.map((entity) => (
          <button
            key={entity.id}
            type="button"
            className={`workspace-row ${entity.id === selectedEntityId ? "selected" : ""}`}
            onClick={() => onSelectEntity(entity.id)}
          >
            <span className="dock-icon dock-icon-blue">{entity.name.slice(0, 2).toUpperCase()}</span>
            <span>
              <strong>{entity.name}</strong>
              <small>
                {entity.componentCount} components
                {entity.tags.length ? ` · #${entity.tags.join(" #")}` : ""}
              </small>
            </span>
            <em className={`badge ${entity.visible ? "badge-valid" : "badge-muted"}`}>
              {entity.componentTypes[0] ?? "entity"}
            </em>
          </button>
        ))
      ) : (
        <p className="muted workspace-note">No authored entities found in this scene document.</p>
      )}
    </div>
  );
}

function Inspector({
  details,
  selectedScene,
  selectedEntity,
  selectedFile,
}: {
  details: EditorModDetailsDto | null;
  selectedScene: EditorSceneSummaryDto | null;
  selectedEntity: EditorSceneEntityDto | null;
  selectedFile: EditorProjectFileDto | null;
}) {
  return (
    <div className="dock-scroll">
      <section className="workspace-section">
        <h3>Selection</h3>
        <dl className="kv-list">
          <dt>Mod</dt>
          <dd>{details?.id ?? "none"}</dd>
          <dt>Scene</dt>
          <dd>{selectedScene?.id ?? "none"}</dd>
          <dt>Label</dt>
          <dd>{selectedScene?.label ?? "none"}</dd>
          <dt>Document</dt>
          <dd title={selectedScene?.documentPath}>{selectedScene?.documentPath ?? "none"}</dd>
          <dt>Script</dt>
          <dd title={selectedScene?.scriptPath}>{selectedScene?.scriptPath ?? "none"}</dd>
          <dt>Entity</dt>
          <dd>{selectedEntity?.id ?? "none"}</dd>
          <dt>File</dt>
          <dd title={selectedFile?.path}>{selectedFile?.relativePath ?? "none"}</dd>
        </dl>
      </section>
      {selectedFile ? (
        <section className="workspace-section">
          <h3>File</h3>
          <dl className="kv-list">
            <dt>Name</dt>
            <dd>{selectedFile.name}</dd>
            <dt>Kind</dt>
            <dd>{selectedFile.kind}</dd>
            <dt>Size</dt>
            <dd>{formatBytes(selectedFile.sizeBytes)}</dd>
            <dt>Path</dt>
            <dd title={selectedFile.path}>{selectedFile.path}</dd>
          </dl>
        </section>
      ) : null}
      {selectedEntity ? (
        <section className="workspace-section">
          <h3>Entity</h3>
          <dl className="kv-list">
            <dt>Name</dt>
            <dd>{selectedEntity.name}</dd>
            <dt>Visible</dt>
            <dd>{selectedEntity.visible ? "yes" : "no"}</dd>
            <dt>Simulation</dt>
            <dd>{selectedEntity.simulationEnabled ? "enabled" : "disabled"}</dd>
            <dt>Collision</dt>
            <dd>{selectedEntity.collisionEnabled ? "enabled" : "disabled"}</dd>
            <dt>Transforms</dt>
            <dd>
              {[
                selectedEntity.hasTransform2 ? "2D" : null,
                selectedEntity.hasTransform3 ? "3D" : null,
              ].filter(Boolean).join(", ") || "none"}
            </dd>
            <dt>Properties</dt>
            <dd>{selectedEntity.propertyCount}</dd>
          </dl>
          <div className="tag-list workspace-component-tags">
            {selectedEntity.componentTypes.length ? (
              selectedEntity.componentTypes.map((component, index) => (
                <span key={`${component}:${index}`} className="tag">{component}</span>
              ))
            ) : (
              <span className="muted">No components.</span>
            )}
          </div>
        </section>
      ) : null}
      <section className="workspace-section">
        <h3>Capabilities</h3>
        <div className="tag-list">
          {details?.capabilities.length ? details.capabilities.map((capability) => <span key={capability} className="tag">{capability}</span>) : <span className="muted">No capabilities.</span>}
        </div>
      </section>
    </div>
  );
}

function PropertiesPanel({
  details,
  selectedScene,
  selectedEntity,
  selectedFile,
}: {
  details: EditorModDetailsDto | null;
  selectedScene: EditorSceneSummaryDto | null;
  selectedEntity: EditorSceneEntityDto | null;
  selectedFile: EditorProjectFileDto | null;
}) {
  return (
    <div className="dock-scroll">
      <section className="workspace-section">
        <h3>Mod Metadata</h3>
        <dl className="kv-list">
          <dt>Name</dt>
          <dd>{details?.name ?? "none"}</dd>
          <dt>Authors</dt>
          <dd>{details?.authors.join(", ") || "none"}</dd>
          <dt>Root</dt>
          <dd title={details?.rootPath}>{details?.rootPath ?? "none"}</dd>
          <dt>Scene Visible</dt>
          <dd>{selectedScene ? (selectedScene.launcherVisible ? "yes" : "no") : "none"}</dd>
          <dt>Entity</dt>
          <dd>{selectedEntity?.name ?? "none"}</dd>
          <dt>Components</dt>
          <dd>{selectedEntity?.componentCount ?? 0}</dd>
          <dt>File</dt>
          <dd>{selectedFile?.relativePath ?? "none"}</dd>
        </dl>
      </section>
      {selectedEntity?.tags.length || selectedEntity?.groups.length ? (
        <section className="workspace-section">
          <h3>Entity Labels</h3>
          <dl className="kv-list">
            <dt>Tags</dt>
            <dd>{selectedEntity.tags.join(", ") || "none"}</dd>
            <dt>Groups</dt>
            <dd>{selectedEntity.groups.join(", ") || "none"}</dd>
          </dl>
        </section>
      ) : null}
    </div>
  );
}

function ProblemsTable({ diagnostics }: { diagnostics: Array<{ level: string; code: string; message: string; path?: string | null }> }) {
  if (diagnostics.length === 0) {
    return <p className="muted workspace-empty">No problems.</p>;
  }
  return (
    <table className="workspace-table">
      <tbody>
        {diagnostics.map((diagnostic, index) => (
          <tr key={`${diagnostic.code}:${index}`}>
            <td><span className={`badge diagnostic-${diagnostic.level}`}>{diagnostic.level}</span></td>
            <td>{diagnostic.code}</td>
            <td>{diagnostic.message}</td>
            <td>{diagnostic.path ?? ""}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function EventTable({ events }: { events: Array<{ type: string }> }) {
  return (
    <table className="workspace-table">
      <tbody>
        {events.map((event, index) => (
          <tr key={`${event.type}:${index}`}>
            <td><code>{event.type}</code></td>
            <td>{index === 0 ? "latest" : "event"}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function TaskTable({ tasks }: { tasks: Array<{ id: string; label: string; status: string; startedAt: number; progress?: number }> }) {
  return (
    <table className="workspace-table">
      <tbody>
        {tasks.slice(0, 12).map((task) => (
          <tr key={task.id}>
            <td><span className={`badge ${task.status === "failed" ? "badge-error" : task.status === "running" ? "badge-info" : "badge-valid"}`}>{task.status}</span></td>
            <td><code>{task.id}</code></td>
            <td>{task.label}</td>
            <td>{task.progress != null ? `${Math.round(task.progress * 100)}%` : formatTaskTime(task.startedAt)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function CachePanel({ details, preview }: { details: EditorModDetailsDto | null; preview?: ScenePreviewDto }) {
  return (
    <div className="dock-scroll">
      <section className="workspace-section">
        <h3>Preview Cache</h3>
        <dl className="kv-list">
          <dt>Project</dt>
          <dd>{details?.projectCacheId ?? "none"}</dd>
          <dt>Status</dt>
          <dd>{preview?.status ?? "missing"}</dd>
          <dt>Frames</dt>
          <dd>{preview?.frameCount ?? 0}</dd>
          <dt>Hash</dt>
          <dd>{preview?.sourceHash ?? "none"}</dd>
        </dl>
      </section>
    </div>
  );
}

function SectionTitle({ title }: { title: string }) {
  return <div className="workspace-section-title">{title}</div>;
}

function Row({ icon, title, detail, badge, selected }: { icon: string; title: string; detail: string; badge?: string; selected?: boolean }) {
  return (
    <div className={`workspace-row ${selected ? "selected" : ""}`}>
      <span className="dock-icon dock-icon-blue">{icon}</span>
      <span>
        <strong>{title}</strong>
        <small>{detail}</small>
      </span>
      {badge ? <em className="badge badge-muted">{badge}</em> : null}
    </div>
  );
}
