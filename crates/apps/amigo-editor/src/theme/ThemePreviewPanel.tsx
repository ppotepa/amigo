import { AlertTriangle, Folder, Play, RefreshCcw, Rocket, Settings } from "lucide-react";
import type { ThemeId } from "./themeTypes";

export function ThemePreviewPanel({ themeId }: { themeId: ThemeId }) {
  return (
    <section className="theme-preview-panel" data-theme={themeId}>
      <div className="preview-window">
        <header className="preview-header">
          <strong>Theme Preview</strong>
          <span className="badge badge-info">border / glow / selection</span>
        </header>

        <main className="theme-sample">
          <div className="theme-sample-actions">
            <button className="button button-primary" type="button">
              <Play size={13} />
              Play
            </button>
            <button className="button button-secondary" type="button">
              <Rocket size={13} />
              Launch
            </button>
            <button className="button button-ghost" type="button">
              <RefreshCcw size={13} />
              Refresh
            </button>
            <button className="icon-button" type="button" aria-label="Settings">
              <Settings size={14} />
            </button>
          </div>

          <div className="theme-sample-row selected">
            <Folder size={16} />
            <span>
              <strong>Selected Scene</strong>
              <small>scene · 18 entities · 4 scripts</small>
            </span>
            <em className="badge badge-valid">valid</em>
          </div>

          <div className="theme-sample-row">
            <Folder size={16} />
            <span>
              <strong>Texture Asset</strong>
              <small>asset · referenced by current scene</small>
            </span>
            <em className="badge badge-info">used</em>
          </div>

          <div className="theme-sample-badges">
            <span className="badge badge-valid">valid</span>
            <span className="badge badge-warning">warning</span>
            <span className="badge badge-error">error</span>
            <span className="badge badge-info">cached</span>
          </div>

          <div className="diagnostic diagnostic-warning">
            <AlertTriangle size={14} />
            <span>Theme feature tokens affect border, glow, input and selection states.</span>
          </div>
        </main>
      </div>
    </section>
  );
}
