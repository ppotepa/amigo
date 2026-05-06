import type {
  EditorProjectFileDto,
  EditorProjectTreeDto,
  EditorSceneSummaryDto,
} from "../../api/dto";
import { findProjectFile, normalizePath } from "./fileTreeSelectors";
import { relativeProjectPath } from "./yamlSourceRefs";

export type ScriptSourceRef = {
  label: string;
  path: string;
  title?: string;
};

export function isScriptPath(path: string | null | undefined): boolean {
  return /\.rhai$/i.test(path ?? "");
}

export function isScriptProjectFile(file: EditorProjectFileDto): boolean {
  return file.kind === "script" ||
    file.kind === "sceneScript" ||
    file.kind === "scriptPackage" ||
    isScriptPath(file.name);
}

export function sceneScriptSource(scene: EditorSceneSummaryDto | null | undefined): ScriptSourceRef | null {
  if (!scene?.scriptPath || !isScriptPath(scene.scriptPath)) return null;
  return {
    label: "Scene Script",
    path: relativeProjectPath(scene.scriptPath),
    title: scene.scriptPath,
  };
}

export function findScriptSourceFile(
  projectTree: EditorProjectTreeDto | undefined,
  source: ScriptSourceRef | null | undefined,
): EditorProjectFileDto | null {
  if (!projectTree?.root || !source?.path) return null;

  const file = findProjectFile(projectTree.root, normalizePath(relativeProjectPath(source.path)));
  return file && isScriptProjectFile(file) ? file : null;
}
