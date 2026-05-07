import type { EditorDiagnosticDto } from "../../api/dto";
import type { EditorTargetRef } from "../editorTargetTypes";

// @codemap anchor:diagnostic-target-adapter domain:workspace role:tree-adapter priority:P1 layer:app tags:editor-target,diagnostics,selection
export function diagnosticToTarget(diagnostic: EditorDiagnosticDto, index = 0): EditorTargetRef {
  return {
    kind: "diagnostic",
    diagnosticId: diagnosticIdFor(diagnostic, index),
    code: diagnostic.code,
    path: diagnostic.path ?? null,
  };
}

export function diagnosticIdFor(diagnostic: EditorDiagnosticDto, index = 0): string {
  return [
    diagnostic.level,
    diagnostic.code,
    diagnostic.path ?? "",
    diagnostic.message,
    String(index),
  ].join(":");
}
