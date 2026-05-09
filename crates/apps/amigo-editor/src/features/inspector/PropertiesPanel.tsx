import type { EditorComponentProps } from "../../editor-components/componentTypes";
import { SceneContextContent } from "../scenes/context/SceneContextContent";
import { TargetContextContent } from "../target-context/TargetContextContent";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { SelectionProperties } from "../../properties/SelectionProperties";
import type { EditorSelection } from "../../properties/propertiesTypes";

// @codemap anchor:item-context-primary-host domain:workspace role:renderer priority:P1 layer:app tags:editor-target,right-dock,primary,item-context
export function ContextPanel({
  context,
  instance,
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  const target = services.currentEditorTarget ?? null;

  if (target?.ref.kind === "scene") {
    return <SceneContextContent context={context} services={services} instance={instance} />;
  }

  if (target) {
    return <TargetContextContent context={context} services={services} instance={instance} />;
  }

  return (
    <PropertiesPanelFallback
      details={services.details ?? null}
      onApplyEditorCommand={services.applyEditorCommand}
      selection={services.selection ?? { kind: "empty" }}
    />
  );
}

export const PropertiesPanel = ContextPanel;

function PropertiesPanelFallback({
  details,
  onApplyEditorCommand,
  selection,
}: {
  details: WorkspaceRuntimeServices["details"];
  onApplyEditorCommand?: WorkspaceRuntimeServices["applyEditorCommand"];
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
      {selection.kind === "uiNode" ? (
        <SelectionProperties
          context={{
            assetRegistry: null,
            assetRegistryError: null,
            details: details ?? null,
            rulesetBusy: false,
            rulesetError: null,
            onApplyEditorCommand,
          }}
          selection={selection}
        />
      ) : null}

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
