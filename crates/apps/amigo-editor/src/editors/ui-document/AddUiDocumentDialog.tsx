import { useMemo, useState } from "react";
import { X } from "lucide-react";
import { OperationScopeHint } from "../../ui/feedback/OperationScopeHint";
import type { AddUiDocumentDraft, UiTemplateKind } from "./uiDocumentEditorTypes";
import {
  createDefaultAddUiDocumentDraft,
  validateAddUiDocumentDraft,
} from "./uiDocumentEditorModel";
import { UI_TEMPLATE_DEFINITIONS } from "./uiDocumentTemplates";
import { UiTemplatePreview } from "./UiTemplatePreview";

export function AddUiDocumentDialog({
  busy = false,
  initialTemplate = "empty-document",
  onClose,
  onCreate,
}: {
  busy?: boolean;
  initialTemplate?: UiTemplateKind;
  onClose: () => void;
  onCreate: (draft: AddUiDocumentDraft) => void;
}) {
  const [draft, setDraft] = useState<AddUiDocumentDraft>(() => ({
    ...createDefaultAddUiDocumentDraft(),
    template: initialTemplate,
  }));

  const selectedTemplate = useMemo(
    () => UI_TEMPLATE_DEFINITIONS.find((template) => template.kind === draft.template) ?? null,
    [draft.template],
  );

  const error = validateAddUiDocumentDraft(draft);
  const templateDisabled = selectedTemplate ? !selectedTemplate.enabled : true;

  return (
    <div className="ui-dialog-backdrop" role="presentation" onMouseDown={busy ? undefined : onClose}>
      <section
        className="ui-editor-dialog ui-editor-dialog-wide ui-add-document-dialog"
        role="dialog"
        aria-modal="true"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <header>
          <div>
            <h2>Add UI Document</h2>
            <p>Create a screen-space UiDocument in the current scene.</p>
          </div>
          <button className="icon-button" type="button" disabled={busy} onClick={onClose}>
            <X size={16} />
          </button>
        </header>

        <div className="ui-add-document-body">
          <section className="ui-template-picker">
            <h3>Choose template</h3>
            <div className="ui-template-picker-list">
              {UI_TEMPLATE_DEFINITIONS.filter(
                (template) =>
                  template.category === "document" ||
                  template.category === "menu" ||
                  template.category === "hud" ||
                  template.category === "dialogue",
              ).map((template) => (
                <button
                  key={template.kind}
                  className={`ui-template-picker-card ${draft.template === template.kind ? "selected" : ""}`}
                  disabled={busy}
                  type="button"
                  onClick={() => setDraft((current) => ({ ...current, template: template.kind }))}
                >
                  <strong>{template.label}</strong>
                  <span>{template.description}</span>
                  {!template.enabled ? <em>{template.disabledReason ?? "Coming soon"}</em> : null}
                </button>
              ))}
            </div>
          </section>

          <UiTemplatePreview template={selectedTemplate} />
        </div>

        <section className="ui-add-document-fields">
          <label className="property-field">
            <span>Name</span>
            <input
              className="property-input"
              disabled={busy}
              value={draft.name}
              onChange={(event) => setDraft((current) => ({ ...current, name: event.target.value }))}
            />
          </label>

          <label className="property-field">
            <span>Entity ID</span>
            <input
              className="property-input"
              disabled={busy}
              value={draft.entityId}
              onChange={(event) => setDraft((current) => ({ ...current, entityId: event.target.value }))}
            />
          </label>

          <label className="property-field">
            <span>Viewport</span>
            <select
              className="property-input"
              disabled={busy}
              value={`${draft.viewportWidth}x${draft.viewportHeight}`}
              onChange={(event) => {
                const [width, height] = event.target.value.split("x").map(Number);
                setDraft((current) => ({ ...current, viewportWidth: width, viewportHeight: height }));
              }}
            >
              <option value="1280x720">1280 x 720</option>
              <option value="1920x1080">1920 x 1080</option>
            </select>
          </label>
        </section>

        {error ? <p className="ui-dialog-error">{error}</p> : null}

        <footer>
          <OperationScopeHint kind="scene-document" />

          <div className="ui-dialog-actions">
            <button className="button button-ghost" type="button" disabled={busy} onClick={onClose}>
              Cancel
            </button>
            <button
              className="button button-primary"
              type="button"
              disabled={Boolean(error) || busy || templateDisabled}
              onClick={() => onCreate(draft)}
            >
              {busy ? "Creating..." : draft.template === "empty-document" ? "Create Blank UI" : "Create From Template"}
            </button>
          </div>
        </footer>
      </section>
    </div>
  );
}
