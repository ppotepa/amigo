import { builtinEditorComponents } from "../editor-components/builtinComponents";
import type { EditorFeature } from "./editorFeatureTypes";

const workspaceEditorFeature = {
  id: "workspace.editor",
  components: builtinEditorComponents,
} satisfies EditorFeature;

export const EDITOR_FEATURES = [workspaceEditorFeature] as const satisfies readonly EditorFeature[];
