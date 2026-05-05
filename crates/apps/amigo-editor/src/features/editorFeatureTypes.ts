import type { EditorComponentDefinition } from "../editor-components/componentTypes";

export type EditorFeature = {
  id: string;
  components?: readonly EditorComponentDefinition[];
};
