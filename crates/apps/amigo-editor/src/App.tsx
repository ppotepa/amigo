import { EditorStoreProvider } from "./app/editorStore";
import { StartupDialog } from "./startup/StartupDialog";
import { useEditorStore } from "./app/editorStore";
import { useEffect, useState } from "react";
import { MainEditorWindow } from "./main-window/MainEditorWindow";
import { ThemeControllerWindow } from "./theme/ThemeControllerWindow";
import { SettingsWindow } from "./settings/SettingsWindow";
import { ModSettingsWindow } from "./settings/ModSettingsWindow";
import { emitWindowFocused } from "./app/windowBus";
import { closeCurrentWindow } from "./main-window/windowControls";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  closeEditorSession,
  markEditorWindowFocused,
  registerEditorWindow,
  unregisterEditorWindow,
} from "./api/editorApi";

function useHashRoute() {
  const [hash, setHash] = useState(window.location.hash || "#/startup");

  useEffect(() => {
    const handleHashChange = () => setHash(window.location.hash);
    window.addEventListener("hashchange", handleHashChange);
    return () => window.removeEventListener("hashchange", handleHashChange);
  }, []);

  return hash;
}

function normalizeWindowPath(windowName: string | null): string | null {
  switch (windowName) {
    case "startup":
      return "/startup";
    case "workspace":
      return "/workspace";
    case "theme":
      return "/theme";
    case "settings":
      return "/settings";
    case "mod-settings":
      return "/mod-settings";
    default:
      return null;
  }
}

function kindForRoute(path: string): string {
  switch (path) {
    case "/workspace":
      return "workspace";
    case "/theme":
      return "theme";
    case "/settings":
      return "settings";
    case "/mod-settings":
      return "mod-settings";
    default:
      return "startup";
  }
}

function parseWindowRoute(hash: string) {
  const searchParams = new URLSearchParams(window.location.search);
  const windowPath = normalizeWindowPath(searchParams.get("window"));
  if (windowPath) {
    return {
      path: windowPath,
      params: searchParams,
      raw: `${window.location.pathname}${window.location.search}${window.location.hash}`,
    };
  }

  const normalizedHash = hash || "#/startup";
  const withoutHash = normalizedHash.startsWith("#") ? normalizedHash.slice(1) : normalizedHash;
  const [path = "/startup", query = ""] = withoutHash.split("?");
  return {
    path,
    params: new URLSearchParams(query),
    raw: normalizedHash,
  };
}

function AppRouteBridge() {
  const { loadEditorSession, recordEvent, state } = useEditorStore();
  const hash = useHashRoute();
  const route = parseWindowRoute(hash);
  const sessionId = route.params.get("sessionId");

  useEffect(() => {
    if (route.path !== "/startup" && route.path !== "") {
      return;
    }

    recordEvent({ type: "StartupDialogOpened" });
  }, [recordEvent, route.path]);

  useEffect(() => {
    if (route.path !== "/workspace" || !sessionId || state.activeSession?.sessionId === sessionId) {
      return;
    }

    void loadEditorSession(sessionId);
  }, [loadEditorSession, route.path, sessionId, state.activeSession?.sessionId]);

  useEffect(() => {
    const label = getCurrentWindow().label;
    void registerEditorWindow(label, kindForRoute(route.path), sessionId);
    return () => {
      void unregisterEditorWindow(label);
    };
  }, [route.path, sessionId]);

  useEffect(() => {
    const label = getCurrentWindow().label;
    const handleFocus = () => {
      void markEditorWindowFocused(label);
      void emitWindowFocused(sessionId);
    };
    window.addEventListener("focus", handleFocus);
    handleFocus();
    return () => window.removeEventListener("focus", handleFocus);
  }, [sessionId]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void getCurrentWindow().onCloseRequested(async (event) => {
      if (route.path !== "/workspace" || !sessionId) {
        return;
      }
      event.preventDefault();
      if (state.hasDirtyState) {
        recordEvent({ type: "WorkspaceCloseBlocked", dirtyFileCount: Object.keys(state.dirtyFiles).length });
        const shouldClose = window.confirm("This workspace has unsaved changes. Discard changes and close?");
        if (!shouldClose) {
          return;
        }
        recordEvent({ type: "WorkspaceCloseConfirmed" });
      }
      await closeEditorSession(sessionId).catch(() => undefined);
      await closeCurrentWindow(sessionId);
    }).then((dispose) => {
      unlisten = dispose;
    });
    return () => {
      unlisten?.();
    };
  }, [recordEvent, route.path, sessionId, state.dirtyFiles, state.hasDirtyState]);

  switch (route.path) {
    case "":
    case "/startup":
      return <StartupDialog />;
    case "/workspace":
      return <MainEditorWindow />;
    case "/theme":
      return <ThemeControllerWindow />;
    case "/settings":
      return <SettingsWindow />;
    case "/mod-settings":
      return <ModSettingsWindow sessionId={sessionId} />;
    default:
      return <WindowRouteError route={route.raw} />;
  }
}

function WindowRouteError({ route }: { route: string }) {
  return (
    <main className="window-route-shell window-route-error">
      <section className="window-route-error-card">
        <h1>Unknown Amigo Editor window route</h1>
        <code>{route || "#/startup"}</code>
      </section>
    </main>
  );
}

export function App() {
  return (
    <EditorStoreProvider>
      <AppRouteBridge />
    </EditorStoreProvider>
  );
}
