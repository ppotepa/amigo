import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { SceneEditorWorkbench } from "./editor/SceneEditorWorkbench";

export function ScenePreviewWorkbench(props: EditorComponentProps<WorkspaceRuntimeServices>) {
  return <SceneEditorWorkbench {...props} />;
}
