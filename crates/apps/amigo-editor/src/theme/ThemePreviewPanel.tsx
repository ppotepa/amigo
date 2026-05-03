import { AlertTriangle, Folder, Play, RefreshCcw } from "lucide-react";
import type { ThemeId } from "./themeTypes";

export function ThemePreviewPanel({ themeId }: { themeId: ThemeId }) {
  return (
    <section className="theme-preview-panel" data-theme={themeId}>
      <div className="preview-window">
        <header className="preview-header">
          <strong>Fresh Minimal</strong>
          <span className="badge badge-info">preview</span>
        </header>

        <main className="theme-sample">
          <div className="theme-sample-actions">
            <button className="button button-primary" type="button">
              <Play size={13} />
              Primary
            </button>
            <button className="button button-secondary" type="button">
              <RefreshCcw size={13} />
              Secondary
            </button>
            <button className="button button-ghost" type="button">
              Ghost
            </button>
          </div>

          <div className="theme-sample-row selected">
            <Folder size={16} />
            <span>
              <strong>Core Runtime</strong>
              <small>2 scenes · 18 files</small>
            </span>
            <em className="badge badge-valid">valid</em>
          </div>

          <div className="theme-sample-badges">
            <span className="badge badge-valid">valid</span>
            <span className="badge badge-warning">warning</span>
            <span className="badge badge-error">error</span>
            <span className="badge badge-info">cached</span>
          </div>

          <div className="diagnostic diagnostic-warning">
            <AlertTriangle size={14} />
            <span>Missing scene preview cache</span>
          </div>
        </main>
      </div>
    </section>
  );
}
