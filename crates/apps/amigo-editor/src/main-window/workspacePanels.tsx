import { useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import {
  AlertTriangle,
  Box,
  Boxes,
  Code2,
  FileCode2,
  FileCog,
  Folder,
  FolderOpen,
  Gauge,
  Image,
  Info,
  Link,
  Map,
  Package,
  Play,
  Plug,
  Search,
  Type,
  Volume2,
} from "lucide-react";
import type { WindowBusEvent } from "../app/windowBusTypes";
import type {
  EditorDiagnosticDto,
  EditorModDetailsDto,
  EditorProjectFileDto,
  EditorProjectFileContentDto,
  EditorProjectStructureNodeDto,
  EditorProjectStructureTreeDto,
  EditorProjectTreeDto,
  EditorSceneEntityDto,
  EditorSceneHierarchyDto,
  EditorSceneSummaryDto,
  ScenePreviewDto,
} from "../api/dto";
import { ComponentHost } from "../editor-components/componentHost";
import { editorComponentById, iconForEditorComponent } from "../editor-components/componentRegistry";
import type { EditorComponentContext, EditorComponentInstance } from "../editor-components/componentTypes";
import { DiagnosticsList } from "../startup/DiagnosticsList";
import { EngineSlideshowPreview } from "../startup/EngineSlideshowPreview";
import {
  canReadProjectFileContent,
  resolveFileWorkspaceDescriptor,
  workspaceDescriptorLanguage,
  type FileWorkspaceDescriptor,
} from "./workspaceResources";

export type WorkspaceComponentServices = {
  allProblems?: EditorDiagnosticDto[];
  details?: EditorModDetailsDto | null;
  eventFilter?: string;
  eventRows?: Array<{ type: string }>;
  eventSearch?: string;
  eventSessionFilter?: string;
  eventSourceFilter?: string;
  handleSelectProjectFile?: (file: EditorProjectFileDto) => void;
  hierarchy?: EditorSceneHierarchyDto;
  hierarchyTask?: { status: string } | undefined;
  onRevealSelectedFile?: () => void;
  preview?: ScenePreviewDto;
  previewPlaying?: boolean;
  previewTask?: { progress?: number; status: string } | undefined;
  projectTree?: EditorProjectTreeDto;
  projectStructureTree?: EditorProjectStructureTreeDto;
  projectTreeTask?: { status: string } | undefined;
  selectedEntity?: EditorSceneEntityDto | null;
  selectedFile?: EditorProjectFileDto | null;
  selectedFileContent?: EditorProjectFileContentDto | null;
  selectedScene?: EditorSceneSummaryDto | null;
  onCreateExpectedFolder?: (expectedPath: string) => Promise<void>;
  onProjectNodeActivated?: (node: EditorProjectStructureNodeDto | ProjectTreeNode) => void;
  selectScene?: (scene: EditorSceneSummaryDto) => Promise<void>;
  selectSceneEntity?: (entityId: string) => void;
  setEventFilter?: (filter: string) => void;
  setEventSearch?: (value: string) => void;
  setEventSessionFilter?: (filter: string) => void;
  setEventSourceFilter?: (filter: string) => void;
  tasks?: Array<{ id: string; label: string; status: string; startedAt: number; progress?: number }>;
  windowEventRows?: WindowBusEvent[];
};

export function componentTabs(instances: EditorComponentInstance[]) {
  return instances.map((instance) => {
    const definition = editorComponentById(instance.componentId);
    return {
      id: instance.instanceId,
      title: instance.titleOverride ?? definition?.title ?? instance.componentId,
      icon: definition ? iconForEditorComponent(definition.icon, 13) : <Box size={13} />,
    };
  });
}

export function renderWorkspaceComponent(
  instance: EditorComponentInstance,
  context: EditorComponentContext,
  services: WorkspaceComponentServices,
) {
  switch (instance.componentId) {
    case "scene.preview":
      return (
        <ScenePreviewWorkbench
          details={services.details ?? null}
          playing={services.previewPlaying ?? true}
          preview={services.preview}
          previewTask={services.previewTask}
          selectedScene={services.selectedScene ?? null}
          onSelectScene={(scene) => void services.selectScene?.(scene)}
        />
      );
    case "file.manifest":
    case "file.scene":
    case "file.scene-script":
    case "file.package":
    case "file.script":
    case "file.texture":
    case "file.sprite":
    case "file.atlas":
    case "file.tileset":
    case "file.tilemap":
    case "file.audio":
    case "file.font":
    case "file.particle":
    case "file.ui":
    case "file.input":
    case "file.config":
    case "file.text":
    case "file.binary":
      return (
        <ResolvedFileWorkspace
          file={services.selectedFile ?? null}
          content={services.selectedFileContent ?? undefined}
          onReveal={services.onRevealSelectedFile}
        />
      );
    case "project.overview":
      return <ProjectOverview details={services.details ?? null} />;
    case "project.capabilities":
      return <ProjectCapabilities details={services.details ?? null} />;
    case "project.dependencies":
      return <ProjectDependencies details={services.details ?? null} />;
    case "project.explorer":
      return (
        <ProjectExplorer
          details={services.details ?? null}
          loading={services.projectTreeTask?.status === "running"}
          onSelectFile={(file) => services.handleSelectProjectFile?.(file)}
          onSelectScene={(scene) => services.selectScene?.(scene) ?? Promise.resolve()}
          onCreateExpectedFolder={services.onCreateExpectedFolder}
          onProjectNodeActivated={services.onProjectNodeActivated}
          projectTree={services.projectTree}
          projectStructureTree={services.projectStructureTree}
          selectedFilePath={services.selectedFile?.relativePath ?? null}
          selectedScene={services.selectedScene ?? null}
        />
      );
    case "scenes.browser":
      return (
        <ScenesBrowser
          details={services.details ?? null}
          onSelectFile={(file) => services.handleSelectProjectFile?.(file)}
          onSelectScene={(scene) => services.selectScene?.(scene) ?? Promise.resolve()}
          projectTree={services.projectTree}
          selectedScene={services.selectedScene ?? null}
        />
      );
    case "assets.browser":
      return (
        <AssetBrowser
          details={services.details ?? null}
          loading={services.projectTreeTask?.status === "running"}
          onSelectFile={(file) => services.handleSelectProjectFile?.(file)}
          projectTree={services.projectTree}
          selectedFilePath={services.selectedFile?.relativePath ?? null}
        />
      );
    case "scripts.browser":
      return (
        <ScriptsBrowser
          details={services.details ?? null}
          onSelectFile={(file) => services.handleSelectProjectFile?.(file)}
          projectTree={services.projectTree}
        />
      );
    case "scene.hierarchy":
      return (
        <SceneHierarchy
          hierarchy={services.hierarchy}
          loading={services.hierarchyTask?.status === "running"}
          onSelectEntity={(entityId) => services.selectSceneEntity?.(entityId)}
          selectedEntityId={services.selectedEntity?.id ?? null}
          selectedScene={services.selectedScene ?? null}
        />
      );
    case "entity.inspector":
      return (
        <Inspector
          details={services.details ?? null}
          selectedEntity={services.selectedEntity ?? null}
          selectedFile={services.selectedFile ?? null}
          selectedScene={services.selectedScene ?? null}
        />
      );
    case "entity.properties":
      return (
        <PropertiesPanel
          details={services.details ?? null}
          selectedEntity={services.selectedEntity ?? null}
          selectedFile={services.selectedFile ?? null}
          selectedScene={services.selectedScene ?? null}
        />
      );
    case "diagnostics.panel":
      return <DiagnosticsList diagnostics={services.allProblems ?? []} />;
    case "diagnostics.problems":
      return <ProblemsTable diagnostics={services.allProblems ?? []} />;
    case "events.log":
      return (
        <EventTable
          events={services.eventRows ?? []}
          filter={services.eventFilter ?? "all"}
          onFilterChange={services.setEventFilter ?? (() => undefined)}
          onSearchChange={services.setEventSearch ?? (() => undefined)}
          onSessionFilterChange={services.setEventSessionFilter ?? (() => undefined)}
          onSourceFilterChange={services.setEventSourceFilter ?? (() => undefined)}
          search={services.eventSearch ?? ""}
          sessionFilter={services.eventSessionFilter ?? "all"}
          sourceFilter={services.eventSourceFilter ?? "all"}
          windowEvents={services.windowEventRows ?? []}
        />
      );
    case "tasks.monitor":
      return <TaskTable tasks={services.tasks ?? []} />;
    case "scripting.console":
      return <p className="muted workspace-empty">Script console output will appear here.</p>;
    case "cache.preview":
      return <CachePanel details={services.details ?? null} preview={services.preview} />;
    default:
      return <ComponentHost context={context} instance={instance} />;
  }
}

function ScenePreviewWorkbench({
  details,
  onSelectScene,
  playing,
  preview,
  previewTask,
  selectedScene,
}: {
  details: EditorModDetailsDto | null;
  onSelectScene: (scene: EditorSceneSummaryDto) => void;
  playing: boolean;
  preview?: ScenePreviewDto;
  previewTask?: { progress?: number; status: string };
  selectedScene: EditorSceneSummaryDto | null;
}) {
  return (
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
          <EngineSlideshowPreview preview={preview} playing={playing} />
        ) : (
          <div className="preview-canvas preview-empty">
            <Gauge size={38} />
            <strong>No workspace preview yet</strong>
            <span>Select a scene or regenerate preview.</span>
          </div>
        )}
      </div>
    </div>
  );
}

