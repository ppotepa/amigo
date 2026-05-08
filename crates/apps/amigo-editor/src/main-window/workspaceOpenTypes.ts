import type {
  EditorProjectFileDto,
  EditorSceneSummaryDto,
  ManagedAssetDto,
} from "../api/dto";
import type {
  EditorComponentDefinition,
  EditorComponentContextOf,
} from "../editor-components/componentTypes";

export type WorkspaceEditorOpenKind =
  | "component"
  | "project-file"
  | "scene"
  | "asset"
  | "ui-document";

export type OpenWorkspaceEditorRequest =
  | {
      kind: "component";
      component: EditorComponentDefinition<any>;
      titleOverride?: string;
      resourceUri?: string;
      context?: EditorComponentContextOf<EditorComponentDefinition<any>>;
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
