import { useState } from "react";
import type { EditorModDetailsDto, EditorProjectFileDto, EditorProjectTreeDto } from "../../api/dto";
import type { EditorComponentProps, ComponentToolbarState } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { ExplorerShell, type ExplorerViewMode } from "../../ui/explorer/ExplorerShell";
import { flattenProjectFiles, normalizePath } from "./fileTreeSelectors";
import { ProjectFileTree } from "./ProjectFileTree";

export function FilesBrowserPanel({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  return (
    <FilesBrowser
      details={services.details ?? null}
      loading={services.projectTreeTask?.status === "running"}
      onSelectFile={(file) => {
        if (services.openProjectFileEditor) {
          services.openProjectFileEditor(file);
          return;
        }
        services.handleSelectProjectFile?.(file);
      }}
      projectTree={services.projectTree}
      selectedFilePath={services.selectedFile?.relativePath ?? null}
      toolbarState={services.toolbarState}
    />
  );
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
  const [localViewMode, setLocalViewMode] = useState<ExplorerViewMode>("tree");

  if (!details || !projectTree) {
    return <p className="muted workspace-empty">No project files loaded.</p>;
  }

  const toolbarMode = String(toolbarState?.viewMode ?? "");
  const viewMode: ExplorerViewMode =
    toolbarMode === "flat" || toolbarMode === "list" ? "list" :
    toolbarMode === "tree" ? "tree" :
    localViewMode;
  const toolbarControlsViewMode = toolbarMode === "flat" || toolbarMode === "list" || toolbarMode === "tree";
  const fileFilter = String(toolbarState?.fileFilter ?? "all");
  const filteredRoot = filterProjectFileTree(projectTree.root, fileFilter, search);
  const flatFiles = flattenProjectFiles(projectTree.root).filter((file) =>
    fileMatchesFilesBrowserFilter(file, fileFilter) &&
    matchesSearch([file.name, file.relativePath, file.kind], search)
  );
  const visibleRoot = viewMode === "list"
    ? { ...projectTree.root, children: flatFiles }
    : filteredRoot;

  return (
    <div className="dock-scroll project-explorer-panel">
      <ExplorerShell
        allowedViewModes={toolbarControlsViewMode ? [viewMode] : ["tree", "list"]}
        className="files-browser-shell"
        loading={loading}
        loadingLabel="Indexing project files..."
        onSearchChange={setSearch}
        onViewModeChange={toolbarControlsViewMode ? undefined : setLocalViewMode}
        search={search}
        searchPlaceholder="Search files..."
        subtitle={details.id}
        title="Files"
        viewMode={viewMode}
      >
        {viewMode === "list" && flatFiles.length === 0 ? <p className="muted workspace-note">No matching files.</p> : null}
        <ProjectFileTree
          node={visibleRoot}
          selectedFilePath={selectedFilePath}
          onSelectFile={onSelectFile}
        />
      </ExplorerShell>
    </div>
  );
}

function filterProjectFileTree(root: EditorProjectFileDto, filter: string, search: string): EditorProjectFileDto {
  return {
    ...root,
    children: root.children
      .map((child) => filterProjectFileTree(child, filter, search))
      .filter((child) => {
        if (child.isDir) return child.children.length > 0;
        return fileMatchesFilesBrowserFilter(child, filter) && matchesSearch([child.name, child.relativePath, child.kind], search);
      }),
  };
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

function matchesSearch(values: string[], search: string): boolean {
  const query = search.trim().toLowerCase();
  return !query || values.some((value) => value.toLowerCase().includes(query));
}
