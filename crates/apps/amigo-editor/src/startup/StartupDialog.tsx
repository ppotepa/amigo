import { useEffect, useMemo, useState } from "react";
import { Settings } from "lucide-react";
import { useEditorStore } from "../app/editorStore";
import { openSettingsWindow, openThemeWindow } from "../api/editorApi";
import { DebugSourceOverlay, DebugSourceProvider, DebugSourceToggleButton, useDebugSourceToggle } from "../debug/debugSource";
import { ModInspectorPanel } from "./ModInspectorPanel";
import { ModsPanel } from "./ModsPanel";
import { ScenePreviewWorkspace } from "./ScenePreviewWorkspace";
import { ThemeButton } from "../theme/ThemeButton";
import { NewProjectDialog } from "./NewProjectDialog";
import "../styles/startup-dialog.css";

export function StartupDialog() {
  const { state, scanMods } = useEditorStore();
  const { showDebugSources, setShowDebugSources } = useDebugSourceToggle();
  const [newProjectOpen, setNewProjectOpen] = useState(false);

  useEffect(() => {
    void scanMods();
  }, []);

  const blockingTask = useMemo(
    () => Object.values(state.tasks).find((task) => task.busyLevel === "blocking" && task.status === "running"),
    [state.tasks],
  );
  const blockingMessage = useMemo(() => {
    if (!blockingTask) return "";
    if (blockingTask.label.toLowerCase().startsWith("creating ")) {
      return "Creating project structure and initial scene...";
    }
    if (blockingTask.label.toLowerCase().startsWith("opening ")) {
      return "Opening workspace window and loading editor session...";
    }
    return "Working...";
  }, [blockingTask]);

  function reportWindowOpenError(error: unknown) {
    window.alert(`Failed to open window: ${error instanceof Error ? error.message : String(error)}`);
  }

  return (
    <DebugSourceProvider value={showDebugSources}>
      <DebugSourceOverlay
        enabled={showDebugSources}
        source="src/startup/StartupDialog.tsx"
        className="debug-source-root-shell"
        contentClassName="debug-source-root-content"
      >
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
          <ThemeButton onClick={() => void openThemeWindow().catch(reportWindowOpenError)} />
          <DebugSourceToggleButton showDebugSources={showDebugSources} onToggle={() => setShowDebugSources((current) => !current)} />
          <button className="button button-ghost" type="button" onClick={() => void openSettingsWindow().catch(reportWindowOpenError)}>
            <Settings size={16} />
            Settings
          </button>
        </div>
      </header>

      <section className="startup-main-grid">
        <ModsPanel onNewProject={() => setNewProjectOpen(true)} onRescan={() => void scanMods()} />
        <ScenePreviewWorkspace />
        <ModInspectorPanel />
      </section>

      {newProjectOpen ? <NewProjectDialog onClose={() => setNewProjectOpen(false)} /> : null}
      {blockingTask ? (
        <div className="blocking-overlay">
          <div className="blocking-card">
            <div className="spinner" />
            <strong>{blockingTask.label}</strong>
            <span>{blockingMessage}</span>
          </div>
        </div>
      ) : null}
        </main>
      </DebugSourceOverlay>
    </DebugSourceProvider>
  );
}
