import { useState } from "react";
import { X } from "lucide-react";
import type { AddUiTemplateDraft, UiTemplateKind } from "./uiDocumentEditorTypes";
import { createDefaultAddTemplateDraft, validateAddTemplateDraft } from "./uiDocumentEditorModel";
import { UI_TEMPLATE_DEFINITIONS } from "./uiDocumentTemplates";

export function AddUiTemplateDialog({
  initialTemplate = "vertical-menu",
  parentPath,
  onClose,
  onCreate,
}: {
  initialTemplate?: UiTemplateKind;
  parentPath: string;
  onClose: () => void;
  onCreate: (draft: AddUiTemplateDraft) => void;
}) {
  const [draft, setDraft] = useState<AddUiTemplateDraft>(() => ({
    ...createDefaultAddTemplateDraft(parentPath),
    template: initialTemplate,
  }));

  const selectedTemplate = UI_TEMPLATE_DEFINITIONS.find((template) => template.kind === draft.template);
  const error = validateAddTemplateDraft(draft);

  return (
    <div className="ui-dialog-backdrop" role="presentation" onMouseDown={onClose}>
      <section
        className="ui-editor-dialog ui-template-dialog"
        role="dialog"
        aria-modal="true"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <header>
          <h2>Add Template</h2>
          <button className="icon-button" type="button" onClick={onClose}>
            <X size={16} />
          </button>
        </header>

        <div className="ui-template-dialog-body">
          <div className="ui-template-list">
            {UI_TEMPLATE_DEFINITIONS.filter((template) => template.category !== "document").map((template) => (
              <button
                key={template.kind}
                className={`ui-template-card ${draft.template === template.kind ? "selected" : ""}`}
                disabled={!template.enabled}
                type="button"
                onClick={() => setDraft((current) => ({ ...current, template: template.kind }))}
              >
                <strong>{template.label}</strong>
                <span>{template.description}</span>
                {!template.enabled ? <em>{template.disabledReason ?? "Coming soon"}</em> : null}
              </button>
            ))}
          </div>

          <aside className="ui-template-preview">
            <h3>Preview</h3>
            <div className="ui-template-preview-box">{selectedTemplate?.previewLabel ?? "Template"}</div>
            <p>{selectedTemplate?.description}</p>
          </aside>
        </div>

        <label className="property-field">
          <span>Parent</span>
          <input className="property-input" value={draft.parentPath} readOnly />
        </label>

        <label className="property-field">
          <span>ID prefix</span>
          <input
            className="property-input"
            value={draft.idPrefix}
            onChange={(event) => setDraft((current) => ({ ...current, idPrefix: event.target.value }))}
          />
        </label>

        {error ? <p className="ui-dialog-error">{error}</p> : null}

        <footer>
          <button className="button button-ghost" type="button" onClick={onClose}>
            Cancel
          </button>
          <button className="button button-primary" type="button" disabled={Boolean(error)} onClick={() => onCreate(draft)}>
            Add Template
          </button>
        </footer>
      </section>
    </div>
  );
}
