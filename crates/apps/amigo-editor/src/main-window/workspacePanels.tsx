import { useEffect, useRef, useState } from "react";
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
  Image as ImageIcon,
  Info,
  Link,
  Map as MapIcon,
  Package,
  Play,
  Plug,
  Search,
} from "lucide-react";
import { listenWindowBus } from "../app/windowBus";
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
  AssetRegistryDto,
  CreateAssetImportOptionsDto,
  ManagedAssetDto,
  RawAssetFileDto,
  ScenePreviewDto,
} from "../api/dto";
import { createAssetDescriptor, getAssetRegistry } from "../api/editorApi";
import { assetFolderVisualForKind, assetVisualForKind } from "../assets/assetVisualRegistry";
import { ComponentHost } from "../editor-components/componentHost";
import { editorComponentById, iconForEditorComponent } from "../editor-components/componentRegistry";
import type { ComponentToolbarState, EditorComponentContext, EditorComponentInstance } from "../editor-components/componentTypes";
import { ImageAssetEditor } from "../editors/image/ImageAssetEditor";
import { SheetEditor } from "../editors/sheet/SheetEditor";
import { TilemapEditor } from "../editors/tilemap/TilemapEditor";
import { DiagnosticsList } from "../startup/DiagnosticsList";
import { EngineSlideshowPreview } from "../startup/EngineSlideshowPreview";
import { FolderView, type FolderViewGroup } from "../ui/folder-view/FolderView";
import type { FolderViewStatus } from "../ui/folder-view/folderViewTypes";
import { fileSrc } from "../utils/fileSrc";
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
  onFileDirtyChange?: (path: string, dirty: boolean) => void;
  onProjectTreeRefresh?: () => void;
  onProjectNodeActivated?: (node: EditorProjectStructureNodeDto | ProjectTreeNode) => void;
  selectScene?: (scene: EditorSceneSummaryDto) => Promise<void>;
  selectSceneEntity?: (entityId: string) => void;
  setEventFilter?: (filter: string) => void;
  setEventSearch?: (value: string) => void;
  setEventSessionFilter?: (filter: string) => void;
  setEventSourceFilter?: (filter: string) => void;
  tasks?: Array<{ id: string; label: string; status: string; startedAt: number; progress?: number }>;
  toolbarState?: ComponentToolbarState;
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
      return (
        <ResolvedFileWorkspace
          file={services.selectedFile ?? null}
          content={services.selectedFileContent ?? undefined}
          onReveal={services.onRevealSelectedFile}
        />
      );
    case "file.image-asset":
      if (services.details?.id && services.selectedFile && services.selectedFileContent) {
        return (
          <ImageAssetEditor
            content={services.selectedFileContent}
            file={services.selectedFile}
            modId={services.details.id}
            onDirtyChange={services.onFileDirtyChange}
            onReveal={services.onRevealSelectedFile}
            onSaved={services.onProjectTreeRefresh}
          />
        );
      }
      return (
        <ResolvedFileWorkspace
          file={services.selectedFile ?? null}
          content={services.selectedFileContent ?? undefined}
          onReveal={services.onRevealSelectedFile}
        />
      );
    case "file.texture":
    case "file.raw-image":
    case "file.sprite":
    case "file.atlas":
    case "file.tileset":
      if (
        context.sessionId &&
        instance.resourceUri &&
        (instance.componentId === "file.tileset" ||
          ((instance.componentId === "file.sprite" || instance.componentId === "file.atlas") &&
            services.selectedFile &&
            canReadProjectFileContent(services.selectedFile)))
      ) {
        return (
          <SheetEditor
            resourceUri={instance.resourceUri}
            sessionId={context.sessionId}
            onDirtyChange={services.onFileDirtyChange}
            onSaved={services.onProjectTreeRefresh}
            onReveal={services.onRevealSelectedFile}
          />
        );
      }
      return (
        <ResolvedFileWorkspace
          file={services.selectedFile ?? null}
          content={services.selectedFileContent ?? undefined}
          onReveal={services.onRevealSelectedFile}
        />
      );
    case "file.tilemap":
      if (context.sessionId && instance.resourceUri) {
        return (
          <TilemapEditor
            resourceUri={instance.resourceUri}
            sessionId={context.sessionId}
            onDirtyChange={services.onFileDirtyChange}
          />
        );
      }
      return (
        <ResolvedFileWorkspace
          file={services.selectedFile ?? null}
          content={services.selectedFileContent ?? undefined}
          onReveal={services.onRevealSelectedFile}
        />
      );
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
          toolbarState={services.toolbarState}
        />
      );
    case "assets.browser":
      return (
        <AssetBrowser
          details={services.details ?? null}
          sessionId={context.sessionId ?? undefined}
          loading={services.projectTreeTask?.status === "running"}
          onSelectFile={(file) => services.handleSelectProjectFile?.(file)}
          onRefreshProjectTree={services.onProjectTreeRefresh}
          projectTree={services.projectTree}
          selectedFilePath={services.selectedFile?.relativePath ?? null}
          toolbarState={services.toolbarState}
        />
      );
    case "files.browser":
      return (
        <FilesBrowser
          details={services.details ?? null}
          loading={services.projectTreeTask?.status === "running"}
          onSelectFile={(file) => services.handleSelectProjectFile?.(file)}
          projectTree={services.projectTree}
          selectedFilePath={services.selectedFile?.relativePath ?? null}
          toolbarState={services.toolbarState}
        />
      );
    case "scripts.browser":
      return (
        <ScriptsBrowser
          details={services.details ?? null}
          onSelectFile={(file) => services.handleSelectProjectFile?.(file)}
          projectTree={services.projectTree}
          toolbarState={services.toolbarState}
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
      return <ProblemsTable diagnostics={services.allProblems ?? []} toolbarState={services.toolbarState} />;
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
          toolbarState={services.toolbarState}
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

  const fallbackTree = buildEngineProjectTree(details, projectTree);
  const tree = projectStructureTree?.root
    ? mergeProjectTrees(projectStructureTree.root, fallbackTree)
    : fallbackTree;
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
  | "assetResource"
  | "assetFile"
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

function mergeProjectTrees(
  preferred: EngineProjectTreeNode,
  fallback: ProjectTreeNode,
): ProjectTreeNode {
  const fallbackChildrenById = new globalThis.Map((fallback.children ?? []).map((child) => [child.id, child]));
  const preferredChildren = preferred.children ?? [];
  const mergedChildren = preferredChildren.length > 0
    ? preferredChildren.map((child) => {
        const fallbackChild = fallbackChildrenById.get(child.id);
        return fallbackChild ? mergeProjectTrees(child, fallbackChild) : normalizeProjectTreeNode(child);
      })
    : (fallback.children ?? []);

  return normalizeProjectTreeNode(preferred, mergedChildren);
}

function normalizeProjectTreeNode(
  node: EngineProjectTreeNode,
  children?: ProjectTreeNode[],
): ProjectTreeNode {
  return {
    id: node.id,
    label: node.label,
    kind: node.kind as ProjectTreeNodeKind,
    icon: node.icon,
    status: (node.status ?? undefined) as ProjectTreeNodeStatus | undefined,
    count: node.count ?? undefined,
    path: node.path ?? undefined,
    expectedPath: node.expectedPath ?? undefined,
    exists: node.exists,
    empty: node.empty ?? false,
    ghost: node.ghost ?? false,
    file: node.file ?? undefined,
    scene: node.scene ?? undefined,
    children: children ?? (node.children ?? []).map((child) => normalizeProjectTreeNode(child)),
  };
}

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
  if (["manifest", "sceneDocument", "sceneScript", "assetResource", "assetFile", "scriptFile", "scriptPackage"].includes(node.kind)) {
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
  if (icon === "img") return <ImageIcon size={size} />;
  if (icon === "grid" || icon === "tile") return <Box size={size} />;
  if (icon === "map") return <MapIcon size={size} />;
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
    ["images", details.contentSummary.textures],
    ["sprites", details.contentSummary.spritesheets],
    ["tilesets", details.contentSummary.tilesets],
    ["tilemaps", details.contentSummary.tilemaps],
    ["fonts", details.contentSummary.fonts],
    ["audio", details.contentSummary.audio],
  ] as const;
  return categories.map(([label, count]) => {
    const expectedPath = `assets/${label}/`;
    const actualPath = assetCategoryPath(root, label);
    const children = root
      ? assetCategoryFiles(root, label).map((file) => assetResourceNode(file))
      : [];
    const actualCount = Math.max(children.length, count);
    const exists = Boolean(actualPath);
    return {
      id: `asset:${label}`,
      label,
      kind: exists ? "assetCategory" : "expectedFolder",
      icon: assetCategoryIcon(label),
      count: actualCount,
      status: exists ? (actualCount > 0 ? "ok" : "empty") : "missing",
      path: actualPath,
      expectedPath,
      exists,
      empty: exists && actualCount === 0,
      ghost: !exists,
      children,
    };
  });
}