function ProjectOverview({ details }: { details: EditorModDetailsDto | null }) {
  if (!details) {
    return <p className="muted workspace-empty">No mod loaded.</p>;
  }
  return (
    <div className="project-center-panel">
      <header>
        <span className="dock-icon dock-icon-blue"><Boxes size={14} /></span>
        <div>
          <h2>{details.name}</h2>
          <p>{details.id} · {details.version}</p>
        </div>
        <span className={`badge status-${details.status}`}>{details.status}</span>
      </header>
      <section className="workspace-section">
        <h3>Summary</h3>
        <dl className="kv-list">
          <dt>Root</dt>
          <dd title={details.rootPath}>{details.rootPath}</dd>
          <dt>Authors</dt>
          <dd>{details.authors.join(", ") || "none"}</dd>
          <dt>Scenes</dt>
          <dd>{details.sceneCount} total · {details.visibleSceneCount} launcher visible</dd>
          <dt>Files</dt>
          <dd>{details.contentSummary.totalFiles}</dd>
          <dt>Diagnostics</dt>
          <dd>{details.diagnostics.length}</dd>
        </dl>
      </section>
      <section className="workspace-section">
        <h3>Description</h3>
        <p className="muted">{details.description || "No description."}</p>
      </section>
      <ContentBreakdown details={details} />
    </div>
  );
}

function ProjectCapabilities({ details }: { details: EditorModDetailsDto | null }) {
  return (
    <div className="project-center-panel">
      <header>
        <span className="dock-icon dock-icon-blue"><Plug size={14} /></span>
        <div>
          <h2>Capabilities</h2>
          <p>{details?.id ?? "No mod"}</p>
        </div>
      </header>
      <section className="workspace-section">
        <h3>Declared Capabilities</h3>
        <div className="project-token-grid">
          {details?.capabilities.length ? details.capabilities.map((capability) => (
            <span key={capability} className="tag">{capability}</span>
          )) : <p className="muted">No capabilities declared.</p>}
        </div>
      </section>
    </div>
  );
}

function ProjectDependencies({ details }: { details: EditorModDetailsDto | null }) {
  return (
    <div className="project-center-panel">
      <header>
        <span className="dock-icon dock-icon-blue"><Package size={14} /></span>
        <div>
          <h2>Dependencies</h2>
          <p>{details?.id ?? "No mod"}</p>
        </div>
      </header>
      <section className="workspace-section">
        <h3>Dependencies</h3>
        {details?.dependencies.length ? details.dependencies.map((dependency) => {
          const missing = details.missingDependencies.includes(dependency);
          return (
            <div key={dependency} className="workspace-row">
              <span className="dock-icon dock-icon-blue"><Package size={13} /></span>
              <span>
                <strong>{dependency}</strong>
                <small>{missing ? "missing from mods root" : "available"}</small>
              </span>
              <em className={`badge ${missing ? "badge-error" : "badge-valid"}`}>{missing ? "missing" : "ok"}</em>
            </div>
          );
        }) : <p className="muted">No dependencies.</p>}
      </section>
    </div>
  );
}

