import { useEffect } from "react";
import { FolderOpen, RefreshCcw, Settings } from "lucide-react";
import { useEditorStore } from "../app/editorStore";
import { selectedModId } from "../app/selectionSelectors";
import { openSettingsWindow, openThemeWindow, pickModsRoot, setEditorModsRoot } from "../api/editorApi";
import { ActivityFooter } from "./ActivityFooter";
import { ModInspectorPanel } from "./ModInspectorPanel";
import { ModsPanel } from "./ModsPanel";
import { ScenePreviewWorkspace } from "./ScenePreviewWorkspace";
import { ThemeButton } from "../theme/ThemeButton";
import "../styles/startup-dialog.css";

export function StartupDialog() {
  const { state, scanMods, openSelectedMod } = useEditorStore();

  useEffect(() => {
    void scanMods();
  }, [scanMods]);

  const hasBlockingTask = Object.values(state.tasks).some((task) => task.busyLevel === "blocking" && task.status === "running");

  async function handleBrowseMods() {
    const picked = await pickModsRoot();
    if (!picked) {
      return;
    }
    await setEditorModsRoot(picked);
    await scanMods();
  }

  function reportWindowOpenError(error: unknown) {
    window.alert(`Failed to open window: ${error instanceof Error ? error.message : String(error)}`);
  }

  return (
    <main className="startup-shell window-shell launcher-window-shell">
      <header className="startup-header window-titlebar">
        <div className="brand window-brand">
          <div className="brand-mark">A</div>
          <div>
            <strong>Amigo Editor</strong>
            <span>Interactive mod launcher powered by mod.toml discovery.</span>
          </div>
        </div>

        <div className="header-actions window-titlebar-actions">
          <span className="pill">mods / discovery</span>
          <ThemeButton onClick={() => void openThemeWindow().catch(reportWindowOpenError)} />
          <button className="button button-ghost" type="button" onClick={() => void openSettingsWindow().catch(reportWindowOpenError)}>
            <Settings size={16} />
            Settings
          </button>
        </div>
      </header>

      <section className="startup-main-grid">
        <ModsPanel />
        <ScenePreviewWorkspace />
        <ModInspectorPanel />
      </section>

      <footer className="startup-footer window-statusbar">
        <ActivityFooter />
        <div className="footer-actions">
          <button className="button button-ghost" type="button" onClick={() => void handleBrowseMods()}>
            <FolderOpen size={16} />
            Browse...
          </button>
          <button className="button button-ghost" type="button" onClick={() => void scanMods()}>
            <RefreshCcw size={16} />
            Rescan
          </button>
          <button className="button button-primary" type="button" disabled={!selectedModId(state.selection)} onClick={() => void openSelectedMod()}>
            <FolderOpen size={16} />
            Open Mod
          </button>
        </div>
      </footer>

      {hasBlockingTask ? (
        <div className="blocking-overlay">
          <div className="blocking-card">
            <div className="spinner" />
            <strong>Opening mod...</strong>
            <span>Preparing editor session.</span>
          </div>
        </div>
      ) : null}
    </main>
  );
}
