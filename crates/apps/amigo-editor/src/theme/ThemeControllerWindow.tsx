import { closeCurrentWindow } from "../main-window/windowControls";
import { DebugSourceOverlay, readDebugSourcePreference } from "../debug/debugSource";
import { ThemeControllerContent } from "./ThemeControllerDialog";

export function ThemeControllerWindow() {
  return (
    <DebugSourceOverlay enabled={readDebugSourcePreference()} source="src/theme/ThemeControllerWindow.tsx" className="debug-source-root-shell" contentClassName="debug-source-root-content">
      <main className="window-route-shell standalone-window-shell">
        <ThemeControllerContent onClose={() => void closeCurrentWindow()} />
      </main>
    </DebugSourceOverlay>
  );
}