function ContentBreakdown({ details }: { details: EditorModDetailsDto }) {
  const rows = [
    ["Scene YAML", details.contentSummary.sceneYaml],
    ["Scripts", details.contentSummary.scripts],
    ["Textures", details.contentSummary.textures],
    ["Spritesheets", details.contentSummary.spritesheets],
    ["Audio", details.contentSummary.audio],
    ["Fonts", details.contentSummary.fonts],
    ["Tilemaps", details.contentSummary.tilemaps],
    ["Tilesets", details.contentSummary.tilesets],
    ["Packages", details.contentSummary.packages],
    ["Unknown", details.contentSummary.unknownFiles],
  ] as const;
  return (
    <section className="workspace-section">
      <h3>Content Breakdown</h3>
      <div className="workspace-count-list two-column">
        {rows.map(([label, value]) => <CountRow key={label} label={label} value={value} />)}
      </div>
    </section>
  );
}

function ProjectExplorer({
  details,
  projectTree,
  projectStructureTree,
  loading,
  onCreateExpectedFolder,
  onProjectNodeActivated,
  selectedScene,
  selectedFilePath,
  onSelectScene,
  onSelectFile,
}: {
  details: EditorModDetailsDto | null;
  projectTree?: EditorProjectTreeDto;
  projectStructureTree?: EditorProjectStructureTreeDto;
  loading: boolean;
  onCreateExpectedFolder?: (expectedPath: string) => Promise<void>;
  onProjectNodeActivated?: (node: EngineProjectTreeNode) => void;
  selectedScene: EditorSceneSummaryDto | null;
  selectedFilePath: string | null;
  onSelectScene: (scene: EditorSceneSummaryDto) => Promise<void>;
  onSelectFile: (file: EditorProjectFileDto) => void;
}) {
  if (!details) {
    return <p className="muted workspace-empty">No mod details loaded.</p>;
  }

  const tree = projectStructureTree?.root ?? buildEngineProjectTree(details, projectTree);
  const [search, setSearch] = useState("");
  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});
  const [selectedProjectNode, setSelectedProjectNode] = useState<EngineProjectTreeNode | null>(tree);
  const [contextMenu, setContextMenu] = useState<{
    node: EngineProjectTreeNode;
    x: number;
    y: number;
  } | null>(null);

  function activateProjectNode(node: EngineProjectTreeNode) {
    setSelectedProjectNode(node);
    onProjectNodeActivated?.(node);
  }

  return (
    <div className="dock-scroll project-explorer-panel">
      <label className="project-tree-searchbar">
        <Search size={13} />
        <input
          placeholder="Search project..."
          value={search}
          onChange={(event) => setSearch(event.target.value)}
        />
      </label>
      <div className="project-tree-separator" aria-hidden="true" />
      {selectedProjectNode ? (
        <ProjectNodeActionStrip
          node={selectedProjectNode}
          onCreateExpectedFolder={onCreateExpectedFolder}
          onOpenNode={activateProjectNode}
          onSelectFile={onSelectFile}
          onSelectScene={onSelectScene}
        />
      ) : null}
      {loading ? <p className="muted workspace-note">Indexing project files...</p> : null}
      <EngineProjectTree
        node={tree}
        depth={0}
        selectedFilePath={selectedFilePath}
        selectedSceneId={selectedScene?.id ?? null}
        onSelectFile={onSelectFile}
        onSelectScene={onSelectScene}
        onCreateExpectedFolder={onCreateExpectedFolder}
        onProjectNodeActivated={activateProjectNode}
        onOpenContextMenu={(node, x, y) => setContextMenu({ node, x, y })}
        search={search}
        collapsed={collapsed}
        selectedNodeId={selectedProjectNode?.id ?? null}
        onToggle={(nodeId) => setCollapsed((current) => ({ ...current, [nodeId]: !current[nodeId] }))}
      />
      {contextMenu ? (
        <ProjectNodeContextMenu
          menu={contextMenu}
          onClose={() => setContextMenu(null)}
          onCreateExpectedFolder={onCreateExpectedFolder}
          onProjectNodeActivated={onProjectNodeActivated}
        />
      ) : null}
    </div>
  );
}

type ProjectTreeNodeStatus = "ok" | "valid" | "ready" | "warn" | "error" | "empty" | "missing" | "cached";

type ProjectTreeNodeKind =
  | "modRoot"
  | "overview"
  | "manifest"
  | "folder"
  | "expectedFolder"
  | "scene"
  | "sceneDocument"
  | "sceneScript"
  | "assetCategory"
  | "scriptFile"
  | "scriptPackage"
  | "virtualGroup"
  | "capabilities"
  | "dependencies"
  | "diagnostics";

type ProjectTreeNode = {
  id: string;
  label: string;
  kind: ProjectTreeNodeKind;
  icon: string;
  status?: ProjectTreeNodeStatus;
  count?: number;
  path?: string;
  expectedPath?: string;
  exists: boolean;
  empty?: boolean;
  ghost?: boolean;
  file?: EditorProjectFileDto;
  scene?: EditorSceneSummaryDto;
  children?: ProjectTreeNode[];
};

type EngineProjectTreeNode = ProjectTreeNode | EditorProjectStructureNodeDto;

