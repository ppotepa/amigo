import type { EditorComponentProps } from "../../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../../main-window/workspaceRuntimeServices";
import { TargetContextContent } from "../../target-context/TargetContextContent";

export function EntityContextContent(props: EditorComponentProps<WorkspaceRuntimeServices>) {
  return <TargetContextContent {...props} />;
}

