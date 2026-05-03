import { useMemo, useState } from "react";
import type React from "react";
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
import type { EditorModDetailsDto, EditorSceneHierarchyDto, EditorSceneSummaryDto, ScenePreviewDto } from "../api/dto";
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
    returnToStartup,
    regeneratePreview,
    recordEvent,
    selectScene,
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
  const preview = activePreview(details, selectedScene?.id ?? null, state.previews);
  const previewTask = details && selectedScene ? state.tasks[`preview:${details.id}:${selectedScene.id}`] : undefined;
  const runningTasks = Object.values(state.tasks).filter((task) => task.status === "running");
  const eventRows = state.events.slice(0, 8);
  const sceneDiagnostics = selectedScene?.diagnostics ?? [];
  const modDiagnostics = details?.diagnostics ?? [];
  const problems = [...modDiagnostics, ...sceneDiagnostics];
  const hierarchy = details && selectedScene ? state.sceneHierarchies[previewKey(details.id, selectedScene.id)] : undefined;
  const hierarchyTask = details && selectedScene ? state.tasks[`scene-hierarchy:${details.id}:${selectedScene.id}`] : undefined;
  const leftDockPlugins = DEFAULT_WORKSPACE_LAYOUT.leftDock.tabs.map(dockPluginById).filter(isDockPlugin);
  const rightDockPlugins = DEFAULT_WORKSPACE_LAYOUT.rightDock.tabs.map(dockPluginById).filter(isDockPlugin);
  const bottomDockPlugins = DEFAULT_WORKSPACE_LAYOUT.bottomDock.tabs.map(dockPluginById).filter(isDockPlugin);

  const workspaceTabs = useMemo(() => {
    const tabs = selectedScene ? [
      { id: "scene-preview", title: `${selectedScene.label} Preview`, icon: <Play size={13} /> },
      { id: "scene-document", title: selectedScene.documentPath.split(/[\\/]/).pop() ?? "scene.yml", icon: <FileCode2 size={13} /> },
      { id: "scene-script", title: selectedScene.scriptPath.split(/[\\/]/).pop() ?? "scene.rhai", icon: <Terminal size={13} /> },
    ] : [{ id: "scene-preview", title: "Scene Preview", icon: <Play size={13} /> }];
    return tabs;
  }, [selectedScene]);

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
          {leftTab === "project-explorer" ? <ProjectExplorer details={details} selectedScene={selectedScene} onSelectScene={selectScene} /> : null}
          {leftTab === "asset-browser" ? <AssetBrowser details={details} /> : null}
          {leftTab === "scene-hierarchy" ? <SceneHierarchy selectedScene={selectedScene} hierarchy={hierarchy} loading={hierarchyTask?.status === "running"} /> : null}
        </DockArea>

        <section className="workspace-center">
          <div className="workspace-tabs">
            {workspaceTabs.map((tab, index) => (
              <button
                key={tab.id}
                type="button"
                className={`workspace-tab ${index === 0 ? "active" : ""}`}
                onClick={() => recordEvent({ type: "WorkspaceTabSelected", tabId: tab.id })}
              >
                {tab.icon}
                {tab.title}
              </button>
            ))}
          </div>

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
          {rightTab === "inspector" ? <Inspector details={details} selectedScene={selectedScene} /> : null}
          {rightTab === "diagnostics" ? <DiagnosticsList diagnostics={problems} /> : null}
          {rightTab === "properties" ? <PropertiesPanel details={details} selectedScene={selectedScene} /> : null}
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
          {bottomTab === "problems" ? <ProblemsTable diagnostics={problems} /> : null}
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
  selectedScene,
  onSelectScene,
}: {
  details: EditorModDetailsDto | null;
  selectedScene: EditorSceneSummaryDto | null;
  onSelectScene: (scene: EditorSceneSummaryDto) => Promise<void>;
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
      <SectionTitle title="Mod Root" />
      <Row icon="T" title="mod.toml" detail={`${details.status} · ${details.version}`} badge="manifest" selected />
      <Row icon="S" title="scenes" detail={`${details.sceneCount} scenes`} badge={`${details.visibleSceneCount} visible`} />
      <Row icon="A" title="assets" detail={`${details.contentSummary.totalFiles} files`} badge="summary" />
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

function AssetBrowser({ details }: { details: EditorModDetailsDto | null }) {
  const summary = details?.contentSummary;
  if (!summary) {
    return <p className="muted workspace-empty">No assets loaded.</p>;
  }
  const rows = [
    ["Textures", summary.textures],
    ["Spritesheets", summary.spritesheets],
    ["Audio", summary.audio],
    ["Tilemaps", summary.tilemaps],
    ["Tilesets", summary.tilesets],
    ["Scripts", summary.scripts],
    ["Packages", summary.packages],
    ["Unknown", summary.unknownFiles],
  ];
  return (
    <div className="dock-scroll">
      <label className="workspace-search">
        <span>Filter</span>
        <input placeholder="Assets..." />
      </label>
      <div className="workspace-count-list">
        {rows.map(([label, value]) => (
          <div key={label} className="workspace-count-row">
            <span>{label}</span>
            <strong>{value}</strong>
          </div>
        ))}
      </div>
    </div>
  );
}

function SceneHierarchy({
  selectedScene,
  hierarchy,
  loading,
}: {
  selectedScene: EditorSceneSummaryDto | null;
  hierarchy?: EditorSceneHierarchyDto;
  loading: boolean;
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
          <div key={entity.id} className="workspace-row">
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
          </div>
        ))
      ) : (
        <p className="muted workspace-note">No authored entities found in this scene document.</p>
      )}
    </div>
  );
}

function Inspector({ details, selectedScene }: { details: EditorModDetailsDto | null; selectedScene: EditorSceneSummaryDto | null }) {
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
        </dl>
      </section>
      <section className="workspace-section">
        <h3>Capabilities</h3>
        <div className="tag-list">
          {details?.capabilities.length ? details.capabilities.map((capability) => <span key={capability} className="tag">{capability}</span>) : <span className="muted">No capabilities.</span>}
        </div>
      </section>
    </div>
  );
}

function PropertiesPanel({ details, selectedScene }: { details: EditorModDetailsDto | null; selectedScene: EditorSceneSummaryDto | null }) {
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
        </dl>
      </section>
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
