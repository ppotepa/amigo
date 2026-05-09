import type { ResolvedEditorTarget } from "../editor-targets";

export function defaultDetailsTabForTarget(
  target: Pick<ResolvedEditorTarget, "ref" | "diagnostics"> | null,
): string {
  if (!target) return "scene-info";
  if (target.ref.kind === "scene") return "scene-info";
  if (target.ref.kind === "projectFile" || target.ref.kind === "script") return "file-info";
  if (target.diagnostics.length > 0) return "diagnostics";
  return "properties";
}
