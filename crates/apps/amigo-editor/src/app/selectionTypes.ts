export type EditorUiNodeSelectionRef = {
  kind: "uiNode";
  modId: string;
  sceneId: string;
  entityId: string;
  componentIndex: number;
  nodePath: string;
};

export type EditorSelectionRef =
  | { kind: "empty" }
  | { kind: "mod"; modId: string }
  | { kind: "scene"; modId: string; sceneId: string }
  | { kind: "entity"; modId: string; sceneId: string; entityId: string }
  | EditorUiNodeSelectionRef
  | { kind: "asset"; modId: string; assetKey: string; filePath?: string }
  | { kind: "projectFile"; modId: string; path: string };