function buildEngineProjectTree(details: EditorModDetailsDto, projectTree?: EditorProjectTreeDto): ProjectTreeNode {
  const root = projectTree?.root;
  const manifest = root ? findProjectFile(root, "mod.toml") : null;
  const allFiles = root ? flattenProjectFiles(root) : [];
  const scriptFiles = allFiles.filter((file) => file.kind === "script" && !isSceneOwnedScript(details, file.relativePath));
  const packageFiles = allFiles.filter((file) => file.relativePath.startsWith("packages/"));
  const diagnosticsCount = details.diagnostics.length + details.scenes.reduce((count, scene) => count + scene.diagnostics.length, 0);

  return {
    id: `mod:${details.id}`,
    label: details.id,
    kind: "modRoot",
    icon: "Mod",
    status: statusForEditorStatus(details.status),
    count: details.contentSummary.totalFiles,
    path: projectTree?.rootPath ?? details.rootPath,
    exists: true,
    children: [
      {
        id: "overview",
        label: "Overview",
        kind: "overview",
        icon: "Info",
        status: statusForEditorStatus(details.status),
        exists: true,
      },
      {
        id: "manifest:mod.toml",
        label: "mod.toml",
        kind: "manifest",
        icon: "Toml",
        status: manifest ? statusForEditorStatus(details.status) : "error",
        path: manifest?.relativePath ?? "mod.toml",
        expectedPath: "mod.toml",
        exists: Boolean(manifest),
        ghost: !manifest,
        file: manifest ?? undefined,
      },
      {
        id: "group:scenes",
        label: "scenes",
        kind: rootChildExists(root, "scenes") ? "folder" : "expectedFolder",
        icon: "Sc",
        count: details.scenes.length,
        status: details.scenes.length ? "ready" : "missing",
        expectedPath: "scenes/",
        exists: rootChildExists(root, "scenes"),
        ghost: !rootChildExists(root, "scenes"),
        empty: details.scenes.length === 0,
        children: details.scenes.map((scene) => sceneProjectNode(scene, root)),
      },
      {
        id: "group:assets",
        label: "assets",
        kind: rootChildExists(root, "assets") ? "folder" : "expectedFolder",
        icon: "Assets",
        count: assetTotal(details),
        status: assetTotal(details) ? "ok" : "empty",
        expectedPath: "assets/",
        exists: rootChildExists(root, "assets"),
        ghost: !rootChildExists(root, "assets"),
        empty: assetTotal(details) === 0,
        children: assetCategoryNodes(details, root),
      },
      {
        id: "group:scripts",
        label: "scripts",
        kind: rootChildExists(root, "scripts") ? "folder" : "expectedFolder",
        icon: "Rh",
        count: scriptFiles.length,
        status: rootChildExists(root, "scripts") ? (scriptFiles.length ? "ok" : "empty") : "missing",
        expectedPath: "scripts/",
        exists: rootChildExists(root, "scripts"),
        ghost: !rootChildExists(root, "scripts"),
        empty: rootChildExists(root, "scripts") && scriptFiles.length === 0,
        children: scriptFiles.slice(0, 24).map((file) => fileProjectNode(file, "scriptFile")),
      },
      {
        id: "group:packages",
        label: "packages",
        kind: rootChildExists(root, "packages") ? "folder" : "expectedFolder",
        icon: "Pkg",
        count: details.contentSummary.packages,
        status: rootChildExists(root, "packages") ? (details.contentSummary.packages ? "ok" : "empty") : "missing",
        expectedPath: "packages/",
        exists: rootChildExists(root, "packages"),
        ghost: !rootChildExists(root, "packages"),
        empty: rootChildExists(root, "packages") && packageFiles.length === 0,
        children: packageFiles.slice(0, 24).map((file) => fileProjectNode(file, "scriptPackage")),
      },
      {
        id: "virtual:capabilities",
        label: "Capabilities",
        kind: "capabilities",
        icon: "Plug",
        count: details.capabilities.length,
        status: details.capabilities.length ? "ok" : "empty",
        exists: true,
      },
      {
        id: "virtual:dependencies",
        label: "Dependencies",
        kind: "dependencies",
        icon: "Link",
        count: details.dependencies.length,
        status: details.missingDependencies.length ? "warn" : "ok",
        exists: true,
      },
      {
        id: "virtual:diagnostics",
        label: "Diagnostics",
        kind: "diagnostics",
        icon: "Diag",
        count: diagnosticsCount,
        status: diagnosticsCount ? "warn" : "ok",
        exists: true,
      },
    ],
  };
}

function EngineProjectTree({
  collapsed,
  depth,
  node,
  onCreateExpectedFolder,
  onOpenContextMenu,
  onProjectNodeActivated,
  onSelectFile,
  onSelectScene,
  onToggle,
  search,
  selectedFilePath,
  selectedNodeId,
  selectedSceneId,
}: {
  collapsed: Record<string, boolean>;
  depth: number;
  node: EngineProjectTreeNode;
  onCreateExpectedFolder?: (expectedPath: string) => Promise<void>;
  onOpenContextMenu: (node: EngineProjectTreeNode, x: number, y: number) => void;
  onProjectNodeActivated?: (node: EngineProjectTreeNode) => void;
  onSelectFile: (file: EditorProjectFileDto) => void;
  onSelectScene: (scene: EditorSceneSummaryDto) => Promise<void>;
  onToggle: (nodeId: string) => void;
  search: string;
  selectedFilePath: string | null;
  selectedNodeId: string | null;
  selectedSceneId: string | null;
}) {
  const selected = node.id === selectedNodeId || node.file?.relativePath === selectedFilePath || node.scene?.id === selectedSceneId;
  const expandable = Boolean(node.children?.length);
  const isCollapsed = collapsed[node.id] ?? false;
  const normalizedSearch = search.trim().toLowerCase();
  if (normalizedSearch && !projectNodeMatchesSearch(node, normalizedSearch)) {
    return null;
  }

  function activate() {
    onProjectNodeActivated?.(node);
    if (node.scene) {
      void onSelectScene(node.scene);
      return;
    }
    if (node.file) {
      onSelectFile(node.file);
    }
  }

  return (
    <div className="engine-tree-branch">
      <button
        type="button"
        className={`engine-tree-node ${projectNodeTypographyClass(node)} ${selected ? "selected" : ""} ${node.ghost ? "ghost" : ""} ${node.empty ? "empty" : ""} status-${node.status ?? "ok"}`}
        style={{ paddingLeft: 8 + depth * 14 }}
        onClick={activate}
        onContextMenu={(event) => {
          event.preventDefault();
          onOpenContextMenu(node, event.clientX, event.clientY);
        }}
        title={node.path ?? node.expectedPath ?? node.label}
      >
        <span
          className="tree-twist"
          onClick={(event) => {
            if (!expandable) return;
            event.stopPropagation();
            onToggle(node.id);
          }}
        >
          {expandable ? (isCollapsed ? "▸" : "▾") : ""}
        </span>
        <span className="dock-icon tree-icon">{projectNodeIcon(node)}</span>
        <span className="tree-label">
          <strong>{node.label}</strong>
          {node.expectedPath && !node.exists ? <small>{node.expectedPath}</small> : null}
        </span>
        {node.count != null ? <span className="tree-count">{node.count}</span> : null}
      </button>
      {!isCollapsed && node.children?.map((child) => (
        <EngineProjectTree
          key={child.id}
          collapsed={collapsed}
          depth={depth + 1}
          node={child}
          onCreateExpectedFolder={onCreateExpectedFolder}
          onOpenContextMenu={onOpenContextMenu}
          onProjectNodeActivated={onProjectNodeActivated}
          selectedFilePath={selectedFilePath}
          selectedSceneId={selectedSceneId}
          onSelectFile={onSelectFile}
          onSelectScene={onSelectScene}
          onToggle={onToggle}
          search={search}
          selectedNodeId={selectedNodeId}
        />
      ))}
    </div>
  );
}

