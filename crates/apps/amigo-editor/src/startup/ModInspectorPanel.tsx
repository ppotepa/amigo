import { ChevronDown, ChevronRight, FileSearch, FolderOpen, ShieldCheck, Trash2, AlertTriangle } from "lucide-react";
import { selectedScene as resolveSelectedScene } from "../app/store/editorSelectors";
import type { ReactNode } from "react";
import { useState } from "react";
import { useEditorStore } from "../app/editorStore";
import { DebugSourceOverlay, useDebugSourceEnabled } from "../debug/debugSource";
import { ContentSummaryGrid } from "./ContentSummaryGrid";
import { DiagnosticsList } from "./DiagnosticsList";

export function ModInspectorPanel() {
  const { state, deleteModProject, revealSelectedModFolder, revealSelectedSceneDocument, toggleInspectorSection, validateSelectedMod } = useEditorStore();
  const showDebugSources = useDebugSourceEnabled();
  const details = state.modDetails;
  const selectedScene = resolveSelectedScene(state);
  const [confirmDeleteOpen, setConfirmDeleteOpen] = useState(false);
  const [deleteBusy, setDeleteBusy] = useState(false);
  const [deleteError, setDeleteError] = useState<string | null>(null);

  if (!details) {
    return (
      <DebugSourceOverlay enabled={showDebugSources} source="src/startup/ModInspectorPanel.tsx" contentClassName="debug-source-fill">
        <aside className="panel mod-inspector">
          <div className="inspector-empty">
            <strong>No mod selected</strong>
            <span>Select a mod to view metadata.</span>
          </div>
        </aside>
      </DebugSourceOverlay>
    );
  }

  const open = state.openInspectorSections;

  return (
    <DebugSourceOverlay enabled={showDebugSources} source="src/startup/ModInspectorPanel.tsx" contentClassName="debug-source-fill">
      <aside className="panel mod-inspector">
      <div className="sticky-inspector-header">
        <div className="mod-avatar">{details.id.slice(0, 2).toUpperCase()}</div>
        <div>
          <h2>{details.name}</h2>
          <p>{details.id} · {details.version}</p>
        </div>
        <span className={`status-badge status-${details.status}`}>{details.status}</span>
      </div>

      <div className="inspector-scroll">
        <div className="inspector-actions" aria-label="Inspector actions">
          <button className="inspector-action-button" type="button" title="Validate mod" aria-label="Validate mod" onClick={() => void validateSelectedMod()}>
            <ShieldCheck size={15} />
          </button>
          <button className="inspector-action-button" type="button" title="Reveal mod folder" aria-label="Reveal mod folder" onClick={() => void revealSelectedModFolder()}>
            <FolderOpen size={15} />
          </button>
          <button className="inspector-action-button" type="button" title="Reveal scene document" aria-label="Reveal scene document" disabled={!selectedScene} onClick={() => void revealSelectedSceneDocument()}>
            <FileSearch size={15} />
          </button>
          <button className="inspector-action-button inspector-action-button-danger" type="button" title="Delete project" aria-label="Delete project" onClick={() => setConfirmDeleteOpen(true)}>
            <Trash2 size={15} />
          </button>
        </div>

        <Section id="summary" title="Mod Summary" open={open.summary} onToggle={toggleInspectorSection}>
          <dl className="kv-list">
            <dt>Mod ID</dt><dd>{details.id}</dd>
            <dt>Version</dt><dd>{details.version}</dd>
            <dt>Authors</dt><dd>{details.authors.join(", ") || "none"}</dd>
            <dt>Root</dt><dd title={details.rootPath}>{details.rootPath}</dd>
            <dt>Description</dt><dd>{details.description ?? "No description."}</dd>
          </dl>
        </Section>

        <Section id="content" title="Content Breakdown" open={open.content} onToggle={toggleInspectorSection}>
          <ContentSummaryGrid summary={details.contentSummary} />
        </Section>

        <Section id="scene" title="Selected Scene" open={open.scene} onToggle={toggleInspectorSection}>
          {selectedScene ? (
            <dl className="kv-list">
              <dt>ID</dt><dd>{selectedScene.id}</dd>
              <dt>Label</dt><dd>{selectedScene.label}</dd>
              <dt>Document</dt><dd title={selectedScene.documentPath}>{selectedScene.documentPath}</dd>
              <dt>Script</dt><dd title={selectedScene.scriptPath}>{selectedScene.scriptPath}</dd>
              <dt>Launcher</dt><dd>{selectedScene.launcherVisible ? "visible" : "hidden"}</dd>
            </dl>
          ) : <p className="muted">No scene selected.</p>}
        </Section>

        <Section id="dependencies" title="Dependencies" open={open.dependencies} onToggle={toggleInspectorSection}>
          <TagList values={details.dependencies} empty="No dependencies." />
        </Section>

        <Section id="capabilities" title="Capabilities" open={open.capabilities} onToggle={toggleInspectorSection}>
          <TagList values={details.capabilities} empty="No capabilities." />
        </Section>

        <Section id="diagnostics" title="Diagnostics" open={open.diagnostics} onToggle={toggleInspectorSection}>
          <DiagnosticsList diagnostics={details.diagnostics} />
        </Section>
      </div>
      {confirmDeleteOpen ? (
        <div className="confirm-modal-backdrop" onMouseDown={deleteBusy ? undefined : () => setConfirmDeleteOpen(false)}>
          <section className="confirm-modal" role="dialog" aria-modal="true" onMouseDown={(event) => event.stopPropagation()}>
            <header className="confirm-modal-header">
              <AlertTriangle size={16} />
              <strong>Delete project</strong>
            </header>
            <p className="confirm-modal-copy">
              This will permanently delete <code>{details.id}</code> and all its files from disk.
            </p>
            {deleteError ? <p className="confirm-modal-error">{deleteError}</p> : null}
            <footer className="confirm-modal-footer">
              <button className="button button-ghost" type="button" disabled={deleteBusy} onClick={() => setConfirmDeleteOpen(false)}>
                Cancel
              </button>
              <button
                className="button button-danger"
                type="button"
                disabled={deleteBusy}
                onClick={() => {
                  setDeleteBusy(true);
                  setDeleteError(null);
                  void deleteModProject(details.id)
                    .then(() => setConfirmDeleteOpen(false))
                    .catch((error) => setDeleteError(error instanceof Error ? error.message : String(error)))
                    .finally(() => setDeleteBusy(false));
                }}
              >
                {deleteBusy ? "Deleting..." : "Delete project"}
              </button>
            </footer>
          </section>
        </div>
      ) : null}
      </aside>
    </DebugSourceOverlay>
  );
}

function Section({ id, title, open, onToggle, children }: { id: string; title: string; open: boolean; onToggle: (id: string) => void; children: ReactNode }) {
  return (
    <section className="inspector-section">
      <button type="button" className="section-header interactive" onClick={() => onToggle(id)}>
        {open ? <ChevronDown size={15} /> : <ChevronRight size={15} />}
        {title}
      </button>
      {open ? <div className="section-body">{children}</div> : null}
    </section>
  );
}

function TagList({ values, empty }: { values: string[]; empty: string }) {
  if (values.length === 0) {
    return <p className="muted">{empty}</p>;
  }
  return (
    <div className="tag-list">
      {values.map((value) => <span key={value} className="tag">{value}</span>)}
    </div>
  );
}
