import { ChevronDown, ChevronRight, FileSearch, FolderOpen, ShieldCheck } from "lucide-react";
import type { ReactNode } from "react";
import { useEditorStore } from "../app/editorStore";
import { ContentSummaryGrid } from "./ContentSummaryGrid";
import { DiagnosticsList } from "./DiagnosticsList";

export function ModInspectorPanel() {
  const { state, revealSelectedModFolder, revealSelectedSceneDocument, toggleInspectorSection, validateSelectedMod } = useEditorStore();
  const details = state.modDetails;
  const selectedScene = details?.scenes.find((scene) => scene.id === state.selectedSceneId);

  if (!details) {
    return (
      <aside className="panel mod-inspector">
        <div className="inspector-empty">
          <strong>No mod selected</strong>
          <span>Select a mod to view metadata.</span>
        </div>
      </aside>
    );
  }

  const open = state.openInspectorSections;

  return (
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
    </aside>
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
