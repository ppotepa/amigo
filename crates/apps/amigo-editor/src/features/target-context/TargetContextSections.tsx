import { SelectionProperties } from "../../properties/SelectionProperties";
import type { EditorDiagnosticDto } from "../../api/dto";
import { contextKindLabelForTarget } from "../../editor-targets/editorTargetContextPresentation";
import type { ResolvedEditorTarget } from "../../editor-targets/editorTargetTypes";
import { composeSectionsForResolvedTarget } from "../../editor-targets/editorTargetSectionComposer";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { TraitSectionRenderer } from "../metadata/TraitSectionRenderer";
import { ItemContextNavigator } from "./ItemContextNavigator";

export type TargetContextSectionProps = {
  target: ResolvedEditorTarget;
  services: WorkspaceRuntimeServices;
};

// @codemap anchor:target-header-panel domain:workspace role:renderer priority:P1 layer:app tags:editor-target,item-context,header
export const TargetHeaderPanel = ({ target }: TargetContextSectionProps) => {
  const kindLabel = contextKindLabelForTarget(target);
  return (
    <section className="target-context-section item-context-header">
      <div className="item-context-title-row">
        <div>
          <p className="item-context-eyebrow">{kindLabel} Context</p>
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

export const TargetDetailsPanel = ({ target }: TargetContextSectionProps) => {
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

export const TargetProjectSummaryPanel = ({ target, services }: TargetContextSectionProps) => {
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

export const TargetFileSummaryPanel = ({ target }: TargetContextSectionProps) => {
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

export const TargetAssetSummaryPanel = ({ target }: TargetContextSectionProps) => {
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

export const TargetEntityComponentsNavigatorPanel = ({ target, services }: TargetContextSectionProps) => {
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

export const TargetTraitSummarySectionsPanel = ({ target, services }: TargetContextSectionProps) => {
  const sections = composeSectionsForResolvedTarget(target, services.metadataCatalog).filter(
    (section) => section.placement === "summary",
  );

  if (!sections.length) return null;

  return (
    <>
      {sections.map((section) => (
        <TraitSectionRenderer
          key={`${section.traitKind}:${section.id}`}
          section={section}
          services={services}
          target={target}
        />
      ))}
    </>
  );
};

export const TargetTraitDetailSectionsPanel = ({ target, services }: TargetContextSectionProps) => {
  const sections = composeSectionsForResolvedTarget(target, services.metadataCatalog).filter(
    (section) => section.placement === "details",
  );

  if (!sections.length) return null;

  return (
    <>
      {sections.map((section) => (
        <TraitSectionRenderer
          key={`${section.traitKind}:${section.id}`}
          section={section}
          services={services}
          target={target}
        />
      ))}
    </>
  );
};


export const TargetDiagnosticSummaryPanel = ({ target, services }: TargetContextSectionProps) => {
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
export const TargetQuickActionsPanel = ({ target, services }: TargetContextSectionProps) => {
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

export const TargetPropertiesPanel = ({ target, services }: TargetContextSectionProps) => {
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

export const TargetActionsPanel = ({ target, services }: TargetContextSectionProps) => {
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

export const TargetDiagnosticsPanel = ({ target, services }: TargetContextSectionProps) => {
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

export const TargetHistoryPanel = (_props: TargetContextSectionProps) => {
  return (
    <section className="target-context-section">
      <h3>History</h3>
      <p className="muted workspace-note">Target-specific history will appear here.</p>
    </section>
  );
};

export const TargetSourcePreviewPanel = ({ target, services }: TargetContextSectionProps) => {
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
