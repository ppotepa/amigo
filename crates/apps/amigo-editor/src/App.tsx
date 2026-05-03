import { EditorStoreProvider } from "./app/editorStore";
import { StartupDialog } from "./startup/StartupDialog";
import { useEditorStore } from "./app/editorStore";
import { useEffect } from "react";
import { MainEditorWindow } from "./main-window/MainEditorWindow";

function StartupDialogBridge() {
  const { state, recordEvent } = useEditorStore();

  useEffect(() => {
    recordEvent({ type: "StartupDialogOpened" });
  }, [recordEvent]);

  return state.appMode === "editor" ? <MainEditorWindow /> : <StartupDialog />;
}

export function App() {
  return (
    <EditorStoreProvider>
      <StartupDialogBridge />
    </EditorStoreProvider>
  );
}
