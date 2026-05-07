import type { EditorProjectFileDto } from "../../api/dto";
import type { EditorTargetRef } from "../editorTargetTypes";

// @codemap anchor:file-target-adapter domain:project role:tree-adapter priority:P1 layer:app tags:editor-target,file,selection
export function projectFileToTarget(file: EditorProjectFileDto): EditorTargetRef {
  if (file.kind === "script" || file.kind === "sceneScript" || /\.rhai$/i.test(file.relativePath)) {
    return { kind: "script", path: file.relativePath };
  }

  return { kind: "projectFile", path: file.relativePath };
}

export function projectFilePathToTarget(path: string): EditorTargetRef {
  if (/\.rhai$/i.test(path)) {
    return { kind: "script", path };
  }

  return { kind: "projectFile", path };
}
