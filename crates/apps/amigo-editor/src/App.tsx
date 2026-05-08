import { EditorStoreProvider } from "./app/editorStore";
import { StartupDialog } from "./startup/StartupDialog";
import { AppSplash } from "./startup/AppSplash";
import { useEditorStore } from "./app/editorStore";
import { useEffect, useState } from "react";
import { MainEditorWindow } from "./main-window/MainEditorWindow";
import type { DetachedWorkspaceSurfaceRequest } from "./main-window/MainEditorWindow";
import { ThemeControllerWindow } from "./theme/ThemeControllerWindow";
import { SettingsWindow } from "./settings/SettingsWindow";
import { ModSettingsWindow } from "./settings/ModSettingsWindow";
import { emitWindowFocused } from "./app/windowBus";
import { closeCurrentWindow } from "./main-window/windowControls";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { DebugSourceOverlay, readDebugSourcePreference } from "./debug/debugSource";
import type { EditorSerializedComponentContext } from "./editor-components/componentTypes";
import {
  closeEditorSession,
  getLaunchFlags,
  markEditorWindowFocused,
  registerEditorWindow,
  unregisterEditorWindow,
} from "./api/editorApi";

function hasNoSplashUrlFlag(): boolean {
  const searchParams = new URLSearchParams(window.location.search);
  if (searchParams.has("noSplash") || searchParams.get("splash") === "0") {
    return true;
  }

  const hashQuery = window.location.hash.split("?")[1] ?? "";
  const hashParams = new URLSearchParams(hashQuery);
  return hashParams.has("noSplash") || hashParams.get("splash") === "0";
}

function isStartupRouteForSplash(): boolean {
  const searchParams = new URLSearchParams(window.location.search);
  const windowPath = normalizeWindowPath(searchParams.get("window"));
  if (windowPath) {
    return windowPath === "/startup";
  }

  const hash = window.location.hash || "#/startup";
  const withoutHash = hash.startsWith("#") ? hash.slice(1) : hash;
  const [path = "/startup"] = withoutHash.split("?");
  return path === "" || path === "/startup";
}

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
  const workspaceId = route.params.get("workspaceId") ?? "main";
  const detachedSurface = detachedSurfaceFromRoute(route.params);

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
      const isDetachedWorkspace = workspaceId !== "main";
      if (!isDetachedWorkspace && state.hasDirtyState) {
        recordEvent({ type: "WorkspaceCloseBlocked", dirtyFileCount: Object.keys(state.dirtyFiles).length });
        const shouldClose = window.confirm("This workspace has unsaved changes. Discard changes and close?");
        if (!shouldClose) {
          return;
        }
        recordEvent({ type: "WorkspaceCloseConfirmed" });
      }
      if (!isDetachedWorkspace) {
        await closeEditorSession(sessionId).catch(() => undefined);
        await closeCurrentWindow(sessionId);
        return;
      }

      await closeCurrentWindow();
    }).then((dispose) => {
      unlisten = dispose;
    });
    return () => {
      unlisten?.();
    };
  }, [recordEvent, route.path, sessionId, state.dirtyFiles, state.hasDirtyState, workspaceId]);

  switch (route.path) {
    case "":
    case "/startup":
      return <StartupDialog />;
    case "/workspace":
      return <MainEditorWindow detachedSurface={detachedSurface} workspaceId={workspaceId} />;
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

function detachedSurfaceFromRoute(params: URLSearchParams): DetachedWorkspaceSurfaceRequest | null {
  const componentId = params.get("componentId");
  if (!componentId) {
    return null;
  }

  return {
    componentId,
    context: parseRouteContext(params.get("context")),
    filePath: params.get("filePath") ?? undefined,
    resourceUri: params.get("resourceUri") ?? undefined,
    titleOverride: params.get("titleOverride") ?? params.get("title") ?? undefined,
  };
}

function parseRouteContext(value: string | null): EditorSerializedComponentContext | undefined {
  if (!value) {
    return undefined;
  }

  try {
    const parsed = JSON.parse(value);
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      return undefined;
    }

    return Object.fromEntries(
      Object.entries(parsed).filter((entry): entry is [string, string] => typeof entry[1] === "string"),
    );
  } catch {
    return undefined;
  }
}

function WindowRouteError({ route }: { route: string }) {
  return (
    <DebugSourceOverlay enabled={readDebugSourcePreference()} source="src/App.tsx" className="debug-source-root-shell" contentClassName="debug-source-root-content">
      <main className="window-route-shell window-route-error">
        <section className="window-route-error-card">
          <h1>Unknown Amigo Editor window route</h1>
          <code>{route || "#/startup"}</code>
        </section>
      </main>
    </DebugSourceOverlay>
  );
}

const STARTUP_SPLASH_DURATION_MS = 3000;

export function App() {
  const [showSplash, setShowSplash] = useState(isStartupRouteForSplash() && !hasNoSplashUrlFlag());
  const [splashExiting, setSplashExiting] = useState(false);

  useEffect(() => {
    if (!showSplash) return;

    let cancelled = false;
    let exitTimeout: number | undefined;
    const dismissSplash = (animated: boolean) => {
      if (cancelled) return;
      if (!animated) {
        setShowSplash(false);
        return;
      }
      setSplashExiting(true);
      exitTimeout = window.setTimeout(() => {
        if (!cancelled) {
          setShowSplash(false);
        }
      }, 250);
    };

    void getLaunchFlags()
      .then((flags) => {
        if (!cancelled && flags.includes("--no-splash")) {
          dismissSplash(false);
        }
      })
      .catch(() => undefined);

    const timeout = window.setTimeout(() => dismissSplash(true), STARTUP_SPLASH_DURATION_MS);
    return () => {
      cancelled = true;
      window.clearTimeout(timeout);
      if (exitTimeout !== undefined) {
        window.clearTimeout(exitTimeout);
      }
    };
  }, [showSplash]);

  return (
    <EditorStoreProvider>
      <AppRouteBridge />
      {showSplash ? <AppSplash exiting={splashExiting} /> : null}
    </EditorStoreProvider>
  );
}
