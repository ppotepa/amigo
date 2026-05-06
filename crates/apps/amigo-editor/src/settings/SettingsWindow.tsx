import { openThemeWindow } from "../api/editorApi";
import { DebugSourceOverlay, readDebugSourcePreference } from "../debug/debugSource";
import { closeCurrentWindow } from "../main-window/windowControls";
import { SettingsDialogContent } from "./SettingsDialog";

export function SettingsWindow() {
  function reportWindowOpenError(error: unknown) {
    window.alert(`Failed to open window: ${error instanceof Error ? error.message : String(error)}`);
  }

  return (
    <DebugSourceOverlay enabled={readDebugSourcePreference()} source="src/settings/SettingsWindow.tsx" className="debug-source-root-shell" contentClassName="debug-source-root-content">
      <main className="window-route-shell standalone-window-shell">
        <SettingsDialogContent
          onClose={() => void closeCurrentWindow()}
          onOpenTheme={() => void openThemeWindow().catch(reportWindowOpenError)}
        />
      </main>
    </DebugSourceOverlay>
  );
}
