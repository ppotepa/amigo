import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { TargetContextContent } from "../../features/target-context/TargetContextContent";

export function TargetViewFallback(props: EditorComponentProps<WorkspaceRuntimeServices>) {
  return <TargetContextContent {...props} />;
}
