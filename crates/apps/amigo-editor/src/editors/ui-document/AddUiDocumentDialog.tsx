import { useState } from "react";
import { LayoutPanelTop } from "lucide-react";
import { AppDialog } from "../../ui/dialog/AppDialog";
import type { AddUiDocumentDraft, UiTemplateKind } from "./uiDocumentEditorTypes";
import { createDefaultAddUiDocumentDraft, validateAddUiDocumentDraft } from "./uiDocumentEditorModel";
import { UI_TEMPLATE_DEFINITIONS } from "./uiDocumentTemplates";

export function AddUiDocumentDialog({
  busy = false,
  initialTemplate,
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
    template: initialTemplate ?? createDefaultAddUiDocumentDraft().template,
  }));

  const error = validateAddUiDocumentDraft(draft);
  const selectedTemplate = UI_TEMPLATE_DEFINITIONS.find((template) => template.kind === draft.template);

  return (
    <AppDialog
      title="Add UI Document"
      subtitle="Create a screen-space UiDocument for menus, HUDs, overlays or dialogs."
      icon={<LayoutPanelTop size={16} />}
      toneClassName="app-dialog-tone-cyan"
      dialogClassName="app-dialog-compact"
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
            disabled={Boolean(error) || busy || !selectedTemplate?.enabled}
            onClick={() => onCreate(draft)}
          >
            {busy ? "Creating..." : "Create Document"}
          </button>
        </>
      }
    >
      <div className="dialog-form-grid">
        <label className="dialog-field">
          <span>Name</span>
          <input
            className="dialog-input"
            value={draft.name}
            disabled={busy}
            onChange={(event) => setDraft((current) => ({ ...current, name: event.target.value }))}
          />
        </label>

        <label className="dialog-field">
          <span>Entity ID</span>
          <input
            className="dialog-input"
            value={draft.entityId}
            disabled={busy}
            onChange={(event) => setDraft((current) => ({ ...current, entityId: event.target.value }))}
          />
          <small>Lowercase id used by the scene entity.</small>
        </label>

        <div className="dialog-form-grid two-col">
          <label className="dialog-field">
            <span>Viewport</span>
            <select
              className="dialog-select"
              value={`${draft.viewportWidth}x${draft.viewportHeight}`}
              disabled={busy}
              onChange={(event) => {
                const [width, height] = event.target.value.split("x").map(Number);
                setDraft((current) => ({ ...current, viewportWidth: width, viewportHeight: height }));
              }}
            >
              <option value="1280x720">1280 x 720</option>
              <option value="1920x1080">1920 x 1080</option>
            </select>
          </label>

          <label className="dialog-field">
            <span>Target</span>
            <input className="dialog-input" value="screen-space" disabled />
          </label>
        </div>

        <label className="dialog-field">
          <span>Template</span>
          <select
            className="dialog-select"
            value={draft.template}
            disabled={busy}
            onChange={(event) => setDraft((current) => ({ ...current, template: event.target.value as UiTemplateKind }))}
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
          {selectedTemplate ? <small>{selectedTemplate.description}</small> : null}
        </label>

        <p className="dialog-muted-note">
          Scene document changes use the active editor session. Use Save or Discard to commit or revert.
        </p>

        {error ? <p className="dialog-error">{error}</p> : null}
      </div>
    </AppDialog>
  );
}