function assetCategoryFiles(root: EditorProjectFileDto, label: string): EditorProjectFileDto[] {
  return flattenProjectFiles(root).filter((file) => assetCategoryMatchesFile(label, file));
}

function assetCategoryMatchesFile(label: string, file: EditorProjectFileDto): boolean {
  if (label === "images") return file.kind === "imageAsset";
  if (label === "sprites") return file.kind === "spritesheet";
  if (label === "tilemaps") return file.kind === "tilemap";
  if (label === "tilesets") return file.kind === "tileset";
  if (label === "fonts") return file.kind === "font";
  if (label === "audio") return file.kind === "audio";
  return false;
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

function assetResourceNode(file: EditorProjectFileDto): ProjectTreeNode {
  return {
    id: `assetResource:${file.relativePath}`,
    label: assetDisplayLabel(file),
    kind: "assetResource",
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
  if (!root) return undefined;
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
    details.contentSummary.fonts +
    details.contentSummary.audio;
}

function assetCategoryIcon(label: string): string {
  if (label === "images") return "Img";
  if (label === "sprites") return "Grid";
  if (label === "tilemaps") return "Map";
  if (label === "tilesets") return "Tile";
  if (label === "fonts") return "Type";
  if (label === "audio") return "Aud";
  return "?";
}

function assetDisplayLabel(file: EditorProjectFileDto): string {
  return file.name.replace(
    /\.(image|sprite|atlas|tileset|tile-ruleset|tilemap|font|audio|particle|material|ui)\.ya?ml$/i,
    "",
  );
}

function isSceneOwnedScript(details: EditorModDetailsDto, relativePath: string): boolean {
  return details.scenes.some((scene) => relativeProjectPath(scene.scriptPath) === relativePath);
}

function FilesBrowser({
  details,
  projectTree,
  loading,
  selectedFilePath,
  onSelectFile,
  toolbarState,
}: {
  details: EditorModDetailsDto | null;
  projectTree?: EditorProjectTreeDto;
  loading: boolean;
  selectedFilePath: string | null;
  onSelectFile: (file: EditorProjectFileDto) => void;
  toolbarState?: ComponentToolbarState;
}) {
  const [search, setSearch] = useState("");
  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({});

  if (!details || !projectTree) {
    return <p className="muted workspace-empty">No project files loaded.</p>;
  }

  const tree = fileBrowserTree(details, projectTree.root);
  const viewMode = String(toolbarState?.viewMode ?? "tree");
  const fileFilter = String(toolbarState?.fileFilter ?? "all");
  const flatFiles = flattenProjectFiles(projectTree.root).filter((file) =>
    fileMatchesFilesBrowserFilter(file, fileFilter) &&
    matchesSearch([file.name, file.relativePath, file.kind], search)
  );

  return (
    <div className="dock-scroll project-explorer-panel">
      <label className="project-tree-searchbar">
        <Search size={13} />
        <input
          placeholder="Search files..."
          value={search}
          onChange={(event) => setSearch(event.target.value)}
        />
      </label>
      <div className="project-tree-separator" aria-hidden="true" />
      {loading ? <p className="muted workspace-note">Indexing project files...</p> : null}
      {viewMode === "flat" ? (
        <div className="workspace-list-view">
          {flatFiles.map((file) => (
            <button
              key={file.relativePath}
              type="button"
              className={`workspace-row-button ${selectedFilePath === file.relativePath ? "selected" : ""}`}
              onClick={() => onSelectFile(file)}
            >
              <span className="dock-icon dock-icon-blue">{fileIcon(file)}</span>
              <span>
                <strong>{file.name}</strong>
                <small>{file.relativePath}</small>
              </span>
            </button>
          ))}
          {flatFiles.length === 0 ? <p className="muted workspace-note">No matching files.</p> : null}
        </div>
      ) : (
        <EngineProjectTree
          node={filterFileBrowserTree(tree, fileFilter)}
          depth={0}
          selectedFilePath={selectedFilePath}
          selectedSceneId={null}
          onSelectFile={onSelectFile}
          onSelectScene={() => Promise.resolve()}
          onOpenContextMenu={() => undefined}
          search={search}
          collapsed={collapsed}
          selectedNodeId={null}
          onToggle={(nodeId) => setCollapsed((current) => ({ ...current, [nodeId]: !current[nodeId] }))}
        />
      )}
    </div>
  );
}

function fileBrowserTree(details: EditorModDetailsDto, root: EditorProjectFileDto): ProjectTreeNode {
  return {
    id: `files:${details.id}`,
    label: details.id,
    kind: "modRoot",
    icon: "Mod",
    status: statusForEditorStatus(details.status),
    count: details.contentSummary.totalFiles,
    path: root.path,
    exists: true,
    children: root.children.map(fileBrowserNode),
  };
}

function fileBrowserNode(file: EditorProjectFileDto): ProjectTreeNode {
  if (file.isDir) {
    return {
      id: `dir:${file.relativePath}`,
      label: file.name,
      kind: "folder",
      icon: "Folder",
      status: file.children.length ? "ok" : "empty",
      path: file.relativePath,
      exists: true,
      empty: file.children.length === 0,
      children: file.children.map(fileBrowserNode),
    };
  }

  return fileProjectNode(file, "assetFile");
}

function filterFileBrowserTree(node: ProjectTreeNode, filter: string): ProjectTreeNode {
  if (!node.children?.length) return node;
  const children = node.children
    .map((child) => filterFileBrowserTree(child, filter))
    .filter((child) => {
      if (child.children?.length) return true;
      return child.file ? fileMatchesFilesBrowserFilter(child.file, filter) : filter === "all";
    });
  return { ...node, children };
}

function fileMatchesFilesBrowserFilter(file: EditorProjectFileDto, filter: string): boolean {
  if (filter === "all") return true;
  if (filter === "descriptors") {
    return /(\.image|\.sprite|\.atlas|\.tileset|\.tile-ruleset|\.tilemap|\.font|\.audio|\.particle|\.material|\.ui)\.ya?ml$/i.test(file.name);
  }
  if (filter === "raw") return file.kind === "rawImage" || file.kind === "rawAudio" || file.kind === "rawFont";
  if (filter === "scripts") return file.kind === "script" || file.kind === "sceneScript" || file.kind === "scriptPackage";
  if (filter === "scenes") return file.kind === "sceneDocument" || normalizePath(file.relativePath).startsWith("scenes/");
  return true;
}

function AssetBrowser({
  details,
  sessionId,
  projectTree,
  loading,
  selectedFilePath,
  onSelectFile,
  onRefreshProjectTree,
  toolbarState,
}: {
  details: EditorModDetailsDto | null;
  sessionId?: string;
  projectTree?: EditorProjectTreeDto;
  loading: boolean;
  selectedFilePath: string | null;
  onSelectFile: (file: EditorProjectFileDto) => void;
  onRefreshProjectTree?: () => void;
  toolbarState?: ComponentToolbarState;
}) {
  const [registry, setRegistry] = useState<AssetRegistryDto | null>(null);
  const [search, setSearch] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function refreshRegistry() {
    if (!sessionId) return;
    setBusy(true);
    setError(null);
    try {
      setRegistry(await getAssetRegistry(sessionId));
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    void refreshRegistry();
  }, [sessionId]);

  useEffect(() => {
    if (toolbarState?.refreshNonce) {
      void refreshRegistry();
    }
  }, [toolbarState?.refreshNonce]);

  useEffect(() => {
    if (!details?.id) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listenWindowBus((event) => {
      if (disposed) return;
      if (
        (event.type === "AssetRegistryChanged" || event.type === "AssetDescriptorChanged") &&
        event.payload.modId === details.id
      ) {
        void refreshRegistry();
        onRefreshProjectTree?.();
      }
    }).then((cleanup) => {
      unlisten = cleanup;
      if (disposed) cleanup();
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [details?.id, sessionId]);

  async function createDescriptorFromRaw(raw: RawAssetFileDto) {
    if (!sessionId) return;
    const suggestedKind = suggestedDescriptorKind(raw);
    const kind = window.prompt("Descriptor kind: image, tileset, sprite", suggestedKind);
    if (!kind) return;
    const normalizedKind = kind.trim().toLowerCase();
    if (!["image", "tileset", "sprite"].includes(normalizedKind)) {
      setError("Only image, tileset and sprite descriptors are available in the current MVP.");
      return;
    }
    const suggestedId = raw.relativePath.split("/").pop()?.replace(/\.[^.]+$/, "").toLowerCase().replace(/[^a-z0-9-]+/g, "-") ?? "asset";
    const assetId = window.prompt("Asset id", suggestedId);
    if (!assetId) return;
    const importOptions = normalizedKind === "tileset" || normalizedKind === "sprite"
      ? promptImageSheetImportOptions(raw, normalizedKind)
      : null;
    if ((normalizedKind === "tileset" || normalizedKind === "sprite") && !importOptions) {
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const created = await createAssetDescriptor(sessionId, {
        rawFilePath: raw.relativePath,
        kind: normalizedKind,
        assetId,
        importOptions,
      });
      await refreshRegistry();
      onRefreshProjectTree?.();
      onSelectFile(projectFileFromManagedAsset(created));
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  if (!details || !sessionId) {
    return <p className="muted workspace-empty">No assets loaded.</p>;
  }

  const registryManaged = (registry?.managedAssets ?? []).filter(isMvpManagedAsset);
  const fallbackManaged = buildManagedAssetFallback(details.id, projectTree?.root);
  const managed = registryManaged.length
    ? registryManaged
    : fallbackManaged.filter((asset) => isMvpManagedAsset(asset));
  const raw = (registry?.rawFiles ?? []).filter(isMvpRawAsset);
  const viewMode = String(toolbarState?.viewMode ?? "tree");
  const kindFilter = String(toolbarState?.kind ?? "all");
  const issuesOnly = Boolean(toolbarState?.issuesOnly ?? false);
  const filteredManaged = managed.filter((asset) => {
    if (issuesOnly && asset.status === "valid") return false;
    if (kindFilter !== "all" && asset.kind !== kindFilter) return false;
    return matchesSearch([asset.label, asset.assetKey, asset.descriptorRelativePath, asset.kind], search);
  });
  const filteredRaw = raw.filter((file) => {
    if (kindFilter !== "all" && kindFilter !== "image-2d") return false;
    if (issuesOnly && !file.orphan) return false;
    return matchesSearch([file.relativePath, file.mediaType, ...file.referencedBy], search);
  });
  const groupedManaged = groupManagedAssets(filteredManaged);

  return (
    <div className="dock-scroll">
      <label className="workspace-search">
        <span>Search</span>
        <input value={search} placeholder="Assets..." onChange={(event) => setSearch(event.target.value)} />
      </label>
      {loading || busy ? <p className="muted workspace-note">Indexing assets...</p> : null}
      {error ? <p className="muted workspace-note">{error}</p> : null}
      <div className="workspace-count-list">
        <CountRow label="Managed" value={managed.length} />
        <CountRow label="Raw" value={raw.length} />
        <CountRow label="Orphans" value={raw.filter((file) => file.orphan).length} />
        <CountRow label="Issues" value={(registry?.diagnostics ?? []).length} />
      </div>
      {viewMode === "tree" ? (
        <div className="asset-tree-view">
          <AssetRegistryTree
            groupedManaged={groupedManaged}
            rawFiles={filteredRaw}
            selectedFilePath={selectedFilePath}
            onCreateDescriptor={createDescriptorFromRaw}
            onSelectFile={onSelectFile}
          />
        </div>
      ) : viewMode === "tiles" ? (
        <AssetTileExplorer
          groupedManaged={groupedManaged}
          rawFiles={filteredRaw}
          selectedFilePath={selectedFilePath}
          onCreateDescriptor={createDescriptorFromRaw}
          onSelectFile={onSelectFile}
        />
      ) : (
        <>
          <SectionTitle title={`Managed Assets ${filteredManaged.length ? `(${filteredManaged.length})` : ""}`} />
          {filteredManaged.length ? filteredManaged.slice(0, 120).map((asset) => renderManagedAssetRow(asset, selectedFilePath, onSelectFile, "list")) : (
            <p className="muted workspace-note">No managed assets.</p>
          )}
        </>
      )}
      {viewMode === "list" && filteredRaw.length ? (
        <>
          <SectionTitle title={`Raw / Unmanaged (${filteredRaw.length})`} />
          {filteredRaw.slice(0, 120).map((file) => (
            <div key={file.relativePath} className={`workspace-row asset-registry-row ${selectedFilePath === file.relativePath ? "selected" : ""}`}>
              <button type="button" onClick={() => onSelectFile(projectFileFromRawAsset(file))}>
                <span className={`dock-icon asset-status-icon ${file.orphan ? "asset-status-warning" : "asset-status-valid"}`}>{rawAssetIcon(file.mediaType)}</span>
                <span>
                  <strong>{file.relativePath.split("/").pop()}</strong>
                  <small>{file.relativePath}</small>
                </span>
                <small className="asset-row-status">{file.orphan ? "orphan" : "referenced"}</small>
              </button>
              {file.orphan ? (
                <button type="button" className="workspace-row-action" onClick={() => void createDescriptorFromRaw(file)}>
                  descriptor
                </button>
              ) : null}
            </div>
          ))}
        </>
      ) : null}
    </div>
  );
}

function projectFileFromManagedAsset(asset: ManagedAssetDto): EditorProjectFileDto {
  return {
    name: asset.descriptorRelativePath.split("/").pop() ?? asset.assetId,
    path: asset.descriptorPath,
    relativePath: asset.descriptorRelativePath,
    kind: projectKindForManagedAsset(asset),
    isDir: false,
    sizeBytes: 0,
    children: [],
  };
}

function renderManagedAssetRow(
  asset: ManagedAssetDto,
  selectedFilePath: string | null,
  onSelectFile: (file: EditorProjectFileDto) => void,
  variant: "tree" | "list" = "list",
) {
  return (
    <div key={asset.assetKey} className={`workspace-row asset-registry-row ${variant === "tree" ? "tree-row" : ""} ${selectedFilePath === asset.descriptorRelativePath ? "selected" : ""}`}>
      <button
        type="button"
        onClick={() => onSelectFile(projectFileFromManagedAsset(asset))}
      >
        <span className={`dock-icon asset-status-icon ${assetVisualForKind(asset.kind).tone} asset-status-${asset.status}`}>
          {assetIcon(asset.kind)}
        </span>
        <span>
          <strong>{asset.label}</strong>
          <small>{variant === "tree" ? asset.descriptorRelativePath : `${assetKindLabel(asset.kind)} · ${asset.assetKey}`}</small>
        </span>
        <small className="asset-row-status">{asset.status}</small>
      </button>
    </div>
  );
}

function AssetRegistryTree({
  groupedManaged,
  rawFiles,
  selectedFilePath,
  onCreateDescriptor,
  onSelectFile,
}: {
  groupedManaged: globalThis.Map<string, ManagedAssetDto[]>;
  rawFiles: RawAssetFileDto[];
  selectedFilePath: string | null;
  onCreateDescriptor: (file: RawAssetFileDto) => Promise<void>;
  onSelectFile: (file: EditorProjectFileDto) => void;
}) {
  const groups = Array.from(groupedManaged.entries());
  return (
    <div className="asset-registry-tree">
      <div className="asset-tree-root">
        <span className="tree-twist">▾</span>
        <span className={`dock-icon asset-status-icon ${assetFolderVisualForKind("root").tone}`}>{assetFolderIcon("root")}</span>
        <strong>Assets</strong>
        <small>{groups.reduce((count, [, assets]) => count + assets.length, 0)}</small>
      </div>
      {groups.map(([kind, assets]) => (
        <section key={kind} className="asset-tree-group">
          <div className="asset-tree-folder">
            <span className="tree-twist">▾</span>
            <span className={`dock-icon asset-status-icon ${assetFolderVisualForKind(kind).tone}`}>{assetFolderIcon(kind)}</span>
            <strong>{assetKindLabel(kind)}</strong>
            <small>{assets.length}</small>
          </div>
          <div className="asset-tree-children">
            {assets.map((asset) => renderManagedAssetRow(asset, selectedFilePath, onSelectFile, "tree"))}
          </div>
        </section>
      ))}
      {rawFiles.length ? (
        <section className="asset-tree-group">
          <div className="asset-tree-folder">
            <span className="tree-twist">▾</span>
            <span className={`dock-icon asset-status-icon ${assetVisualForKind("image/raw").tone} asset-status-warning`}>{rawAssetIcon("image/raw")}</span>
            <strong>Raw Images</strong>
            <small>{rawFiles.length}</small>
          </div>
          <div className="asset-tree-children">
            {rawFiles.slice(0, 80).map((file) => renderRawAssetRow(file, selectedFilePath, onSelectFile, onCreateDescriptor))}
          </div>
        </section>
      ) : null}
    </div>
  );
}

function AssetTileExplorer({
  groupedManaged,
  rawFiles,
  selectedFilePath,
  onCreateDescriptor,
  onSelectFile,
}: {
  groupedManaged: globalThis.Map<string, ManagedAssetDto[]>;
  rawFiles: RawAssetFileDto[];
  selectedFilePath: string | null;
  onCreateDescriptor: (file: RawAssetFileDto) => Promise<void>;
  onSelectFile: (file: EditorProjectFileDto) => void;
}) {
  const groups: FolderViewGroup[] = [
    ...Array.from(groupedManaged.entries()).map(([kind, assets]) => ({
      id: kind,
      title: assetKindLabel(kind),
      subtitle: `${assets.length} managed assets`,
      icon: assetFolderIcon(kind),
      items: assets.map((asset) => {
        const source = asset.sourceFiles.find((file) => file.exists && /\.(png|jpe?g|webp)$/i.test(file.relativePath));
        const visual = assetVisualForKind(asset.kind);
        return {
          id: asset.assetKey,
          title: asset.label,
          subtitle: visual.label,
          thumbnailSrc: source ? fileSrc(source.path) : undefined,
          icon: visual.icon,
          status: folderStatusForAsset(asset.status),
          tone: visual.tone,
          selected: selectedFilePath === asset.descriptorRelativePath,
          kind: asset.kind,
          onOpen: () => onSelectFile(projectFileFromManagedAsset(asset)),
        };
      }),
    })),
    {
      id: "raw-images",
      title: "Raw Images",
      subtitle: `${rawFiles.length} source files`,
      icon: rawAssetIcon("image/raw"),
      items: rawFiles.map((file) => ({
        id: file.relativePath,
        title: file.relativePath.split("/").pop() ?? file.relativePath,
        subtitle: file.orphan ? "Raw orphan" : "Raw referenced",
        thumbnailSrc: fileSrc(file.path),
        icon: rawAssetIcon(file.mediaType),
        status: file.orphan ? "warning" : "valid",
        tone: assetVisualForKind(file.mediaType).tone,
        selected: selectedFilePath === file.relativePath,
        kind: file.mediaType,
        onOpen: () => onSelectFile(projectFileFromRawAsset(file)),
        actions: file.orphan ? [{
          id: "descriptor",
          label: "descriptor",
          onRun: () => void onCreateDescriptor(file),
        }] : undefined,
      })),
    },
  ];

  return (
    <FolderView
      density="compact"
      emptyMessage="No assets match the current filter."
      groups={groups}
      thumbnailMode="pixel"
    />
  );
}

function folderStatusForAsset(status: string): FolderViewStatus {
  if (status === "valid") return "valid";
  if (status === "missingSource") return "missing";
  if (status === "error") return "error";
  return "warning";
}

function renderRawAssetRow(
  file: RawAssetFileDto,
  selectedFilePath: string | null,
  onSelectFile: (file: EditorProjectFileDto) => void,
  onCreateDescriptor: (file: RawAssetFileDto) => Promise<void>,
) {
  return (
    <div key={file.relativePath} className={`workspace-row asset-registry-row tree-row ${selectedFilePath === file.relativePath ? "selected" : ""}`}>
      <button type="button" onClick={() => onSelectFile(projectFileFromRawAsset(file))}>
        <span className={`dock-icon asset-status-icon ${assetVisualForKind(file.mediaType).tone} ${file.orphan ? "asset-status-warning" : "asset-status-valid"}`}>
          {rawAssetIcon(file.mediaType)}
        </span>
        <span>
          <strong>{file.relativePath.split("/").pop()}</strong>
          <small>{file.relativePath}</small>
        </span>
        <small className="asset-row-status">{file.orphan ? "orphan" : "referenced"}</small>
      </button>
      {file.orphan ? (
        <button type="button" className="workspace-row-action" onClick={() => void onCreateDescriptor(file)}>
          descriptor
        </button>
      ) : null}
    </div>
  );
}

function assetFolderIcon(kind: string) {
  return assetFolderVisualForKind(kind).icon;
}

function groupManagedAssets(assets: ManagedAssetDto[]): globalThis.Map<string, ManagedAssetDto[]> {
  const grouped = new globalThis.Map<string, ManagedAssetDto[]>();
  for (const asset of assets) {
    const list = grouped.get(asset.kind) ?? [];
    list.push(asset);
    grouped.set(asset.kind, list);
  }
  return grouped;
}

function assetKindLabel(kind: string): string {
  return assetVisualForKind(kind).label;
}

function buildManagedAssetFallback(modId: string, root?: EditorProjectFileDto): ManagedAssetDto[] {
  if (!root) return [];
  return flattenProjectFiles(root)
    .filter((file) => ["imageAsset", "tileset", "tilemap", "spritesheet"].includes(file.kind))
    .map((file) => managedAssetFromProjectFile(modId, file));
}

function managedAssetFromProjectFile(modId: string, file: EditorProjectFileDto): ManagedAssetDto {
  const descriptorRelativePath = normalizePath(file.relativePath);
  const assetId = descriptorRelativePath.split("/").pop()?.replace(/\.(image|sprite|atlas|tileset|tile-ruleset|tilemap)\.ya?ml$/i, "") ?? file.name;
  const area = descriptorRelativePath.startsWith("assets/")
    ? descriptorRelativePath.split("/")[1] ?? "assets"
    : "assets";
  const assetKey = `${modId}/${area}/${assetId}`;
  return {
    assetId,
    kind: managedAssetKindFromProjectFile(file),
    label: assetId,
    assetKey,
    descriptorPath: file.path,
    descriptorRelativePath: file.relativePath,
    sourceFiles: [],
    status: "valid",
    diagnostics: [],
  };
}

function managedAssetKindFromProjectFile(file: EditorProjectFileDto): string {
  if (file.kind === "imageAsset") return "image-2d";
  if (file.kind === "spritesheet") return "sprite-sheet-2d";
  if (file.kind === "tilemap") return "tilemap-2d";
  if (file.kind === "tileset") {
    const normalized = normalizePath(file.relativePath).toLowerCase();
    if (normalized.endsWith(".tile-ruleset.yml") || normalized.endsWith(".tile-ruleset.yaml")) {
      return "tile-ruleset-2d";
    }
    return "tileset-2d";
  }
  return "yaml";
}

function projectFileFromRawAsset(file: RawAssetFileDto): EditorProjectFileDto {
  return {
    name: file.relativePath.split("/").pop() ?? file.relativePath,
    path: file.path,
    relativePath: file.relativePath,
    kind: file.mediaType.startsWith("image/") ? "rawImage" : "unknown",
    isDir: false,
    sizeBytes: 0,
    children: [],
  };
}

function projectKindForManagedAsset(asset: ManagedAssetDto): string {
  if (asset.kind === "image-2d") return "imageAsset";
  if (asset.kind === "tileset-2d" || asset.kind === "tile-ruleset-2d") return "tileset";
  if (asset.kind === "tilemap-2d") return "tilemap";
  if (asset.kind === "sprite-sheet-2d") return "spritesheet";
  return "yaml";
}

function suggestedDescriptorKind(file: RawAssetFileDto): string {
  if (file.mediaType.startsWith("image/")) return "image";
  return "image";
}

function promptImageSheetImportOptions(
  file: RawAssetFileDto,
  kind: "tileset" | "sprite",
): CreateAssetImportOptionsDto | null {
  const imageWidth = Math.max(1, file.width ?? 0);
  const imageHeight = Math.max(1, file.height ?? 0);
  const tileWidth = promptPositiveInt(`${kind} tile width`, 32);
  if (tileWidth == null) return null;
  const tileHeight = promptPositiveInt(`${kind} tile height`, 32);
  if (tileHeight == null) return null;
  const defaultColumns = imageWidth > 0 ? Math.max(1, Math.floor(imageWidth / tileWidth)) : 1;
  const defaultRows = imageHeight > 0 ? Math.max(1, Math.floor(imageHeight / tileHeight)) : 1;
  const columns = promptPositiveInt(`${kind} columns`, defaultColumns);
  if (columns == null) return null;
  const rows = promptPositiveInt(`${kind} rows`, defaultRows);
  if (rows == null) return null;
  const tileCount = promptPositiveInt(`${kind} tile/frame count`, columns * rows);
  if (tileCount == null) return null;
  const marginX = promptNonNegativeInt(`${kind} margin X`, 0);
  if (marginX == null) return null;
  const marginY = promptNonNegativeInt(`${kind} margin Y`, 0);
  if (marginY == null) return null;
  const spacingX = promptNonNegativeInt(`${kind} spacing X`, 0);
  if (spacingX == null) return null;
  const spacingY = promptNonNegativeInt(`${kind} spacing Y`, 0);
  if (spacingY == null) return null;
  const fps = kind === "sprite" ? promptPositiveInt("sprite fps", 12) : null;
  if (kind === "sprite" && fps == null) return null;

  return {
    tileWidth,
    tileHeight,
    columns,
    rows,
    tileCount,
    marginX,
    marginY,
    spacingX,
    spacingY,
    fps,
  };
}

function promptPositiveInt(label: string, fallback: number): number | null {
  const value = window.prompt(label, String(fallback));
  if (value == null) return null;
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    window.alert(`${label} must be a positive integer.`);
    return null;
  }
  return parsed;
}

function promptNonNegativeInt(label: string, fallback: number): number | null {
  const value = window.prompt(label, String(fallback));
  if (value == null) return null;
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed) || parsed < 0) {
    window.alert(`${label} must be a non-negative integer.`);
    return null;
  }
  return parsed;
}

function assetIcon(kind: string) {
  return assetVisualForKind(kind).icon;
}

function rawAssetIcon(mediaType: string) {
  return assetVisualForKind(mediaType).icon;
}

function isMvpManagedAsset(asset: ManagedAssetDto): boolean {
  return ["image-2d", "tileset-2d", "tile-ruleset-2d", "tilemap-2d", "sprite-sheet-2d"].includes(asset.kind);
}

function isMvpRawAsset(file: RawAssetFileDto): boolean {
  return file.mediaType.startsWith("image/");
}

function matchesSearch(values: string[], search: string): boolean {
  const query = search.trim().toLowerCase();
  return !query || values.some((value) => value.toLowerCase().includes(query));
}

function ScenesBrowser({
  details,
  projectTree,
  selectedScene,
  toolbarState,
  onSelectScene,
  onSelectFile,
}: {
  details: EditorModDetailsDto | null;
  projectTree?: EditorProjectTreeDto;
  selectedScene: EditorSceneSummaryDto | null;
  toolbarState?: ComponentToolbarState;
  onSelectScene: (scene: EditorSceneSummaryDto) => Promise<void>;
  onSelectFile: (file: EditorProjectFileDto) => void;
}) {
  if (!details) {
    return <p className="muted workspace-empty">No scenes loaded.</p>;
  }

  const visibleOnly = Boolean(toolbarState?.visibleOnly ?? false);
  const viewMode = String(toolbarState?.viewMode ?? "list");
  const scenes = details.scenes
    .filter((scene) => !visibleOnly || scene.launcherVisible)
    .sort((left, right) => {
      if (viewMode !== "status") return 0;
      return sceneStatusRank(left.status) - sceneStatusRank(right.status) || left.label.localeCompare(right.label);
    });

  return (
    <div className="dock-scroll">
      <SectionTitle title={`Scenes ${scenes.length}`} />
      {scenes.map((scene) => {
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
      {scenes.length === 0 ? <p className="muted workspace-note">No scenes match the current toolbar filter.</p> : null}
    </div>
  );
}

function ScriptsBrowser({
  details,
  projectTree,
  toolbarState,
  onSelectFile,
}: {
  details: EditorModDetailsDto | null;
  projectTree?: EditorProjectTreeDto;
  toolbarState?: ComponentToolbarState;
  onSelectFile: (file: EditorProjectFileDto) => void;
}) {
  const viewMode = String(toolbarState?.viewMode ?? "tree");
  const packagesOnly = Boolean(toolbarState?.packagesOnly ?? false) || viewMode === "packages";
  const files = projectTree ? flattenProjectFiles(projectTree.root).filter((file) => file.kind === "script") : [];
  const packages = projectTree ? flattenProjectFiles(projectTree.root).filter((file) => file.relativePath.startsWith("packages/")) : [];
  const scriptFiles = viewMode === "flat"
    ? [...files].sort((left, right) => left.relativePath.localeCompare(right.relativePath))
    : files;
  return (
    <div className="dock-scroll">
      {!packagesOnly ? (
        <>
          <SectionTitle title={`Scripts ${scriptFiles.length}`} />
          {scriptFiles.length ? scriptFiles.map((file) => (
            <button key={file.relativePath} type="button" className="workspace-row" onClick={() => onSelectFile(file)}>
              <span className="dock-icon dock-icon-green">Rh</span>
              <span>
                <strong>{file.name}</strong>
                <small>{file.relativePath}</small>
              </span>
              <em className="badge badge-muted">rhai</em>
            </button>
          )) : <p className="muted workspace-note">No Rhai scripts indexed.</p>}
        </>
      ) : null}
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

function sceneStatusRank(status: string): number {
  if (status === "error" || status === "invalid") return 0;
  if (status === "warn" || status === "warning") return 1;
  if (status === "valid" || status === "ready" || status === "ok") return 2;
  return 3;
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
  return file.kind === "texture" || file.kind === "spritesheet" || file.kind === "rawImage" || /\.(png|jpe?g|webp)$/i.test(file.name);
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
              <RawImageWorkspace file={file} />
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

function RawImageWorkspace({ file }: { file: EditorProjectFileDto }) {
  const [zoom, setZoom] = useState(1);
  const [fitMode, setFitMode] = useState(true);
  const viewportRef = useRef<HTMLDivElement | null>(null);
  const dragRef = useRef<{ active: boolean; left: number; top: number; x: number; y: number }>({
    active: false,
    left: 0,
    top: 0,
    x: 0,
    y: 0,
  });

  function adjustZoom(delta: number) {
    setFitMode(false);
    setZoom((current) => Math.min(5, Math.max(0.1, Number((current + delta).toFixed(2)))));
  }

  function resetPan() {
    if (!viewportRef.current) return;
    viewportRef.current.scrollLeft = 0;
    viewportRef.current.scrollTop = 0;
  }

  return (
    <div
      className="file-image-stage"
      onWheel={(event) => {
        if (!event.ctrlKey) return;
        event.preventDefault();
        adjustZoom(event.deltaY > 0 ? -0.1 : 0.1);
      }}
    >
      <div className="file-image-toolbar">
        <button className="button button-icon" type="button" title="Zoom out" onClick={() => adjustZoom(-0.25)}>
          -
        </button>
        <span className="file-image-zoom-label">{Math.round(zoom * 100)}%</span>
        <button className="button button-icon" type="button" title="Fit image" onClick={() => { setFitMode(true); setZoom(1); resetPan(); }}>
          Fit
        </button>
        <button className="button button-icon" type="button" title="Reset zoom" onClick={() => { setFitMode(false); setZoom(1); resetPan(); }}>
          1:1
        </button>
        <button className="button button-icon" type="button" title="Reset pan" onClick={resetPan}>
          Pan
        </button>
        <button className="button button-icon" type="button" title="Zoom in" onClick={() => adjustZoom(0.25)}>
          +
        </button>
      </div>
      <div
        ref={viewportRef}
        className="file-image-viewport"
        onMouseDown={(event) => {
          if (event.button !== 0 && event.button !== 1) return;
          dragRef.current = {
            active: true,
            left: viewportRef.current?.scrollLeft ?? 0,
            top: viewportRef.current?.scrollTop ?? 0,
            x: event.clientX,
            y: event.clientY,
          };
        }}
        onMouseMove={(event) => {
          if (!dragRef.current.active || !viewportRef.current) return;
          viewportRef.current.scrollLeft = dragRef.current.left - (event.clientX - dragRef.current.x);
          viewportRef.current.scrollTop = dragRef.current.top - (event.clientY - dragRef.current.y);
        }}
        onMouseUp={() => {
          dragRef.current.active = false;
        }}
        onMouseLeave={() => {
          dragRef.current.active = false;
        }}
      >
        <img
          className="file-image-preview"
          src={fileSrc(file.path)}
          alt={file.name}
          draggable={false}
          style={{
            maxWidth: fitMode ? "100%" : "none",
            maxHeight: fitMode ? "100%" : "none",
            transform: fitMode ? undefined : `scale(${zoom})`,
          }}
        />
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

function ProblemsTable({
  diagnostics,
  toolbarState,
}: {
  diagnostics: Array<{ level: string; code: string; message: string; path?: string | null }>;
  toolbarState?: ComponentToolbarState;
}) {
  const level = String(toolbarState?.level ?? "all");
  const filteredDiagnostics = diagnostics.filter((diagnostic) => {
    if (level === "all") return true;
    return diagnostic.level === level;
  });

  if (filteredDiagnostics.length === 0) {
    return <p className="muted workspace-empty">No problems.</p>;
  }
  return (
    <table className="workspace-table">
      <tbody>
        {filteredDiagnostics.map((diagnostic, index) => (
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
  toolbarState,
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
  toolbarState?: ComponentToolbarState;
  windowEvents: WindowBusEvent[];
}) {
  const eventTypes = Array.from(new Set(windowEvents.map((event) => event.type))).sort();
  const sessions = Array.from(new Set(windowEvents.flatMap((event) => (event.sessionId ? [event.sessionId] : [])))).sort();
  const sources = Array.from(new Set(windowEvents.flatMap((event) => (event.sourceWindow ? [event.sourceWindow] : [])))).sort();
  const normalizedSearch = search.trim().toLowerCase();
  const category = String(toolbarState?.category ?? "all");
  const filteredWindowEvents = windowEvents.filter((event) => {
    if (!windowEventMatchesCategory(event, category)) return false;
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

function windowEventMatchesCategory(event: WindowBusEvent, category: string): boolean {
  if (category === "all") return true;
  if (category === "window") {
    return event.type.includes("Window") || event.type.includes("Workspace") || event.type.includes("Session");
  }
  if (category === "asset") return event.type.includes("Asset");
  if (category === "workspace") return event.type.includes("Workspace") || event.type.includes("Session");
  if (category === "cache") return event.type.includes("Cache") || event.type.includes("Preview");
  if (category === "settings") return event.type.includes("Settings") || event.type.includes("Theme") || event.type.includes("Font");
  return true;
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
