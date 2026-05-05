import { useState } from "react";
import { Search } from "lucide-react";
import type { EditorModDetailsDto, EditorProjectFileDto, EditorProjectTreeDto } from "../../api/dto";
import type { EditorComponentProps, ComponentToolbarState } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { flattenProjectFiles, normalizePath } from "./fileTreeSelectors";
import { fileIcon, ProjectFileTree } from "./ProjectFileTree";

export function FilesBrowserPanel({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
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

  const viewMode = String(toolbarState?.viewMode ?? "tree");
  const fileFilter = String(toolbarState?.fileFilter ?? "all");
  const filteredRoot = filterProjectFileTree(projectTree.root, fileFilter, search);
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
        <ProjectFileTree
          node={filteredRoot}
          depth={0}
          selectedFilePath={selectedFilePath}
          collapsed={collapsed}
          onSelectFile={onSelectFile}
          onToggle={(nodeId) => setCollapsed((current) => ({ ...current, [nodeId]: !current[nodeId] }))}
        />
      )}
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
