import type React from "react";
import type {
  EditorCommandDto,
  EditorFrameDto,
  EditorFrameResultDto,
  EditorModeSessionDto,
  EditorObjectEditCommandKindDto,
  EditorObjectPlacementKindDto,
  EditorBounds2Dto,
  EditorPointerEventDto,
  EditorViewportDto,
  EditorSceneCanvasKindDto,
  EditorSceneEntityDto,
  EditorSceneSnapshotQualityDto,
  EditorSceneSnapshotDto,
} from "../../../api/dto";
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
  movable: boolean;
  lockedReason?: string;
  placementKind: EditorObjectPlacementKindDto;
  editCommandKind: EditorObjectEditCommandKindDto;
  transform: SceneEditorTransform;
  bounds: SceneEditorRect;
  renderBounds?: EditorBounds2Dto;
  selectionBounds?: EditorBounds2Dto;
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
  frame?: EditorFrameDto | null;
  previewSync?: SceneEditorPreviewSyncState;
  snapshot?: EditorSceneSnapshotDto | null;
  editorModeSession?: EditorModeSessionDto | null;
  selectedEntityId: string | null;
  mode: SceneEditorMode;
  tool: SceneEditorTool;
  viewport: SceneEditorViewportState;
  onViewportChange: (viewport: SceneEditorViewportState) => void;
  onViewportResize?: (viewport: EditorViewportDto) => Promise<EditorFrameResultDto | null>;
  onModeChange: (mode: SceneEditorMode) => void;
  onToolChange: (tool: SceneEditorTool) => void;
  onFitViewport: () => void;
  onResetZoom: () => void;
  onZoomChange: (zoom: number) => void;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onSelectEntity: (entityId: string | null) => void;
  onApplyCommand?: (command: EditorCommandDto) => Promise<import("../../../api/dto").EditorCommandResultDto | null>;
  onPointerEvent?: (event: EditorPointerEventDto) => Promise<EditorFrameResultDto | null>;
};

export type SceneEditorCanvasEngine = {
  kind: SceneEditorCanvasKind;
  label: string;
  render: React.ComponentType<SceneEditorCanvasProps>;
};
