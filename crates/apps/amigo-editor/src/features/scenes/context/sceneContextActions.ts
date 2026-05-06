import type {
  EditorProjectFileDto,
  EditorProjectTreeDto,
  EditorSceneSummaryDto,
} from "../../../api/dto";
import { findScriptSourceFile, sceneScriptSource } from "../../files/scriptSourceRefs";
import { findYamlSourceFile, sceneYamlSource } from "../../files/yamlSourceRefs";

export function resolveSceneYamlFile(
  projectTree: EditorProjectTreeDto | undefined,
  scene: EditorSceneSummaryDto,
): EditorProjectFileDto | null {
  return findYamlSourceFile(projectTree?.root, sceneYamlSource(scene));
}

export function resolveSceneScriptFile(
  projectTree: EditorProjectTreeDto | undefined,
  scene: EditorSceneSummaryDto,
): EditorProjectFileDto | null {
  return findScriptSourceFile(projectTree, sceneScriptSource(scene));
}

export function openSceneScriptFile({
  onOpenFile,
  projectTree,
  scene,
}: {
  projectTree: EditorProjectTreeDto | undefined;
  scene: EditorSceneSummaryDto;
  onOpenFile?: (file: EditorProjectFileDto) => void;
}): boolean {
  const file = resolveSceneScriptFile(projectTree, scene);
  if (!file) return false;
  onOpenFile?.(file);
  return true;
}
