import type { EditorSelectionRef } from "./selectionTypes";

export function selectedModId(selection: EditorSelectionRef): string | null {
  switch (selection.kind) {
    case "mod":
    case "scene":
    case "entity":
    case "asset":
    case "projectFile":
      return selection.modId;
    case "empty":
      return null;
  }
}

export function selectedSceneId(selection: EditorSelectionRef): string | null {
  switch (selection.kind) {
    case "scene":
    case "entity":
      return selection.sceneId;
    default:
      return null;
  }
}

export function selectedEntityId(selection: EditorSelectionRef): string | null {
  return selection.kind === "entity" ? selection.entityId : null;
}

export function selectedAssetKey(selection: EditorSelectionRef): string | null {
  return selection.kind === "asset" ? selection.assetKey : null;
}

export function selectedFilePath(selection: EditorSelectionRef): string | null {
  return selection.kind === "projectFile" ? selection.path : null;
}
