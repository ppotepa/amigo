import { SelectionProperties } from "../../properties/SelectionProperties";
import type { EditorDiagnosticDto } from "../../api/dto";
import type { TargetPanelComponent } from "../../editor-targets/editorTargetContextTypes";
import type { ResolvedEditorTarget } from "../../editor-targets/editorTargetTypes";
import { GenericPropertiesPanel } from "../metadata/GenericPropertiesPanel";
import { ItemContextNavigator } from "./ItemContextNavigator";

// @codemap anchor:target-header-panel domain:workspace role:renderer priority:P1 layer:app tags:editor-target,item-context,header
export const TargetHeaderPanel: TargetPanelComponent = ({ target }) => {
  return (
    <section className="target-context-section item-context-header">
      <div className="item-context-title-row">
        <div>
          <p className="item-context-eyebrow">Item Context</p>
          <h3>{target.descriptor.label}</h3>
        </div>
        <span className={`badge ${target.status === "resolved" ? "badge-valid" : "badge-muted"}`}>
          {target.status}
        </span>
      </div>

      <div className="item-context-badges">
        <span className="badge badge-muted">{target.descriptor.kind}</span>
        <span className="badge badge-muted">{target.descriptor.selectionKind}</span>
      </div>

      {target.descriptor.subtitle ? (
        <p className="muted workspace-note" title={target.descriptor.subtitle}>
          {target.descriptor.subtitle}
        </p>
      ) : null}

      {target.descriptor.breadcrumbs.length ? (
        <p className="item-context-breadcrumbs">
          {target.descriptor.breadcrumbs.join(" / ")}
        </p>
      ) : null}

      {target.reason ? <p className="muted workspace-note">{target.reason}</p> : null}
    </section>
  );
};

export const TargetDetailsPanel: TargetPanelComponent = ({ target }) => {
  return (
    <section className="target-context-section">
      <h3>Details</h3>
      <dl className="kv-list">
        <dt>Kind</dt>
        <dd>{target.descriptor.kind}</dd>
        <dt>Status</dt>
        <dd>{target.status}</dd>
        <dt>Selection</dt>
        <dd>{target.descriptor.selectionKind}</dd>
        <dt>Path</dt>
        <dd>{target.descriptor.subtitle ?? "none"}</dd>
      </dl>
    </section>
  );
};

export const TargetProjectSummaryPanel: TargetPanelComponent = ({ target, services }) => {
  const details = services.details ?? null;
  return (
    <section className="target-context-section item-context-summary">
      <h3>Project Summary</h3>
      <dl className="kv-list">
        <dt>Name</dt>
        <dd>{details?.name ?? target.descriptor.label}</dd>
        <dt>Mod ID</dt>
        <dd>{details?.id ?? target.descriptor.label}</dd>
        <dt>Root</dt>
        <dd title={details?.rootPath}>{details?.rootPath ?? "none"}</dd>
        <dt>Authors</dt>
        <dd>{details?.authors.join(", ") || "none"}</dd>
        <dt>Scenes</dt>
        <dd>{details?.scenes.length ?? 0}</dd>
        <dt>Diagnostics</dt>
        <dd>{services.allProblems?.length ?? 0}</dd>
      </dl>
    </section>
  );
};

export const TargetFileSummaryPanel: TargetPanelComponent = ({ target }) => {
  const selection = target.selection;
  const file = selection.kind === "projectFile" ? selection.file : null;

  return (
    <section className="target-context-section item-context-summary">
      <h3>{target.ref.kind === "script" ? "Script Summary" : "File Summary"}</h3>
      <dl className="kv-list">
        <dt>Name</dt>
        <dd>{file?.name ?? target.descriptor.label}</dd>
        <dt>Relative Path</dt>
        <dd>{file?.relativePath ?? target.descriptor.subtitle ?? "none"}</dd>
        <dt>Kind</dt>
        <dd>{file?.kind ?? target.ref.kind}</dd>
        <dt>Directory</dt>
        <dd>{file?.isDir ? "yes" : "no"}</dd>
        <dt>Status</dt>
        <dd>{target.status}</dd>
      </dl>
    </section>
  );
};

