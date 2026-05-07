import { useState } from "react";
import { Plus } from "lucide-react";
import type { EditorUiNodeDto } from "../../api/dto";
import { AppDialog } from "../../ui/dialog/AppDialog";
import type { AddUiNodeDraft, UiNodeCreateKind } from "./uiDocumentEditorTypes";
import {
  allowedUiChildrenForNode,
  firstAllowedUiChildKind,
  uiNodeCannotHaveChildrenReason,
} from "./uiNodeCapabilities";
import {
  createDefaultAddNodeDraft,
  defaultNodeId,
  defaultNodeText,
  labelFromId,
  validateAddNodeDraft,
} from "./uiDocumentEditorModel";
import { UI_NODE_PALETTE } from "./uiDocumentTemplates";

// @codemap anchor:add-ui-node-dialog domain:ui-document role:dialog priority:P1 layer:app tags:add-node,capabilities
export function AddUiNodeDialog({
  busy = false,
  initialKind,
  parentNode,
  parentPath,
  onClose,
  onCreate,
}: {
  busy?: boolean;
  initialKind?: UiNodeCreateKind;
  parentNode: EditorUiNodeDto | null;
  parentPath: string;
  onClose: () => void;
  onCreate: (draft: AddUiNodeDraft) => void;
}) {
  const allowedKinds = allowedUiChildrenForNode(parentNode);
  const resolvedInitialKind =
    initialKind && allowedKinds.includes(initialKind)
      ? initialKind
      : firstAllowedUiChildKind(parentNode) ?? "button";
  const [draft, setDraft] = useState<AddUiNodeDraft>(() =>
    createDefaultAddNodeDraft(parentPath, resolvedInitialKind),
  );
  const blockedReason = allowedKinds.length ? null : uiNodeCannotHaveChildrenReason(parentNode);
  const error = blockedReason ?? validateAddNodeDraft(draft);

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
    <AppDialog
      title="Add Node"
      subtitle={`Parent: ${parentNode?.label ?? parentPath}`}
      icon={<Plus size={16} />}
      toneClassName="app-dialog-tone-violet"
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
            disabled={Boolean(error) || busy}
            onClick={() => onCreate(draft)}
          >
            {busy ? "Adding..." : "Add Node"}
          </button>
        </>
      }
    >
      <div className="dialog-form-grid">
        <label className="dialog-field">
          <span>Node type</span>
          <select
            className="dialog-select"
            value={draft.kind}
            disabled={busy || Boolean(blockedReason)}
            onChange={(event) => updateKind(event.target.value as UiNodeCreateKind)}
          >
            {UI_NODE_PALETTE.filter((item) => allowedKinds.includes(item.kind)).map((item) => (
              <option key={item.kind} value={item.kind} disabled={!item.enabled}>
                {item.label}
              </option>
            ))}
          </select>
        </label>

        <div className="dialog-form-grid two-col">
          <label className="dialog-field">
            <span>ID</span>
              <input
                className="dialog-input"
                value={draft.id}
                disabled={busy || Boolean(blockedReason)}
                onChange={(event) => setDraft((current) => ({ ...current, id: event.target.value }))}
              />
          </label>

          <label className="dialog-field">
            <span>Label</span>
              <input
                className="dialog-input"
                value={draft.label}
                disabled={busy || Boolean(blockedReason)}
                onChange={(event) => setDraft((current) => ({ ...current, label: event.target.value }))}
              />
          </label>
        </div>

        <label className="dialog-field">
          <span>Text</span>
            <input
              className="dialog-input"
              value={draft.text}
              disabled={busy || Boolean(blockedReason)}
              placeholder="Only required for Text/Button nodes"
            onChange={(event) => setDraft((current) => ({ ...current, text: event.target.value }))}
          />
        </label>

        <p className="dialog-muted-note">
          The new node will be inserted under <code>{parentPath}</code>. Structure changes use the active editor session.
        </p>

        {error ? <p className="dialog-error">{error}</p> : null}
      </div>
    </AppDialog>
  );
}