function projectNodeTypographyClass(node: EngineProjectTreeNode): string {
  if (["manifest", "sceneDocument", "sceneScript", "scriptFile", "scriptPackage"].includes(node.kind)) {
    return "file-node";
  }
  return "heading-node";
}

function ProjectNodeActionStrip({
  node,
  onCreateExpectedFolder,
  onOpenNode,
  onSelectFile,
  onSelectScene,
}: {
  node: EngineProjectTreeNode;
  onCreateExpectedFolder?: (expectedPath: string) => Promise<void>;
  onOpenNode: (node: EngineProjectTreeNode) => void;
  onSelectFile: (file: EditorProjectFileDto) => void;
  onSelectScene: (scene: EditorSceneSummaryDto) => Promise<void>;
}) {
  const detail = node.path ?? node.expectedPath ?? projectNodeKindLabel(node.kind);
  const canCreate = Boolean(node.ghost && node.expectedPath && onCreateExpectedFolder);
  const canCopy = Boolean(node.path || node.expectedPath);
  const canOpen = Boolean(node.file || node.scene || ["overview", "capabilities", "dependencies", "diagnostics"].includes(node.kind));

  return (
    <div className="project-node-action-strip">
      <span className="project-node-action-summary">
        <strong>{node.label}</strong>
        <small>{detail}</small>
      </span>
      <span className="project-node-action-buttons">
        {canOpen ? (
          <button
            type="button"
            onClick={() => {
              onOpenNode(node);
              if (node.scene) {
                void onSelectScene(node.scene);
              } else if (node.file) {
                onSelectFile(node.file);
              }
            }}
          >
            Open
          </button>
        ) : null}
        {canCreate ? (
          <button type="button" onClick={() => void onCreateExpectedFolder?.(node.expectedPath!)}>
            Create
          </button>
        ) : null}
        {canCopy ? (
          <button type="button" onClick={() => void navigator.clipboard.writeText(node.path ?? node.expectedPath ?? "")}>
            Copy
          </button>
        ) : null}
      </span>
    </div>
  );
}

function projectNodeKindLabel(kind: string): string {
  return kind.replace(/([A-Z])/g, " $1").toLowerCase();
}

function projectNodeMatchesSearch(node: EngineProjectTreeNode, search: string): boolean {
  const ownText = `${node.label} ${node.kind} ${node.status ?? ""} ${node.path ?? ""} ${node.expectedPath ?? ""}`.toLowerCase();
  return ownText.includes(search) || (node.children ?? []).some((child) => projectNodeMatchesSearch(child, search));
}

function ProjectNodeContextMenu({
  menu,
  onClose,
  onCreateExpectedFolder,
  onProjectNodeActivated,
}: {
  menu: { node: EngineProjectTreeNode; x: number; y: number };
  onClose: () => void;
  onCreateExpectedFolder?: (expectedPath: string) => Promise<void>;
  onProjectNodeActivated?: (node: EngineProjectTreeNode) => void;
}) {
  const node = menu.node;
  const actions: Array<{ id: string; label: string; run: () => void }> = [];
  if (node.ghost && node.expectedPath) {
    actions.push({
      id: "create-folder",
      label: "Create Folder",
      run: () => void onCreateExpectedFolder?.(node.expectedPath!),
    });
  }
  if (node.scene) {
    actions.push({
      id: "open-scene",
      label: "Open Scene Preview",
      run: () => onProjectNodeActivated?.(node),
    });
    actions.push({
      id: "regenerate-scene",
      label: "Regenerate Preview",
      run: () => onProjectNodeActivated?.(node),
    });
    actions.push({
      id: "validate-scene",
      label: "Validate Scene",
      run: () => onProjectNodeActivated?.(node),
    });
  }
  if (node.expectedPath) {
    actions.push({
      id: "copy-expected",
      label: "Copy Expected Path",
      run: () => void navigator.clipboard.writeText(node.expectedPath!),
    });
  }
  if (node.path) {
    actions.push({
      id: "copy-path",
      label: "Copy Path",
      run: () => void navigator.clipboard.writeText(node.path!),
    });
  }
  if (actions.length === 0) {
    return null;
  }
  return (
    <div
      className="project-node-context-menu"
      style={{ left: menu.x, top: menu.y }}
      onMouseLeave={onClose}
    >
      <header>{node.label}</header>
      {actions.map((action) => (
        <button
          key={action.id}
          type="button"
          onClick={() => {
            action.run();
            onClose();
          }}
        >
          {action.label}
        </button>
      ))}
    </div>
  );
}

function projectNodeIcon(node: EngineProjectTreeNode) {
  const size = 13;
  const icon = node.icon.toLowerCase();
  if (node.kind === "overview" || icon === "info") return <Info size={size} />;
  if (node.kind === "manifest" || icon === "toml") return <FileCog size={size} />;
  if (node.kind === "scene" || icon === "play") return <Play size={size} />;
  if (node.kind === "sceneDocument" || icon === "yml") return <FileCode2 size={size} />;
  if (node.kind === "sceneScript" || icon === "rh") return <Code2 size={size} />;
  if (node.kind === "assetCategory" && icon === "img") return <Image size={size} />;
  if (icon === "aud") return <Volume2 size={size} />;
  if (icon === "map") return <Map size={size} />;
  if (icon === "type") return <Type size={size} />;
  if (node.kind === "scriptPackage" || icon === "pkg") return <Package size={size} />;
  if (node.kind === "capabilities" || icon === "plug") return <Plug size={size} />;
  if (node.kind === "dependencies" || icon === "link") return <Link size={size} />;
  if (node.kind === "diagnostics" || icon === "diag") return <AlertTriangle size={size} />;
  if (node.kind === "modRoot") return <Boxes size={size} />;
  if (node.kind === "expectedFolder" || node.kind === "folder") return <Folder size={size} />;
  return <Box size={size} />;
}

