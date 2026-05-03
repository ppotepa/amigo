import { openThemeWindow } from "../api/editorApi";
import { closeCurrentWindow } from "../main-window/windowControls";
import { SettingsDialogContent } from "./SettingsDialog";

export function ModSettingsWindow({ sessionId }: { sessionId: string | null }) {
  function reportWindowOpenError(error: unknown) {
    window.alert(`Failed to open window: ${error instanceof Error ? error.message : String(error)}`);
  }

  return (
    <main className="window-route-shell standalone-window-shell" data-session-id={sessionId ?? undefined}>
      <SettingsDialogContent
        onClose={() => void closeCurrentWindow()}
        onOpenTheme={() => void openThemeWindow().catch(reportWindowOpenError)}
      />
    </main>
  );
}
