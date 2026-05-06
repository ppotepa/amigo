import { useState } from "react";
import { X } from "lucide-react";
import type { AddUiDocumentDraft, UiTemplateKind } from "./uiDocumentEditorTypes";
import { createDefaultAddUiDocumentDraft, validateAddUiDocumentDraft } from "./uiDocumentEditorModel";
import { UI_TEMPLATE_DEFINITIONS } from "./uiDocumentTemplates";

export function AddUiDocumentDialog({
  busy = false,
  onClose,
  onCreate,
}: {
  busy?: boolean;
  onClose: () => void;
  onCreate: (draft: AddUiDocumentDraft) => void;
}) {
  const [draft, setDraft] = useState<AddUiDocumentDraft>(() => createDefaultAddUiDocumentDraft());
  const error = validateAddUiDocumentDraft(draft);

  return (
    <div className="ui-dialog-backdrop" role="presentation" onMouseDown={onClose}>
      <section
        className="ui-editor-dialog"
        role="dialog"
        aria-modal="true"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <header>
          <h2>Add UI Document</h2>
          <button className="icon-button" type="button" onClick={onClose}>
            <X size={16} />
          </button>
        </header>

        <label className="property-field">
          <span>Name</span>
          <input
            className="property-input"
            value={draft.name}
            onChange={(event) => setDraft((current) => ({ ...current, name: event.target.value }))}
          />
        </label>

        <label className="property-field">
          <span>Entity ID</span>
          <input
            className="property-input"
            value={draft.entityId}
            onChange={(event) => setDraft((current) => ({ ...current, entityId: event.target.value }))}
          />
        </label>

        <label className="property-field">
          <span>Viewport</span>
          <select
            className="property-input"
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

        <label className="property-field">
          <span>Template</span>
          <select
            className="property-input"
            value={draft.template}
            onChange={(event) =>
              setDraft((current) => ({ ...current, template: event.target.value as UiTemplateKind }))
            }
          >
            {UI_TEMPLATE_DEFINITIONS.filter(
              (template) =>
                template.category === "document" ||
                template.category === "menu" ||
                template.category === "hud" ||
                template.category === "dialogue",
            ).map((template) => (
              <option key={template.kind} value={template.kind} disabled={!template.enabled}>
                {template.label}
              </option>
            ))}
          </select>
        </label>

        {error ? <p className="ui-dialog-error">{error}</p> : null}

        <footer>
          <button className="button button-ghost" type="button" disabled={busy} onClick={onClose}>
            Cancel
          </button>
          <button className="button button-primary" type="button" disabled={Boolean(error) || busy} onClick={() => onCreate(draft)}>
            {busy ? "Creating..." : "Create"}
          </button>
        </footer>
      </section>
    </div>
  );
}
