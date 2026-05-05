import type { EditorModDetailsDto, EditorProjectFileDto, EditorProjectTreeDto } from "../../api/dto";
import type { ComponentToolbarState, EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { flattenProjectFiles } from "./fileTreeSelectors";

export function ScriptsBrowserPanel({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  return (
    <ScriptsBrowser
      details={services.details ?? null}
      onSelectFile={(file) => services.handleSelectProjectFile?.(file)}
      projectTree={services.projectTree}
      toolbarState={services.toolbarState}
    />
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

function SectionTitle({ title }: { title: string }) {
  return <h3 className="workspace-section-title">{title}</h3>;
}
