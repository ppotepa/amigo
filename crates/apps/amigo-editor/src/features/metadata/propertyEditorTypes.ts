import type { EditorTargetRef } from "../../editor-targets";

export type PropertyEditRequest = {
  target: EditorTargetRef;
  path: string;
  value: unknown;
};
