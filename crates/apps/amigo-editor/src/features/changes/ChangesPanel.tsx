import { RotateCcw, RotateCw, Save, Trash2 } from "lucide-react";
import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import "./changes.css";

export function ChangesPanel({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  const session = services.editorModeSession;
  const history = session?.history;

  if (!session || !history) {
    return (
      <section className="workspace-section">
        <h3>Changes</h3>
        <p className="muted workspace-note">No active editor session.</p>
      </section>
    );
  }

  return (
    <div className="dock-scroll">
      <section className="workspace-section">
        <h3>Document</h3>
        <div className="workspace-row">
          <span className="dock-icon dock-icon-blue">Y</span>
          <span>
            <strong>{session.sceneId}</strong>
            <small>
              Revision {history.revision} / Saved {history.savedRevision}
            </small>
          </span>
          <em className={`badge ${history.dirty ? "badge-warning" : "badge-valid"}`}>
            {history.dirty ? "unsaved" : "saved"}
          </em>
        </div>
      </section>

      <section className="workspace-section">
        <h3>Actions</h3>
        <div className="property-actions changes-actions">
          <button
            className="button button-ghost"
            type="button"
            disabled={!session.canUndo}
            onClick={() => void services.undoEditorModeTransaction?.()}
          >
            <RotateCcw size={14} />
            Undo
          </button>
          <button
            className="button button-ghost"
            type="button"
            disabled={!session.canRedo}
            onClick={() => void services.redoEditorModeTransaction?.()}
          >
            <RotateCw size={14} />
            Redo
          </button>
          <button
            className="button button-primary"
            type="button"
            disabled={!history.dirty}
            onClick={() => void services.saveEditorModeSession?.()}
          >
            <Save size={14} />
            Save
          </button>
          <button
            className="button button-ghost"
            type="button"
            disabled={!history.dirty}
            onClick={() => void services.discardEditorModeSessionChanges?.()}
          >
            <Trash2 size={14} />
            Discard
          </button>
        </div>
      </section>

      <section className="workspace-section">
        <h3>Unsaved Changes ({history.undoCount})</h3>
        {history.entries.length ? (
          <div className="changes-list">
            {history.entries.map((entry) => (
              <article key={entry.id} className="changes-entry">
                <strong>{entry.label}</strong>
                <span title={entry.target}>{entry.target}</span>
                <em>
                  {entry.kind} / r{entry.revision}
                </em>
              </article>
            ))}
          </div>
        ) : (
          <p className="muted workspace-note">No unsaved changes.</p>
        )}
      </section>

      <section className="workspace-section">
        <h3>Redo Stack ({history.redoCount})</h3>
        {history.redoEntries.length ? (
          <div className="changes-list">
            {history.redoEntries.map((entry) => (
              <article key={entry.id} className="changes-entry muted">
                <strong>{entry.label}</strong>
                <span title={entry.target}>{entry.target}</span>
                <em>
                  {entry.kind} / r{entry.revision}
                </em>
              </article>
            ))}
          </div>
        ) : (
          <p className="muted workspace-note">Redo stack is empty.</p>
        )}
      </section>
    </div>
  );
}
