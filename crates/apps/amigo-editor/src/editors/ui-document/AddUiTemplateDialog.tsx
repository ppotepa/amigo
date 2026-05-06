import { useState } from "react";
import { Boxes } from "lucide-react";
import { AppDialog } from "../../ui/dialog/AppDialog";
import type { AddUiTemplateDraft, UiTemplateKind } from "./uiDocumentEditorTypes";
import { createDefaultAddTemplateDraft, validateAddTemplateDraft } from "./uiDocumentEditorModel";
import { UI_TEMPLATE_DEFINITIONS } from "./uiDocumentTemplates";

export function AddUiTemplateDialog({
  busy = false,
  initialTemplate = "vertical-menu",
  parentPath,
  onClose,
  onCreate,
}: {
  busy?: boolean;
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
    <AppDialog
      title="Add Template"
      subtitle={`Insert a reusable UI subtree under ${parentPath}.`}
      icon={<Boxes size={16} />}
      toneClassName="app-dialog-tone-amber"
      dialogClassName="app-dialog-wide"
      bodyClassName="app-dialog-body-compact"
      footerClassName="app-dialog-footer-compact"
      onClose={onClose}
      closeDisabled={busy}
      footer={
        <>
          <button className="button button-ghost" type="button" disabled={busy} onClick={onClose}>
            Cancel
          </button>
          <button
            className="button button-primary"
            type="button"
            disabled={Boolean(error) || busy}
            onClick={() => onCreate(draft)}
          >
            {busy ? "Adding..." : "Add Template"}
          </button>
        </>
      }
    >
      <div className="ui-template-dialog-layout">
        <div className="dialog-card-grid two-col">
          {UI_TEMPLATE_DEFINITIONS.filter((template) => template.category !== "document").map((template) => (
            <button
              key={template.kind}
              className={`dialog-choice-card ${draft.template === template.kind ? "selected" : ""}`}
              disabled={!template.enabled || busy}
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

          <label className="dialog-field">
            <span>ID prefix</span>
            <input
              className="dialog-input"
              value={draft.idPrefix}
              disabled={busy}
              onChange={(event) => setDraft((current) => ({ ...current, idPrefix: event.target.value }))}
            />
          </label>
        </aside>
      </div>

      {error ? <p className="dialog-error">{error}</p> : null}
    </AppDialog>
  );
}
