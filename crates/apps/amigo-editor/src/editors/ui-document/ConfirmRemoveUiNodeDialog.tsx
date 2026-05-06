import { AlertTriangle } from "lucide-react";
import { AppDialog } from "../../ui/dialog/AppDialog";

export function ConfirmRemoveUiNodeDialog({
  busy = false,
  childCount,
  nodeLabel,
  nodePath,
  onCancel,
  onConfirm,
}: {
  busy?: boolean;
  childCount: number;
  nodeLabel: string;
  nodePath: string;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  return (
    <AppDialog
      title="Remove UI Node?"
      subtitle="This operation edits the active scene document."
      icon={<AlertTriangle size={16} />}
      toneClassName="app-dialog-tone-danger"
      dialogClassName="app-dialog-narrow"
      bodyClassName="app-dialog-body-compact"
      footerClassName="app-dialog-footer-compact"
      onClose={onCancel}
      closeDisabled={busy}
      footer={
        <>
          <button className="button button-ghost" type="button" disabled={busy} onClick={onCancel}>
            Cancel
          </button>
          <button className="button button-danger" type="button" disabled={busy} onClick={onConfirm}>
            {busy ? "Removing..." : "Remove"}
          </button>
        </>
      }
    >
      <div className="dialog-form-grid">
        <p className="dialog-muted-note">
          Remove <strong>{nodeLabel}</strong>
          {childCount > 0 ? ` and ${childCount} child node${childCount === 1 ? "" : "s"}` : ""}?
        </p>

        <p className="dialog-muted-note">
          <code>{nodePath}</code>
        </p>
      </div>
    </AppDialog>
  );
}