export const TargetAssetSummaryPanel: TargetPanelComponent = ({ target }) => {
  const selection = target.selection;
  const asset = selection.kind === "asset" ? selection.asset : null;
  const file = selection.kind === "asset" ? selection.file : null;

  return (
    <section className="target-context-section item-context-summary">
      <h3>Asset Summary</h3>
      <dl className="kv-list">
        <dt>Label</dt>
        <dd>{asset?.label ?? target.descriptor.label}</dd>
        <dt>Asset Key</dt>
        <dd>{asset?.assetKey ?? (target.ref.kind === "asset" ? target.ref.assetKey : "none")}</dd>
        <dt>Kind</dt>
        <dd>{asset?.kind ?? "unknown"}</dd>
        <dt>Domain</dt>
        <dd>{asset?.domain ?? "none"}</dd>
        <dt>Descriptor</dt>
        <dd>{asset?.descriptorRelativePath ?? "none"}</dd>
        <dt>File</dt>
        <dd>{file?.relativePath ?? "none"}</dd>
      </dl>
    </section>
  );
};

export const TargetSceneSummaryPanel: TargetPanelComponent = ({ target, services }) => {
  const selection = target.selection;
  const scene = selection.kind === "scene" ? selection.scene : services.selectedScene ?? null;

  return (
    <section className="target-context-section item-context-summary">
      <h3>Scene Summary</h3>
      <dl className="kv-list">
        <dt>Label</dt>
        <dd>{scene?.label ?? target.descriptor.label}</dd>
        <dt>Scene ID</dt>
        <dd>{scene?.id ?? (target.ref.kind === "scene" ? target.ref.sceneId : "none")}</dd>
        <dt>YAML</dt>
        <dd>{scene?.documentPath ?? "none"}</dd>
        <dt>Script</dt>
        <dd>{scene?.scriptPath ?? "none"}</dd>
        <dt>Launcher Visible</dt>
        <dd>{scene ? (scene.launcherVisible ? "yes" : "no") : "none"}</dd>
        <dt>Entities</dt>
        <dd>{services.hierarchy?.entities.length ?? 0}</dd>
      </dl>
    </section>
  );
};

export const TargetEntitySummaryPanel: TargetPanelComponent = ({ target }) => {
  const selection = target.selection;
  const entity = selection.kind === "entity" ? selection.entity : selection.kind === "component" ? selection.entity : null;

  return (
    <section className="target-context-section item-context-summary">
      <h3>Entity Summary</h3>
      <dl className="kv-list">
        <dt>Name</dt>
        <dd>{entity?.name ?? target.descriptor.label}</dd>
        <dt>Entity ID</dt>
        <dd>{entity?.id ?? (target.ref.kind === "sceneEntity" || target.ref.kind === "component" ? target.ref.entityId : "none")}</dd>
        <dt>Visible</dt>
        <dd>{entity ? (entity.visible ? "yes" : "no") : "none"}</dd>
        <dt>Simulation</dt>
        <dd>{entity ? (entity.simulationEnabled ? "enabled" : "disabled") : "none"}</dd>
        <dt>Components</dt>
        <dd>{entity?.componentCount ?? 0}</dd>
        <dt>Tags</dt>
        <dd>{entity?.tags.join(", ") || "none"}</dd>
        <dt>Groups</dt>
        <dd>{entity?.groups.join(", ") || "none"}</dd>
      </dl>
    </section>
  );
};

export const TargetComponentSummaryPanel: TargetPanelComponent = ({ target }) => {
  const selection = target.selection;
  const component = selection.kind === "component" ? selection.component : null;

  if (!component) return null;

  return (
    <section className="target-context-section item-context-summary">
      <h3>Component Summary</h3>
      <dl className="kv-list">
        <dt>Label</dt>
        <dd>{component.label || component.typeName}</dd>
        <dt>Type</dt>
        <dd>{component.typeName}</dd>
        <dt>Index</dt>
        <dd>#{component.componentIndex}</dd>
        <dt>YAML Path</dt>
        <dd>{component.yamlPath}</dd>
        <dt>Descriptor</dt>
        <dd>{component.descriptorKind ?? "none"}</dd>
        <dt>Properties</dt>
        <dd>{component.properties.length}</dd>
        <dt>Asset Refs</dt>
        <dd>{component.assetRefs.length}</dd>
      </dl>
    </section>
  );
};

