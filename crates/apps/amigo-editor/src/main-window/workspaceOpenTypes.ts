import type {
  EditorProjectFileDto,
  EditorSceneSummaryDto,
  ManagedAssetDto,
} from "../api/dto";

export type WorkspaceEditorOpenKind =
  | "component"
  | "project-file"
  | "scene"
  | "asset"
  | "ui-document";

export type OpenWorkspaceEditorRequest =
  | {
      kind: "component";
      componentId: string;
      titleOverride?: string;
      resourceUri?: string;
      context?: Record<string, string>;
    }
  | {
      kind: "project-file";
      file: EditorProjectFileDto;
    }
  | {
      kind: "scene";
      scene: EditorSceneSummaryDto;
    }
  | {
      kind: "asset";
      asset: ManagedAssetDto;
    }
  | {
      kind: "ui-document";
      sceneId: string;
      entityId: string;
      componentIndex: number;
      focusPath?: string;
      titleOverride?: string;
    };
