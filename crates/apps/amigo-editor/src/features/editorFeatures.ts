import { builtinEditorComponents } from "../editor-components/builtinComponents";
import type { EditorFeature } from "./editorFeatureTypes";

const legacyEditorFeature = {
  id: "legacy.workspace",
  components: builtinEditorComponents,
} satisfies EditorFeature;

export const EDITOR_FEATURES = [legacyEditorFeature] as const satisfies readonly EditorFeature[];