export const TargetEntityComponentsNavigatorPanel: TargetPanelComponent = ({ target, services }) => {
  const selection = target.selection;
  const entity = selection.kind === "entity" ? selection.entity : selection.kind === "component" ? selection.entity : null;

  if (!entity) return null;
  if (target.ref.kind !== "sceneEntity" && target.ref.kind !== "component") return null;

  return (
    <section className="target-context-section item-context-components">
      <ItemContextNavigator
        componentTypes={entity.componentTypes}
        components={entity.components ?? []}
        metadata={services.metadataCatalog ?? null}
        onActivateTarget={(targetRef, intent = "select") => {
          services.activateEditorTarget?.(targetRef, intent);
        }}
        target={target.ref}
      />
    </section>
  );
};

export const TargetComponentInstancesPanel: TargetPanelComponent = ({ target, services }) => {
  const selection = target.selection;

  if (selection.kind === "component") {
    return (
      <section className="target-context-section">
        <h3>Component Properties</h3>
        <GenericPropertiesPanel
          component={selection.component}
          metadata={services.metadataCatalog ?? null}
        />
      </section>
    );
  }

  const entity = selection.kind === "entity" ? selection.entity : null;

  if (!entity) {
    return <TargetDetailsPanel target={target} services={services} />;
  }

  return (
    <section className="target-context-section">
      <h3>Component Properties</h3>
      <GenericPropertiesPanel
        componentTypes={entity.componentTypes}
        components={entity.components ?? []}
        metadata={services.metadataCatalog ?? null}
      />
    </section>
  );
};

export const TargetUiSummaryPanel: TargetPanelComponent = ({ target, services }) => {
  const ref = target.ref;
  const document =
    (ref.kind === "uiDocument" || ref.kind === "uiNode")
      ? services.hierarchy?.uiDocuments.find(
          (candidate) =>
            candidate.entityId === ref.entityId &&
            candidate.componentIndex === ref.componentIndex,
        ) ?? null
      : null;
  const selection = target.selection;
  const node = selection.kind === "uiNode" ? selection.node : null;

  return (
    <section className="target-context-section item-context-summary">
      <h3>{ref.kind === "uiNode" ? "UI Node Summary" : "UI Document Summary"}</h3>
      <dl className="kv-list">
        <dt>Entity</dt>
        <dd>{document?.entityName ?? (ref.kind === "uiDocument" || ref.kind === "uiNode" ? ref.entityId : "none")}</dd>
        <dt>Component</dt>
        <dd>{ref.kind === "uiDocument" || ref.kind === "uiNode" ? ref.componentIndex : "none"}</dd>
        <dt>Node</dt>
        <dd>{node?.label ?? target.descriptor.label}</dd>
        <dt>Kind</dt>
        <dd>{node?.kind ?? "document"}</dd>
        <dt>Path</dt>
        <dd>{node?.path ?? (ref.kind === "uiNode" ? ref.nodePath : document?.root.path ?? "none")}</dd>
        <dt>Children</dt>
        <dd>{node?.childCount ?? document?.root.childCount ?? 0}</dd>
        <dt>Text</dt>
        <dd>{node?.text ?? "none"}</dd>
        <dt>Action</dt>
        <dd>{node?.actionEvent ?? "none"}</dd>
      </dl>
    </section>
  );
};

export const TargetDiagnosticSummaryPanel: TargetPanelComponent = ({ target, services }) => {
  const diagnostic = diagnosticForTarget(target, services.allProblems ?? []);
  return (
    <section className="target-context-section item-context-summary">
      <h3>Diagnostic Summary</h3>
      <dl className="kv-list">
        <dt>Level</dt>
        <dd>{diagnostic?.level ?? "unknown"}</dd>
        <dt>Code</dt>
        <dd>{diagnostic?.code ?? target.descriptor.label}</dd>
        <dt>Message</dt>
        <dd>{diagnostic?.message ?? target.descriptor.subtitle ?? "none"}</dd>
        <dt>Path</dt>
        <dd>{diagnostic?.path ?? "none"}</dd>
      </dl>
    </section>
  );
};

// @codemap anchor:target-quick-actions-panel domain:workspace role:renderer priority:P1 layer:app tags:editor-target,item-context,actions
export const TargetQuickActionsPanel: TargetPanelComponent = ({ target, services }) => {
  const actions = target.descriptor.actions
    .filter((action) => action.visible !== false && action.enabled !== false)
    .slice(0, 4);

  if (actions.length === 0) {
    return null;
  }

  return (
    <section className="target-context-section item-context-quick-actions">
      <h3>Quick Actions</h3>
      <div className="target-context-actions">
        {actions.map((action) => (
          <button
            key={action.id}
            type="button"
            title={action.title}
            onClick={() => services.activateEditorTarget?.(target.ref, action.intent ?? "open")}
          >
            {action.label}
          </button>
        ))}
      </div>
    </section>
  );
};