function sceneProjectNode(scene: EditorSceneSummaryDto, root?: EditorProjectFileDto): ProjectTreeNode {
  const documentPath = relativeProjectPath(scene.documentPath);
  const scriptPath = relativeProjectPath(scene.scriptPath);
  const document = root ? findProjectFile(root, documentPath) : null;
  const script = root ? findProjectFile(root, scriptPath) : null;
  return {
    id: `scene:${scene.id}`,
    label: scene.label || scene.id,
    kind: "scene",
    icon: "Play",
    status: statusForEditorStatus(scene.status) === "valid" ? "ready" : statusForEditorStatus(scene.status),
    count: 2,
    exists: Boolean(document),
    scene,
    children: [
      {
        id: `scene-doc:${scene.id}`,
        label: "scene.yml",
        kind: "sceneDocument",
        icon: "Yml",
        status: document ? "valid" : "missing",
        path: documentPath,
        expectedPath: documentPath,
        exists: Boolean(document),
        ghost: !document,
        file: document ?? undefined,
      },
      {
        id: `scene-script:${scene.id}`,
        label: "scene.rhai",
        kind: "sceneScript",
        icon: "Rh",
        status: script ? "ok" : "missing",
        path: scriptPath,
        expectedPath: scriptPath,
        exists: Boolean(script),
        ghost: !script,
        file: script ?? undefined,
      },
    ],
  };
}

function assetCategoryNodes(details: EditorModDetailsDto, root?: EditorProjectFileDto): ProjectTreeNode[] {
  const categories = [
    ["textures", details.contentSummary.textures],
    ["spritesheets", details.contentSummary.spritesheets],
    ["tilemaps", details.contentSummary.tilemaps],
    ["tilesets", details.contentSummary.tilesets],
    ["audio", details.contentSummary.audio],
    ["fonts", details.contentSummary.fonts],
    ["unknown", details.contentSummary.unknownFiles],
  ] as const;
  return categories.map(([label, count]) => {
    const expectedPath = label === "unknown" ? undefined : `assets/${label}/`;
    const actualPath = assetCategoryPath(root, label);
    const exists = label === "unknown" ? count > 0 : Boolean(actualPath);
    return {
      id: `asset:${label}`,
      label,
      kind: exists ? "assetCategory" : "expectedFolder",
      icon: assetCategoryIcon(label),
      count,
      status: exists ? (count > 0 ? "ok" : "empty") : "missing",
      path: actualPath,
      expectedPath,
      exists,
      empty: exists && count === 0,
      ghost: !exists,
    };
  });
}

function fileProjectNode(file: EditorProjectFileDto, kind: ProjectTreeNodeKind): ProjectTreeNode {
  return {
    id: `${kind}:${file.relativePath}`,
    label: file.name,
    kind,
    icon: fileIcon(file),
    status: "ok",
    path: file.relativePath,
    exists: true,
    file,
  };
}

function rootChildExists(root: EditorProjectFileDto | undefined, relativePath: string): boolean {
  return root ? Boolean(findProjectFile(root, relativePath.replace(/\/$/, ""))) : false;
}

function assetCategoryPath(root: EditorProjectFileDto | undefined, label: string): string | undefined {
  if (!root || label === "unknown") return undefined;
  const preferred = `assets/${label}`;
  if (rootChildExists(root, preferred)) return preferred;
  if (rootChildExists(root, label)) return label;
  return undefined;
}

function relativeProjectPath(path: string): string {
  const normalized = normalizePath(path);
  const scenesIndex = normalized.indexOf("scenes/");
  if (scenesIndex >= 0) return normalized.slice(scenesIndex);
  const scriptsIndex = normalized.indexOf("scripts/");
  if (scriptsIndex >= 0) return normalized.slice(scriptsIndex);
  return normalized;
}

function statusForEditorStatus(status: string): ProjectTreeNodeStatus {
  if (status === "valid") return "valid";
  if (status === "warning" || status === "missingDependency") return "warn";
  if (status === "error" || status === "invalidManifest" || status === "missingSceneFile" || status === "previewFailed") return "error";
  return "ok";
}

function assetTotal(details: EditorModDetailsDto): number {
  return details.contentSummary.textures +
    details.contentSummary.spritesheets +
    details.contentSummary.tilemaps +
    details.contentSummary.tilesets +
    details.contentSummary.audio +
    details.contentSummary.fonts +
    details.contentSummary.unknownFiles;
}

function assetCategoryIcon(label: string): string {
  if (label === "textures") return "Img";
  if (label === "spritesheets") return "Grid";
  if (label === "tilemaps") return "Map";
  if (label === "tilesets") return "Tile";
  if (label === "audio") return "Aud";
  if (label === "fonts") return "Type";
  return "?";
}

