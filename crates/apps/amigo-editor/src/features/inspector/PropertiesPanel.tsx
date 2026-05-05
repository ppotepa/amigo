import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import type { EditorSelection } from "../../properties/propertiesTypes";

export function PropertiesPanel({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  return (
    <PropertiesPanelView
      details={services.details ?? null}
      selection={services.selection ?? { kind: "empty" }}
    />
  );
}

function PropertiesPanelView({
  details,
  selection,
}: {
  details: WorkspaceRuntimeServices["details"];
  selection: EditorSelection;
}) {
  const selectedScene =
    selection.kind === "scene" ? selection.scene :
    selection.kind === "entity" ? selection.scene :
    null;
  const selectedEntity = selection.kind === "entity" ? selection.entity : null;
  const selectedFile =
    selection.kind === "projectFile" ? selection.file :
    selection.kind === "asset" ? selection.file :
    null;

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