export const TargetPropertiesPanel: TargetPanelComponent = ({ target, services }) => {
  if (target.selection.kind !== "uiNode") {
    return <TargetDetailsPanel target={target} services={services} />;
  }

  return (
    <section className="target-context-section">
      <h3>Properties</h3>
      <SelectionProperties
        context={{
          assetRegistry: services.assetRegistry ?? null,
          assetRegistryError: null,
          details: services.details ?? null,
          rulesetBusy: false,
          rulesetError: null,
          onApplyEditorCommand: services.applyEditorCommand,
        }}
        selection={target.selection}
      />
    </section>
  );
};

export const TargetActionsPanel: TargetPanelComponent = ({ target, services }) => {
  const actions = target.descriptor.actions.filter((action) => action.visible !== false);

  return (
    <section className="target-context-section">
      <h3>Actions</h3>
      {actions.length ? (
        <div className="target-context-actions">
          {actions.map((action) => (
            <button
              key={action.id}
              type="button"
              disabled={action.enabled === false}
              title={action.title}
              onClick={() => services.activateEditorTarget?.(target.ref, action.intent ?? "open")}
            >
              {action.label}
            </button>
          ))}
        </div>
      ) : (
        <p className="muted workspace-note">No actions for this target.</p>
      )}
    </section>
  );
};

export const TargetDiagnosticsPanel: TargetPanelComponent = ({ target, services }) => {
  const diagnostics = diagnosticsForTarget(target, services.allProblems ?? []);

  return (
    <section className="target-context-section">
      <h3>Diagnostics</h3>
      {diagnostics.length ? (
        <div className="target-context-list">
          {diagnostics.slice(0, 6).map((diagnostic, index) => (
            <div key={`${diagnostic.code}:${index}`} className="workspace-row">
              <span className={`badge diagnostic-${diagnostic.level}`}>{diagnostic.level}</span>
              <span>
                <strong>{diagnostic.code}</strong>
                <small>{diagnostic.message}</small>
              </span>
            </div>
          ))}
        </div>
      ) : (
        <p className="muted workspace-note">No diagnostics for this target.</p>
      )}
    </section>
  );
};

export const TargetHistoryPanel: TargetPanelComponent = () => {
  return (
    <section className="target-context-section">
      <h3>History</h3>
      <p className="muted workspace-note">Target-specific history will appear here.</p>
    </section>
  );
};

export const TargetSourcePreviewPanel: TargetPanelComponent = ({ target, services }) => {
  const file =
    target.selection.kind === "projectFile" ? target.selection.file :
    target.selection.kind === "asset" ? target.selection.file :
    null;

  return (
    <section className="target-context-section">
      <h3>Source Preview</h3>
      <dl className="kv-list">
        <dt>File</dt>
        <dd>{file?.relativePath ?? target.descriptor.subtitle ?? "none"}</dd>
        <dt>Loaded</dt>
        <dd>{services.selectedFileContent ? "yes" : "no"}</dd>
      </dl>
    </section>
  );
};

// @codemap anchor:target-summary-panels domain:workspace role:renderer priority:P1 layer:app tags:editor-target,item-context,summary
function diagnosticsForTarget(
  target: ResolvedEditorTarget,
  diagnostics: EditorDiagnosticDto[],
): EditorDiagnosticDto[] {
  const subtitle = target.descriptor.subtitle ?? "";
  const breadcrumbs = target.descriptor.breadcrumbs.join("/");
  return diagnostics.filter((diagnostic) => {
    const path = diagnostic.path ?? "";
    return (
      Boolean(path && subtitle.includes(path)) ||
      Boolean(path && breadcrumbs.includes(path)) ||
      diagnostic.message.includes(target.descriptor.label) ||
      diagnostic.code === target.descriptor.label
    );
  });
}

function diagnosticForTarget(
  target: ResolvedEditorTarget,
  diagnostics: EditorDiagnosticDto[],
): EditorDiagnosticDto | null {
  if (target.ref.kind !== "diagnostic") return null;
  return diagnosticsForTarget(target, diagnostics)[0] ?? null;
}
