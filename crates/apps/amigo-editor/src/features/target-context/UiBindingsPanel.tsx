import type { TargetContextSectionProps } from "./TargetContextSections";
import {
  bindingsForUiDocument,
  bindingsForUiNode,
  collectUiBindingsFromServices,
  currentUiDocumentForTarget,
  currentUiNodePathForTarget,
  uiBindingKindLabel,
  uiBindingStatusLabel,
  type UiBindingWithStatus,
} from "./uiBindingsModel";

// @codemap anchor:ui-bindings-panel domain:ui-document role:renderer priority:P1 layer:app tags:bindings,ui-document,right-dock
export const UiBindingsPanel = ({ target, services }: TargetContextSectionProps) => {
  if (target.ref.kind !== "uiDocument" && target.ref.kind !== "uiNode") {
    return null;
  }

  const document = currentUiDocumentForTarget(target, services);
  const bindings = collectUiBindingsFromServices(services, document);
  const nodePath = currentUiNodePathForTarget(target);

  const entries =
    target.ref.kind === "uiNode"
      ? bindingsForUiNode(bindings, document, nodePath)
      : bindingsForUiDocument(bindings, document);

  const title =
    target.ref.kind === "uiNode"
      ? `Bindings${nodePath ? ` for ${nodePath}` : ""}`
      : "UI Document Bindings";

  return (
    <section className="target-context-section ui-bindings-panel">
      <h3>{title}</h3>

      <dl className="kv-list ui-bindings-summary">
        <dt>Source</dt>
        <dd>{services.selectedFileContent?.relativePath ?? "no YAML source loaded"}</dd>
        <dt>Total</dt>
        <dd>{bindings.length}</dd>
        <dt>Shown</dt>
        <dd>{entries.length}</dd>
      </dl>

      {!services.selectedFileContent?.content ? (
        <p className="muted workspace-note">
          UiModelBindings are not available because no scene YAML content is currently loaded.
        </p>
      ) : null}

      {entries.length ? (
        <div className="target-context-list ui-bindings-list">
          {entries.map((entry) => (
            <UiBindingRow key={entry.binding.id} entry={entry} />
          ))}
        </div>
      ) : (
        <p className="muted workspace-note">
          {bindings.length
            ? "No bindings target the current UI selection."
            : "No UiModelBindings found in the loaded YAML source."}
        </p>
      )}

      <p className="muted workspace-note">
        Binding editing is read-only in this batch. Add/Edit/Delete commands come next.
      </p>
    </section>
  );
};

function UiBindingRow({ entry }: { entry: UiBindingWithStatus }) {
  return (
    <div className={`workspace-row ui-binding-row ui-binding-row-${entry.status}`}>
      <span className={`badge ${entry.status === "resolved" ? "badge-valid" : "badge-muted"}`}>
        {uiBindingStatusLabel(entry.status)}
      </span>
      <span>
        <strong>{entry.binding.state}</strong>
        <small>{entry.binding.path}</small>
      </span>
      <em className="badge badge-muted">{uiBindingKindLabel(entry.binding.kind)}</em>
    </div>
  );
}
