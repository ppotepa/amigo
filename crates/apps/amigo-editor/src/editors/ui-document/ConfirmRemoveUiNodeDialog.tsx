import { X } from "lucide-react";

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
    <div className="ui-dialog-backdrop" role="presentation" onMouseDown={onCancel}>
      <section
        className="ui-editor-dialog"
        role="dialog"
        aria-modal="true"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <header>
          <h2>Remove UI Node?</h2>
          <button className="icon-button" type="button" disabled={busy} onClick={onCancel}>
            <X size={16} />
          </button>
        </header>

        <p className="workspace-note">
          Remove <strong>{nodeLabel}</strong>
          {childCount > 0 ? ` and ${childCount} child node${childCount === 1 ? "" : "s"}` : ""}?
        </p>
        <p className="muted workspace-note">
          <code>{nodePath}</code>
        </p>

        <footer>
          <button className="button button-ghost" type="button" disabled={busy} onClick={onCancel}>
            Cancel
          </button>
          <button className="button button-danger" type="button" disabled={busy} onClick={onConfirm}>
            {busy ? "Removing..." : "Remove"}
          </button>
        </footer>
      </section>
    </div>
  );
}
