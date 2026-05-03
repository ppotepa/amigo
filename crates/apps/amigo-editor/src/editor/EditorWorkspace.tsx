import { ArrowLeft, FolderOpen, PanelsTopLeft } from "lucide-react";
import { useEditorStore } from "../app/editorStore";
import { ActivityFooter } from "../startup/ActivityFooter";

export function EditorWorkspace() {
  const { state, returnToStartup } = useEditorStore();
  const session = state.activeSession;
  const mod = state.mods.find((candidate) => candidate.id === session?.modId);

  return (
    <main className="editor-shell">
      <header className="editor-topbar">
        <div className="editor-title">
          <PanelsTopLeft size={20} />
          <div>
            <h1>{mod?.name ?? session?.modId ?? "Editor Workspace"}</h1>
            <p>{session ? `${session.sessionId} · ${session.rootPath}` : "No active session"}</p>
          </div>
        </div>
        <button className="button button-ghost" type="button" onClick={returnToStartup}>
          <ArrowLeft size={16} />
          Startup
        </button>
      </header>

      <section className="editor-grid">
        <aside className="panel editor-explorer">
          <h2>Explorer</h2>
          <button className="mod-row interactive selected" type="button">
            <FolderOpen size={18} />
            <span className="mod-row-main">
              <strong>{session?.modId ?? "no-session"}</strong>
              <small>{session?.selectedSceneId ?? "no scene selected"}</small>
            </span>
            <span className="badge badge-info">session</span>
          </button>
        </aside>

        <section className="panel editor-workarea">
          <h2>Scene Workspace</h2>
          <div className="editor-canvas-placeholder">
            <strong>{session?.selectedSceneId ?? "Scene"}</strong>
            <span>Editor session is active.</span>
          </div>
        </section>

        <aside className="panel editor-inspector">
          <h2>Inspector</h2>
          <dl className="kv-list">
            <dt>Mod</dt>
            <dd>{session?.modId ?? "none"}</dd>
            <dt>Root</dt>
            <dd title={session?.rootPath}>{session?.rootPath ?? "none"}</dd>
            <dt>Created</dt>
            <dd>{session?.createdAt ?? "none"}</dd>
          </dl>
        </aside>
      </section>

      <footer className="startup-footer">
        <ActivityFooter />
      </footer>
    </main>
  );
}