function isSceneOwnedScript(details: EditorModDetailsDto, relativePath: string): boolean {
  return details.scenes.some((scene) => relativeProjectPath(scene.scriptPath) === relativePath);
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
        <CountRow label="Fonts" value={summary.fonts} />
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

function ScenesBrowser({
  details,
  projectTree,
  selectedScene,
  onSelectScene,
  onSelectFile,
}: {
  details: EditorModDetailsDto | null;
  projectTree?: EditorProjectTreeDto;
  selectedScene: EditorSceneSummaryDto | null;
  onSelectScene: (scene: EditorSceneSummaryDto) => Promise<void>;
  onSelectFile: (file: EditorProjectFileDto) => void;
}) {
  if (!details) {
    return <p className="muted workspace-empty">No scenes loaded.</p>;
  }

  return (
    <div className="dock-scroll">
      <SectionTitle title={`Scenes ${details.scenes.length}`} />
      {details.scenes.map((scene) => {
        const document = projectTree ? findProjectFile(projectTree.root, relativeProjectPath(scene.documentPath)) : null;
        const script = projectTree ? findProjectFile(projectTree.root, relativeProjectPath(scene.scriptPath)) : null;
        return (
          <section key={scene.id} className={`workspace-section scene-browser-card ${scene.id === selectedScene?.id ? "selected" : ""}`}>
            <button type="button" className="workspace-row selected scene-browser-main" onClick={() => void onSelectScene(scene)}>
              <span className="dock-icon dock-icon-cyan"><Play size={13} /></span>
              <span>
                <strong>{scene.label}</strong>
                <small>{scene.id} · {scene.launcherVisible ? "launcher visible" : "hidden"}</small>
              </span>
              <em className={`badge status-${scene.status}`}>{scene.status}</em>
            </button>
            <div className="scene-browser-files">
              <button type="button" disabled={!document} onClick={() => document && onSelectFile(document)}>
                <span>scene.yml</span>
                <em className={`badge ${document ? "badge-valid" : "badge-warning"}`}>{document ? "yaml" : "missing"}</em>
              </button>
              <button type="button" disabled={!script} onClick={() => script && onSelectFile(script)}>
                <span>scene.rhai</span>
                <em className={`badge ${script ? "badge-valid" : "badge-warning"}`}>{script ? "rhai" : "missing"}</em>
              </button>
            </div>
          </section>
        );
      })}
    </div>
  );
}

function ScriptsBrowser({
  details,
  projectTree,
  onSelectFile,
}: {
  details: EditorModDetailsDto | null;
  projectTree?: EditorProjectTreeDto;
  onSelectFile: (file: EditorProjectFileDto) => void;
}) {
  const files = projectTree ? flattenProjectFiles(projectTree.root).filter((file) => file.kind === "script") : [];
  const packages = projectTree ? flattenProjectFiles(projectTree.root).filter((file) => file.relativePath.startsWith("packages/")) : [];
  return (
    <div className="dock-scroll">
      <SectionTitle title={`Scripts ${files.length}`} />
      {files.length ? files.map((file) => (
        <button key={file.relativePath} type="button" className="workspace-row" onClick={() => onSelectFile(file)}>
          <span className="dock-icon dock-icon-green">Rh</span>
          <span>
            <strong>{file.name}</strong>
            <small>{file.relativePath}</small>
          </span>
          <em className="badge badge-muted">rhai</em>
        </button>
      )) : <p className="muted workspace-note">No Rhai scripts indexed.</p>}
      <SectionTitle title={`Packages ${details?.contentSummary.packages ?? packages.length}`} />
      {packages.length ? packages.slice(0, 80).map((file) => (
        <button key={file.relativePath} type="button" className="workspace-row" onClick={() => onSelectFile(file)}>
          <span className="dock-icon dock-icon-blue">Pkg</span>
          <span>
            <strong>{file.name}</strong>
            <small>{file.relativePath}</small>
          </span>
        </button>
      )) : <p className="muted workspace-note">No package files indexed.</p>}
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
  return ["texture", "spritesheet", "audio", "font", "tilemap", "tileset", "script", "sceneDocument", "manifest", "yaml"].includes(kind);
}

function isReadableTextFile(file: EditorProjectFileDto): boolean {
  return canReadProjectFileContent(file);
}

function fileIcon(file: EditorProjectFileDto): string {
  if (file.isDir) return "Dir";
  if (file.kind === "manifest") return "T";
  if (file.kind === "sceneDocument") return "Y";
  if (file.kind === "sceneScript") return "Rh";
  if (file.kind === "scriptPackage") return "Pkg";
  if (file.kind === "script") return "Rh";
  if (file.kind === "texture") return "Tx";
  if (file.kind === "spritesheet") return "Sp";
  if (file.kind === "audio") return "Au";
  if (file.kind === "font") return "Fn";
  if (file.kind === "tilemap") return "Tm";
  if (file.kind === "tileset") return "Ts";
  return "F";
}

export function normalizePath(path: string): string {
  return path.replace(/\\/g, "/");
}

export function findProjectFile(root: EditorProjectFileDto, relativePath: string): EditorProjectFileDto | null {
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

export function fileDiagnosticsFor(file: EditorProjectFileDto, content?: { diagnostics: Array<{ level: "info" | "warning" | "error"; code: string; message: string; path?: string | null }> }) {
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

export function FileWorkspace({
  file,
  content,
  onReveal,
}: {
  file: EditorProjectFileDto;
  content?: { content: string; language: string };
  onReveal: () => void;
}) {
  const descriptor = resolveFileWorkspaceDescriptor(file);
  return <ResolvedFileWorkspace file={file} content={content} onReveal={onReveal} descriptor={descriptor} />;
}

function ResolvedFileWorkspace({
  file,
  content,
  descriptor,
  onReveal,
}: {
  file: EditorProjectFileDto | null;
  content?: { content: string; language: string } | null;
  descriptor?: FileWorkspaceDescriptor;
  onReveal?: () => void;
}) {
  if (!file) {
    return <p className="muted workspace-empty">No file selected.</p>;
  }

  const resolved = descriptor ?? resolveFileWorkspaceDescriptor(file);
  const effectiveLanguage = workspaceDescriptorLanguage(resolved, content ?? undefined);
  const metadata = (
    <div className="file-metadata-strip">
      <span>{resolved.fileKind}</span>
      <span>{resolved.shape}</span>
      <span>{resolved.editable ? "editable" : "readonly"}</span>
      <span>{formatBytes(file.sizeBytes)}</span>
      <span>{file.path}</span>
    </div>
  );

  return (
    <div className="file-workbench">
      <div className="scene-workbench-toolbar">
        <div className="scene-heading">
          <span className="dock-icon dock-icon-cyan">{resolved.iconText}</span>
          <strong>{file.name}</strong>
          <span>{file.relativePath}</span>
          <span className="badge badge-info">{resolved.title}</span>
        </div>
        <div className="scene-heading-actions">
          {onReveal ? (
            <button className="button button-tool" type="button" onClick={onReveal}>
              <FolderOpen size={14} />
              Reveal
            </button>
          ) : null}
        </div>
      </div>

      {resolved.shape === "preview-plus-inspector" ? (
        <>
          <div className="file-preview-stage">
            {isImageFile(file) ? (
              <img className="file-image-preview" src={convertFileSrc(file.path)} alt={file.name} draggable={false} />
            ) : (
              <div className="file-preview-empty">
                <FileCode2 size={40} />
                <strong>{resolved.title}</strong>
                <span>{file.relativePath}</span>
              </div>
            )}
          </div>
          {metadata}
        </>
      ) : resolved.shape === "canvas-editor" ? (
        <>
          <div className="file-preview-stage file-domain-placeholder">
            <div className="file-preview-empty">
              <strong>{resolved.title} Workspace</strong>
              <span>Groundwork is ready. Domain editor surface plugs in here.</span>
            </div>
            {content?.content ? (
              <pre className="file-code-preview file-code-preview-overlay" data-language={effectiveLanguage}>
                <code>{content.content}</code>
              </pre>
            ) : null}
          </div>
          {metadata}
        </>
      ) : resolved.shape === "form-plus-source" ? (
        <>
          <div className="file-form-source-layout">
            <section className="file-form-summary">
              <strong>{resolved.title}</strong>
              <span>{resolved.fileKind}</span>
              <span>{resolved.openMode}</span>
              <span>{resolved.editable ? "Will support structured editing" : "Read-only surface"}</span>
            </section>
            <div className="file-preview-stage">
              {content?.content ? (
                <pre className="file-code-preview" data-language={effectiveLanguage}>
                  <code>{content.content}</code>
                </pre>
              ) : (
                <div className="file-preview-empty">
                  <FileCode2 size={40} />
                  <strong>{resolved.title}</strong>
                  <span>{canReadProjectFileContent(file) ? "Loading structured source..." : file.relativePath}</span>
                </div>
              )}
            </div>
          </div>
          {metadata}
        </>
      ) : resolved.shape === "text-editor" ? (
        <>
          <div className="file-preview-stage">
            {content?.content ? (
              <pre className="file-code-preview" data-language={effectiveLanguage}>
                <code>{content.content}</code>
              </pre>
            ) : (
              <div className="file-preview-empty">
                <FileCode2 size={40} />
                <strong>{resolved.title}</strong>
                <span>{canReadProjectFileContent(file) ? "Loading text preview..." : file.relativePath}</span>
              </div>
            )}
          </div>
          {metadata}
        </>
      ) : (
        <>
          <div className="file-preview-stage">
            <div className="file-preview-empty">
              <AlertTriangle size={40} />
              <strong>{resolved.title}</strong>
              <span>{file.relativePath}</span>
            </div>
          </div>
          {metadata}
        </>
      )}
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
            <dd>{[selectedEntity.hasTransform2 ? "2D" : null, selectedEntity.hasTransform3 ? "3D" : null].filter(Boolean).join(", ") || "none"}</dd>
            <dt>Properties</dt>
            <dd>{selectedEntity.propertyCount}</dd>
          </dl>
          <div className="tag-list workspace-component-tags">
            {selectedEntity.componentTypes.length ? (
              selectedEntity.componentTypes.map((component, index) => <span key={`${component}:${index}`} className="tag">{component}</span>)
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

function EventTable({
  events,
  filter,
  onFilterChange,
  onSearchChange,
  onSessionFilterChange,
  onSourceFilterChange,
  search,
  sessionFilter,
  sourceFilter,
  windowEvents,
}: {
  events: Array<{ type: string }>;
  filter: string;
  onFilterChange: (filter: string) => void;
  onSearchChange: (search: string) => void;
  onSessionFilterChange: (filter: string) => void;
  onSourceFilterChange: (filter: string) => void;
  search: string;
  sessionFilter: string;
  sourceFilter: string;
  windowEvents: WindowBusEvent[];
}) {
  const eventTypes = Array.from(new Set(windowEvents.map((event) => event.type))).sort();
  const sessions = Array.from(new Set(windowEvents.flatMap((event) => (event.sessionId ? [event.sessionId] : [])))).sort();
  const sources = Array.from(new Set(windowEvents.flatMap((event) => (event.sourceWindow ? [event.sourceWindow] : [])))).sort();
  const normalizedSearch = search.trim().toLowerCase();
  const filteredWindowEvents = windowEvents.filter((event) => {
    if (filter !== "all" && event.type !== filter) return false;
    if (sessionFilter !== "all" && event.sessionId !== sessionFilter) return false;
    if (sourceFilter !== "all" && event.sourceWindow !== sourceFilter) return false;
    if (!normalizedSearch) return true;
    return `${event.type} ${event.sourceWindow ?? ""} ${event.sessionId ?? ""} ${formatWindowEventPayload(event)}`
      .toLowerCase()
      .includes(normalizedSearch);
  });

  async function copyPayload(event: WindowBusEvent) {
    await navigator.clipboard.writeText(JSON.stringify(event, null, 2));
  }

  return (
    <div className="event-log-panel">
      <div className="event-log-toolbar">
        <input placeholder="Search events..." value={search} onChange={(event) => onSearchChange(event.target.value)} />
        <select value={filter} onChange={(event) => onFilterChange(event.target.value)}>
          <option value="all">All window events</option>
          {eventTypes.map((type) => <option key={type} value={type}>{type}</option>)}
        </select>
        <select value={sessionFilter} onChange={(event) => onSessionFilterChange(event.target.value)}>
          <option value="all">All sessions</option>
          {sessions.map((session) => <option key={session} value={session}>{session}</option>)}
        </select>
        <select value={sourceFilter} onChange={(event) => onSourceFilterChange(event.target.value)}>
          <option value="all">All sources</option>
          {sources.map((source) => <option key={source} value={source}>{source}</option>)}
        </select>
      </div>
      <table className="workspace-table">
        <thead>
          <tr>
            <th>Time</th>
            <th>Type</th>
            <th>Source</th>
            <th>Session</th>
            <th>Summary</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {filteredWindowEvents.map((event) => (
            <tr key={event.eventId}>
              <td>{new Date(event.timestampMs).toLocaleTimeString()}</td>
              <td><code>{event.type}</code></td>
              <td>{event.sourceWindow ?? "app"}</td>
              <td>{event.sessionId ?? ""}</td>
              <td>{formatWindowEventPayload(event)}</td>
              <td>
                <button className="button button-ghost compact-button" type="button" onClick={() => void copyPayload(event)}>
                  Copy
                </button>
              </td>
            </tr>
          ))}
          {events.map((event, index) => (
            <tr key={`${event.type}:${index}`}>
              <td></td>
              <td><code>{event.type}</code></td>
              <td>{index === 0 ? "latest" : "editor"}</td>
              <td></td>
              <td></td>
              <td></td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function formatWindowEventPayload(event: WindowBusEvent): string {
  switch (event.type) {
    case "ThemeSettingsChanged":
      return event.payload.activeThemeId;
    case "FontSettingsChanged":
      return event.payload.activeFontId;
    case "WorkspaceOpened":
      return `${event.payload.modId} · ${event.payload.sessionId}`;
    case "WorkspaceClosed":
    case "SessionClosed":
      return event.payload.sessionId;
    case "WindowCloseRequested":
    case "WindowFocused":
      return event.payload.windowLabel;
    case "CacheInvalidated":
      return `${event.payload.cacheKind}:${event.payload.reason}`;
    default:
      return "";
  }
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

function formatTaskTime(value: number): string {
  return new Date(value).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
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
