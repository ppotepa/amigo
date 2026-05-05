import type React from "react";
import type {
  EditorCommandDto,
  EditorLiveCommandResultDto,
  EditorLiveSceneSessionDto,
  EditorSceneCanvasKindDto,
  EditorSceneEntityDto,
  EditorSceneSnapshotQualityDto,
  EditorSceneSnapshotDto,
  EditorTransform2Dto,
  ScenePreviewDto,
} from "../../../api/dto";
import type { SceneEditorModeKind } from "./sceneEditorMode";
import type { SceneEditorPreviewSyncState } from "./sceneEditorPreviewSync";

export type SceneEditorMode = "edit" | "preview" | "play";

export type SceneEditorTool =
  | "select"
  | "move"
  | "scale"
  | "rotate"
  | "pan";

export type SceneEditorCanvasKind = EditorSceneCanvasKindDto;

export type SceneEditorResolution = {
  width: number;
  height: number;
};

export type SceneEditorPoint = {
  x: number;
  y: number;
};

export type SceneEditorRect = {
  x: number;
  y: number;
  width: number;
  height: number;
};

export type SceneEditorTransform = {
  x: number;
  y: number;
  rotation: number;
  scaleX: number;
  scaleY: number;
};

export type SceneEditorViewportState = {
  zoom: number;
  panX: number;
  panY: number;
};

export type SceneEditorEntityKind =
  | "ui"
  | "camera"
  | "physics"
  | "motion"
  | "render"
  | "audio"
  | "particles"
  | "tilemap"
  | "script"
  | "threed"
  | "other";

export type SceneEditorEntity = {
  id: string;
  name: string;
  source: EditorSceneEntityDto;
  kind: SceneEditorEntityKind;
  visible: boolean;
  locked: boolean;
  transform: SceneEditorTransform;
  bounds: SceneEditorRect;
  componentTypes: string[];
};

export type SceneEditorLayoutSource =
  | "runtime"
  | "document"
  | "fallback"
  | "missingSnapshot";

export type SceneEditorModel = {
  sceneId: string;
  resolution: SceneEditorResolution;
  entities: SceneEditorEntity[];
  layoutSource: SceneEditorLayoutSource;
  quality?: EditorSceneSnapshotQualityDto;
};

export type SceneEditorDragState = {
  entityId: string;
  pointerId: number;
  startScreen: SceneEditorPoint;
  startTransform: SceneEditorTransform;
};

export type SceneEditorCommand =
  | {
      type: "selectEntity";
      entityId: string | null;
    }
  | {
      type: "moveEntity";
      entityId: string;
      x: number;
      y: number;
    };

export type SceneEditorCanvasProps = {
  scene: import("../../../api/dto").EditorSceneSummaryDto;
  canvasKind: EditorSceneCanvasKindDto;
  model: SceneEditorModel;
  preview?: ScenePreviewDto;
  previewSync?: SceneEditorPreviewSyncState;
  snapshot?: EditorSceneSnapshotDto | null;
  selectedEntityId: string | null;
  editorModeKind: SceneEditorModeKind;
  liveAvailable: boolean;
  liveOpening?: boolean;
  liveSession?: EditorLiveSceneSessionDto | null;
  mode: SceneEditorMode;
  tool: SceneEditorTool;
  viewport: SceneEditorViewportState;
  onViewportChange: (viewport: SceneEditorViewportState) => void;
  onEditorModeKindChange: (mode: SceneEditorModeKind) => void;
  onModeChange: (mode: SceneEditorMode) => void;
  onToolChange: (tool: SceneEditorTool) => void;
  onFitViewport: () => void;
  onResetZoom: () => void;
  onZoomChange: (zoom: number) => void;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onSelectEntity: (entityId: string | null) => void;
  onApplyCommand?: (command: EditorCommandDto) => Promise<import("../../../api/dto").EditorCommandResultDto | null>;
  onApplyLiveTransform?: (entityId: string, transform: EditorTransform2Dto) => Promise<EditorLiveCommandResultDto | null>;
};

export type SceneEditorCanvasEngine = {
  kind: SceneEditorCanvasKind;
  label: string;
  render: React.ComponentType<SceneEditorCanvasProps>;
};
