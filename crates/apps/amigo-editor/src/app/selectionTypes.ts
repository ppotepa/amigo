export type EditorSelectionRef =
  | { kind: "empty" }
  | { kind: "mod"; modId: string }
  | { kind: "scene"; modId: string; sceneId: string }
  | { kind: "entity"; modId: string; sceneId: string; entityId: string }
  | { kind: "asset"; modId: string; assetKey: string; filePath?: string }
  | { kind: "projectFile"; modId: string; path: string };
