import type {
  EditorSceneSnapshotDto,
  EditorSceneSummaryDto,
} from "../../../../api/dto";
import { SceneEditorCanvas } from "../SceneEditorCanvas";
import type {
  SceneEditorCanvasEngine,
  SceneEditorCanvasKind,
} from "../sceneEditorTypes";

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
  switch (kind) {
    case "2d":
      return {
        kind: "2d",
        label: "2D Engine Viewport",
        render: SceneEditorCanvas,
      };
    case "2.5d":
      return {
        kind: "2.5d",
        label: "2.5D Engine Viewport",
        render: SceneEditorCanvas,
      };
    case "3d":
      return {
        kind: "3d",
        label: "3D Engine Viewport",
        render: SceneEditorCanvas,
      };
  }
}
