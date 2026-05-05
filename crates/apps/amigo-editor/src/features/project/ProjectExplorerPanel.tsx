import { useState } from "react";
import { AlertTriangle, Box, Boxes, Code2, FileCode2, FileCog, Folder, Image as ImageIcon, Info, Link, Map as MapIcon, Package, Play, Plug, Search } from "lucide-react";
import type { EditorModDetailsDto, EditorProjectFileDto, EditorProjectStructureNodeDto, EditorProjectStructureTreeDto, EditorProjectTreeDto, EditorSceneSummaryDto } from "../../api/dto";
import type { ComponentToolbarState, EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { flattenProjectFiles, findProjectFile, normalizePath } from "../files/fileTreeSelectors";
import { fileIcon } from "../files/ProjectFileTree";

export function ProjectExplorerPanel({ services }: EditorComponentProps<WorkspaceRuntimeServices>) {
  return (
    <ProjectExplorer
      details={services.details ?? null}
      loading={services.projectTreeTask?.status === "running"}
      onCreateExpectedFolder={services.onCreateExpectedFolder}
      onProjectNodeActivated={services.onProjectNodeActivated as ((node: EngineProjectTreeNode) => void) | undefined}
      onSelectFile={(file) => services.handleSelectProjectFile?.(file)}
      onSelectScene={(scene) => services.selectScene?.(scene) ?? Promise.resolve()}
      projectStructureTree={services.projectStructureTree}
      projectTree={services.projectTree}
      selectedFilePath={services.selectedFile?.relativePath ?? null}
      selectedScene={services.selectedScene ?? null}
    />
  );
}
export function ProjectExplorer({
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

export type ProjectTreeNodeStatus = "ok" | "valid" | "ready" | "warn" | "error" | "empty" | "missing" | "cached";

export type ProjectTreeNodeKind =
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

export type ProjectTreeNode = {
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

export type EngineProjectTreeNode = ProjectTreeNode | EditorProjectStructureNodeDto;

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
      projectFolderNode("raw", "Raw", details.contentSummary.textures + details.contentSummary.audio + details.contentSummary.fonts, root, 48),
      projectFolderNode("spritesheets", "Grid", details.contentSummary.spritesheets + details.contentSummary.tilesets + details.contentSummary.tilemaps, root, 64, (file) =>
        file.kind === "spritesheet" || file.kind === "tileset" || file.kind === "tilemap" || file.kind === "rawImage",
      ),
      projectFolderNode("audio", "Aud", details.contentSummary.audio, root, 24),
      projectFolderNode("fonts", "Type", details.contentSummary.fonts, root, 24),
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

export function EngineProjectTree({
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

export function ProjectNodeActionStrip({
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

function projectFolderNode(
  label: string,
  icon: string,
  summaryCount: number,
  root?: EditorProjectFileDto,
  limit = 24,
  filter?: (file: EditorProjectFileDto) => boolean,
): ProjectTreeNode {
  const exists = rootChildExists(root, label);
  const files = root ? filesUnder(root, label).filter((file) => (filter ? filter(file) : true)) : [];
  const count = Math.max(files.length, summaryCount);
  return {
    id: `group:${label}`,
    label,
    kind: exists ? "folder" : "expectedFolder",
    icon,
    count,
    status: exists ? (count ? "ok" : "empty") : "missing",
    path: exists ? label : undefined,
    expectedPath: `${label}/`,
    exists,
    empty: exists && count === 0,
    ghost: !exists,
    children: files.slice(0, limit).map((file) => assetResourceNode(file)),
  };
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

function filesUnder(root: EditorProjectFileDto, relativePath: string): EditorProjectFileDto[] {
  const prefix = `${relativePath.replace(/\/$/, "")}/`;
  return flattenProjectFiles(root).filter((file) => file.relativePath === relativePath || file.relativePath.startsWith(prefix));
}

function relativeProjectPath(path: string): string {
  const normalized = normalizePath(path);
  for (const prefix of ["scenes/", "raw/", "spritesheets/", "audio/", "fonts/", "scripts/", "data/", "docs/", "custom/", "packages/"]) {
    const index = normalized.indexOf(prefix);
    if (index >= 0) return normalized.slice(index);
  }
  return normalized;
}

function statusForEditorStatus(status: string): ProjectTreeNodeStatus {
  if (status === "valid") return "valid";
  if (status === "warning" || status === "missingDependency") return "warn";
  if (status === "error" || status === "invalidManifest" || status === "missingSceneFile" || status === "previewFailed") return "error";
  return "ok";
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
