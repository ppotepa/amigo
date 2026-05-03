import { EditorStoreProvider } from "./app/editorStore";
import { StartupDialog } from "./startup/StartupDialog";
import { useEditorStore } from "./app/editorStore";
import { useEffect, useMemo, useState } from "react";
import { MainEditorWindow } from "./main-window/MainEditorWindow";

function useHashRoute() {
  const [hash, setHash] = useState(window.location.hash);

  useEffect(() => {
    const handleHashChange = () => setHash(window.location.hash);
    window.addEventListener("hashchange", handleHashChange);
    return () => window.removeEventListener("hashchange", handleHashChange);
  }, []);

  return hash;
}

function sessionIdFromHash(hash: string): string | null {
  const queryStart = hash.indexOf("?");
  if (queryStart < 0) {
    return null;
  }

  const params = new URLSearchParams(hash.slice(queryStart + 1));
  return params.get("sessionId");
}

function AppRouteBridge() {
  const { loadEditorSession, recordEvent, state } = useEditorStore();
  const hash = useHashRoute();
  const sessionId = useMemo(() => sessionIdFromHash(hash), [hash]);

  useEffect(() => {
    if (hash.startsWith("#/workspace")) {
      return;
    }

    recordEvent({ type: "StartupDialogOpened" });
  }, [hash, recordEvent]);

  useEffect(() => {
    if (!hash.startsWith("#/workspace") || !sessionId || state.activeSession?.sessionId === sessionId) {
      return;
    }

    void loadEditorSession(sessionId);
  }, [hash, loadEditorSession, sessionId, state.activeSession?.sessionId]);

  if (hash.startsWith("#/workspace")) {
    return <MainEditorWindow />;
  }

  return <StartupDialog />;
}

export function App() {
  return (
    <EditorStoreProvider>
      <AppRouteBridge />
    </EditorStoreProvider>
  );
}
