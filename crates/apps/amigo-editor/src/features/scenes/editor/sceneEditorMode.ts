export type SceneEditorModeKind =
  | "document"
  | "live";

export function sceneEditorModeLabel(mode: SceneEditorModeKind): string {
  return mode === "live" ? "Live Mode" : "Document Mode";
}
