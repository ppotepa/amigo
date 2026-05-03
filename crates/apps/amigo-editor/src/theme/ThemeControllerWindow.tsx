import { closeCurrentWindow } from "../main-window/windowControls";
import { ThemeControllerContent } from "./ThemeControllerDialog";

export function ThemeControllerWindow() {
  return (
    <main className="window-route-shell standalone-window-shell">
      <ThemeControllerContent onClose={() => void closeCurrentWindow()} />
    </main>
  );
}
