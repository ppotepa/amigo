import { useState } from "react";
import { X } from "lucide-react";
import type { AddUiNodeDraft, UiNodeCreateKind } from "./uiDocumentEditorTypes";
import {
  createDefaultAddNodeDraft,
  defaultNodeId,
  defaultNodeText,
  labelFromId,
  validateAddNodeDraft,
} from "./uiDocumentEditorModel";
import { UI_NODE_PALETTE } from "./uiDocumentTemplates";

export function AddUiNodeDialog({
  busy = false,
  initialKind = "button",
  parentPath,
  onClose,
  onCreate,
}: {
  busy?: boolean;
  initialKind?: UiNodeCreateKind;
  parentPath: string;
  onClose: () => void;
  onCreate: (draft: AddUiNodeDraft) => void;
}) {
  const [draft, setDraft] = useState<AddUiNodeDraft>(() => createDefaultAddNodeDraft(parentPath, initialKind));
  const error = validateAddNodeDraft(draft);

  function updateKind(kind: UiNodeCreateKind) {
    const id = defaultNodeId(kind);
    setDraft((current) => ({
      ...current,
      kind,
      id,
      label: labelFromId(id),
      text: defaultNodeText(kind),
    }));
  }

  return (
    <div className="ui-dialog-backdrop" role="presentation" onMouseDown={onClose}>
      <section
        className="ui-editor-dialog"
        role="dialog"
        aria-modal="true"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <header>
          <h2>Add Node</h2>
          <button className="icon-button" type="button" onClick={onClose}>
            <X size={16} />
          </button>
        </header>

        <label className="property-field">
          <span>Parent</span>
          <input className="property-input" value={draft.parentPath} readOnly />
        </label>

        <label className="property-field">
          <span>Node type</span>
          <select className="property-input" value={draft.kind} onChange={(event) => updateKind(event.target.value as UiNodeCreateKind)}>
            {UI_NODE_PALETTE.map((item) => (
              <option key={item.kind} value={item.kind} disabled={!item.enabled}>
                {item.label}
              </option>
            ))}
          </select>
        </label>

        <label className="property-field">
          <span>ID</span>
          <input
            className="property-input"
            value={draft.id}
            onChange={(event) => setDraft((current) => ({ ...current, id: event.target.value }))}
          />
        </label>

        <label className="property-field">
          <span>Label</span>
          <input
            className="property-input"
            value={draft.label}
            onChange={(event) => setDraft((current) => ({ ...current, label: event.target.value }))}
          />
        </label>

        <label className="property-field">
          <span>Text</span>
          <input
            className="property-input"
            value={draft.text}
            onChange={(event) => setDraft((current) => ({ ...current, text: event.target.value }))}
          />
        </label>

        {error ? <p className="ui-dialog-error">{error}</p> : null}

        <footer>
          <button className="button button-ghost" type="button" disabled={busy} onClick={onClose}>
            Cancel
          </button>
          <button className="button button-primary" type="button" disabled={Boolean(error) || busy} onClick={() => onCreate(draft)}>
            {busy ? "Adding..." : "Add Node"}
          </button>
        </footer>
      </section>
    </div>
  );
}
