import type {
  EditorSceneSnapshotDto,
  EditorSceneSummaryDto,
} from "../../../../api/dto";
import { SceneEditor2DCanvas } from "./SceneEditor2DCanvas";
import { SceneEditor25DCanvasPlaceholder } from "./SceneEditor25DCanvasPlaceholder";
import { SceneEditor3DCanvasPlaceholder } from "./SceneEditor3DCanvasPlaceholder";
import type {
  SceneEditorCanvasEngine,
  SceneEditorCanvasKind,
} from "../sceneEditorTypes";

export const SCENE_EDITOR_2D_ENGINE: SceneEditorCanvasEngine = {
  kind: "2d",
  label: "2D Canvas",
  render: SceneEditor2DCanvas,
};

export const SCENE_EDITOR_25D_ENGINE: SceneEditorCanvasEngine = {
  kind: "2.5d",
  label: "2.5D Canvas",
  render: SceneEditor25DCanvasPlaceholder,
};

export const SCENE_EDITOR_3D_ENGINE: SceneEditorCanvasEngine = {
  kind: "3d",
  label: "3D Canvas",
  render: SceneEditor3DCanvasPlaceholder,
};

export function selectSceneCanvasKind({
  scene,
  snapshot,
}: {
  scene: EditorSceneSummaryDto | null | undefined;
  snapshot?: EditorSceneSnapshotDto | null;
}): SceneEditorCanvasKind {
  if (snapshot?.canvasKind) return snapshot.canvasKind;
  if (!scene) return "2d";

  const haystack = [
    scene.id,
    scene.label,
    scene.path,
    scene.documentPath,
    scene.scriptPath,
  ]
    .filter(Boolean)
    .join(" ")
    .toLowerCase();

  if (haystack.includes("2.5d") || haystack.includes("2-5d") || haystack.includes("isometric") || haystack.includes("iso")) {
    return "2.5d";
  }

  if (haystack.includes("3d") || haystack.includes("mesh") || haystack.includes("material") || haystack.includes("cube")) {
    return "3d";
  }

  return "2d";
}

export function selectSceneCanvasEngine(kind: SceneEditorCanvasKind): SceneEditorCanvasEngine {
  if (kind === "3d") return SCENE_EDITOR_3D_ENGINE;
  if (kind === "2.5d") return SCENE_EDITOR_25D_ENGINE;
  return SCENE_EDITOR_2D_ENGINE;
}
