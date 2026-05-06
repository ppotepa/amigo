export type EditorStatus = "valid" | "warning" | "error" | "missingDependency" | "invalidManifest" | "missingSceneFile" | "previewFailed";

export type PreviewStatus = "missing" | "queued" | "rendering" | "ready" | "failed";

export type TaskStatus = "queued" | "running" | "completed" | "failed";

export interface EditorDiagnosticDto {
  level: "info" | "warning" | "error";
  code: string;
  message: string;
  path?: string | null;
}

export interface EditorSceneSummaryDto {
  id: string;
  label: string;
  description?: string | null;
  path: string;
  documentPath: string;
  scriptPath: string;
  launcherVisible: boolean;
  status: EditorStatus;
  previewCacheKey: string;
  previewImageUrl?: string | null;
  previewFps: number;
  diagnostics: EditorDiagnosticDto[];
}

export interface EditorModSummaryDto {
  id: string;
  name: string;
  version: string;
  description?: string | null;
  authors: string[];
  rootPath: string;
  dependencies: string[];
  missingDependencies: string[];
  capabilities: string[];
  sceneCount: number;
  visibleSceneCount: number;
  status: EditorStatus;
  diagnostics: EditorDiagnosticDto[];
  lastModified?: string | null;
  projectCacheId?: string;
  previewStatus: PreviewStatus;
  contentSummary: EditorContentSummaryDto;
}

export interface EditorContentSummaryDto {
  scenes: number;
  sceneYaml: number;
  scripts: number;
  textures: number;
  spritesheets: number;
  audio: number;
  fonts: number;
  tilemaps: number;
  tilesets: number;
  packages: number;
  unknownFiles: number;
  totalFiles: number;
}

export interface EditorModDetailsDto extends EditorModSummaryDto {
  scenes: EditorSceneSummaryDto[];
}

export interface ScenePreviewDto {
  modId: string;
  sceneId: string;
  status: PreviewStatus;
  fps: number;
  frameCount: number;
  imageUrl?: string | null;
  frameUrls: string[];
  width: number;
  height: number;
  durationMs: number;
  generatedAt?: string | null;
  sourceHash: string;
  diagnostics: EditorDiagnosticDto[];
}

export interface EditorSceneEntityDto {
  id: string;
  name: string;
  tags: string[];
  groups: string[];
  visible: boolean;
  simulationEnabled: boolean;
  collisionEnabled: boolean;
  hasTransform2: boolean;
  hasTransform3: boolean;
  propertyCount: number;
  componentCount: number;
  componentTypes: string[];
}

export interface EditorSceneHierarchyDto {
  modId: string;
  sceneId: string;
  sceneLabel: string;
  entityCount: number;
  componentCount: number;
  entities: EditorSceneEntityDto[];
  diagnostics: EditorDiagnosticDto[];
}

export interface EditorProjectFileDto {
  name: string;
  path: string;
  relativePath: string;
  kind: string;
  isDir: boolean;
  sizeBytes: number;
  children: EditorProjectFileDto[];
}

export interface EditorProjectTreeDto {
  modId: string;
  rootPath: string;
  totalFiles: number;
  root: EditorProjectFileDto;
}

export interface EditorProjectStructureTreeDto {
  modId: string;
  rootPath: string;
  root: EditorProjectStructureNodeDto;
}

export interface EditorProjectStructureNodeDto {
  id: string;
  label: string;
  kind: string;
  icon: string;
  status?: string | null;
  count?: number | null;
  path?: string | null;
  expectedPath?: string | null;
  exists: boolean;
  empty: boolean;
  ghost: boolean;
  file?: EditorProjectFileDto | null;
  scene?: EditorSceneSummaryDto | null;
  children: EditorProjectStructureNodeDto[];
}

export interface EditorProjectFileContentDto {
  modId: string;
  path: string;
  relativePath: string;
  kind: string;
  language: string;
  sizeBytes: number;
  content: string;
  diagnostics: EditorDiagnosticDto[];
}

export interface WriteProjectFileRequestDto {
  relativePath: string;
  content: string;
}

export interface OpenModResultDto {
  modId: string;
  rootPath: string;
  sessionId: string;
  createdAt: string;
  selectedSceneId?: string | null;
}

export interface ThemeSettingsDto {
  activeThemeId: string;
  activeFontId: string;
}

export interface EditorSettingsDto {
  settingsVersion: number;
  modsRoot?: string | null;
  cacheRootOverride?: string | null;
  activeThemeId: string;
  activeFontId: string;
  lastOpenedModId?: string | null;
}

export interface CacheProjectInfoDto {
  projectCacheId: string;
  modId: string;
  displayName: string;
  rootPath: string;
  lastSeenAt: string;
  projectSizeBytes: number;
}

export interface CacheInfoDto {
  cacheRoot: string;
  cacheRootMode: string;
  totalSizeBytes: number;
  projectCount: number;
  projects: CacheProjectInfoDto[];
}

export interface CachePolicyDto {
  maxPreviewCacheBytes?: number | null;
  maxAgeDays?: number | null;
  autoCleanupEnabled: boolean;
}

export interface CacheMaintenanceResultDto {
  removedEntries: number;
  removedBytes: number;
  remainingPreviewBytes: number;
  orphanedProjectsRemoved: number;
}

export interface EditorWindowInfoDto {
  label: string;
  kind: string;
  sessionId?: string | null;
  focused: boolean;
  lastSeenAt: string;
}

export interface EditorWindowRegistryDto {
  windows: EditorWindowInfoDto[];
}

export interface EditorSessionDto {
  sessionId: string;
  modId: string;
  rootPath: string;
  createdAt: string;
  selectedSceneId?: string | null;
}

export type SheetKind = "tileset" | "spritesheet";

export type SheetSourceSchemaKind = "descriptor";

export interface SheetResourceDto {
  resourceUri: string;
  absolutePath: string;
  relativePath: string;
  kind: SheetKind;
  schemaVersion: number;
  sourceSchemaKind: SheetSourceSchemaKind;
  id: string;
  label: string;
  imagePath: string;
  imageAbsolutePath: string;
  imageExists: boolean;
  imageWidth?: number | null;
  imageHeight?: number | null;
  declaredImageWidth?: number | null;
  declaredImageHeight?: number | null;
  cellWidth: number;
  cellHeight: number;
  columns: number;
  rows: number;
  count: number;
  marginX: number;
  marginY: number;
  spacingX: number;
  spacingY: number;
  tileset?: TileSetPayloadDto | null;
  animations?: SpriteAnimationDto[] | null;
  diagnostics: EditorDiagnosticDto[];
}

export interface SpriteAnimationDto {
  id: string;
  frames: number[];
  fps?: number | null;
  looping?: boolean | null;
}

export interface TileSetPayloadDto {
  defaults: TileSetDefaultsDto;
  tiles: TileMetadataDto[];
}

export interface TileSetDefaultsDto {
  collision: string;
  damageable: boolean;
}

export interface TileMetadataDto {
  key: string;
  id: number;
  role?: string | null;
  name?: string | null;
  category?: string | null;
  collision?: string | null;
  damageable?: boolean | null;
  tags: string[];
}

export interface TilemapResourceDto {
  resourceUri: string;
  absolutePath: string;
  relativePath: string;
  schemaVersion: number;
  id: string;
  label: string;
  tilesetResourceUri?: string | null;
  width: number;
  height: number;
  originOffsetX: number;
  originOffsetY: number;
  cells: TilemapCellDto[];
  diagnostics: EditorDiagnosticDto[];
}

export interface TilemapCellDto {
  x: number;
  y: number;
  tileId: number;
}

export interface TileRulesetResourceDto {
  resourceUri: string;
  absolutePath: string;
  relativePath: string;
  schemaVersion: number;
  id: string;
  label: string;
  tileWidth: number;
  tileHeight: number;
  tilesetResourceUri?: string | null;
  terrains: TileRulesetTerrainDto[];
  diagnostics: EditorDiagnosticDto[];
}

export interface TileRulesetTerrainDto {
  id: string;
  symbol: string;
  collision?: string | null;
  variants: TileRulesetVariantsDto;
}

export interface TileRulesetVariantsDto {
  single?: number | null;
  leftCap?: number | null;
  middle?: number | null;
  rightCap?: number | null;
  sideLeft?: number | null;
  sideRight?: number | null;
  center?: number | null;
  topCap?: number | null;
  bottomCap?: number | null;
  verticalMiddle?: number | null;
  outerCornerTopLeft?: number | null;
  outerCornerTopRight?: number | null;
  outerCornerBottomLeft?: number | null;
  outerCornerBottomRight?: number | null;
  innerCornerTopLeft?: number | null;
  innerCornerTopRight?: number | null;
  innerCornerBottomLeft?: number | null;
  innerCornerBottomRight?: number | null;
}

export type AssetStatusDto = "valid" | "warning" | "error" | "missingSource";
export type AssetDomainDto = "spritesheet" | "tilemap" | "audio" | "font" | "scene" | "script" | "raw";
export type AssetRoleDto = "family" | "subasset" | "reference" | "file";

export interface AssetSourceRefDto {
  path: string;
  relativePath: string;
  exists: boolean;
  role: string;
}

export interface ManagedAssetDto {
  assetId: string;
  kind: string;
  label: string;
  assetKey: string;
  parentKey?: string | null;
  references: string[];
  usedBy: string[];
  domain: AssetDomainDto;
  role: AssetRoleDto;
  descriptorPath: string;
  descriptorRelativePath: string;
  sourceFiles: AssetSourceRefDto[];
  status: AssetStatusDto;
  diagnostics: EditorDiagnosticDto[];
}

export interface RawAssetFileDto {
  path: string;
  relativePath: string;
  mediaType: string;
  width?: number | null;
  height?: number | null;
  referencedBy: string[];
  orphan: boolean;
}

export interface CreateAssetImportOptionsDto {
  tileWidth?: number | null;
  tileHeight?: number | null;
  columns?: number | null;
  rows?: number | null;
  tileCount?: number | null;
  marginX?: number | null;
  marginY?: number | null;
  spacingX?: number | null;
  spacingY?: number | null;
  fps?: number | null;
}

export interface AssetRegistryDto {
  sessionId: string;
  modId: string;
  rootPath: string;
  managedAssets: ManagedAssetDto[];
  rawFiles: RawAssetFileDto[];
  diagnostics: EditorDiagnosticDto[];
}

export interface CreateAssetDescriptorRequestDto {
  rawFilePath: string;
  kind: string;
  assetId: string;
  importOptions?: CreateAssetImportOptionsDto | null;
}

export interface CreateSpritesheetRulesetRequestDto {
  spritesheetAssetKey: string;
  rulesetId?: string | null;
}

export type EditorModeDto = "edit" | "preview" | "play";

export type EditorToolDto =
  | "select"
  | "move"
  | "scale"
  | "rotate"
  | "rect"
  | "pan";

export type EditorToolSpaceDto =
  | "world"
  | "local";

export interface EditorSnapSettingsDto {
  enabled: boolean;
  gridSize: number;
  angleStepDeg: number;
  scaleStep: number;
}

export interface EditorSelectionDto {
  selectedEntityIds: string[];
}

export interface EditorToolStateDto {
  activeTool: EditorToolDto;
  space: EditorToolSpaceDto;
  snap: EditorSnapSettingsDto;
}

export type EditorFrameTransportKindDto = "image-url";

export type EditorRenderTransportPreferenceDto =
  | "auto"
  | "image-url";

export interface EditorViewportDto {
  cssWidth: number;
  cssHeight: number;
  renderWidth: number;
  renderHeight: number;
  devicePixelRatio: number;
  cameraX?: number | null;
  cameraY?: number | null;
  zoom?: number | null;
}

export interface EditorFrameDto {
  sessionId: string;
  revision: number;
  transport: EditorFrameTransportKindDto;
  width: number;
  height: number;
  devicePixelRatio: number;
  imageUrl?: string | null;
  renderTimeMs?: number | null;
  encodedBytes?: number | null;
}

export interface EditorModeSessionDto {
  editorModeSessionId: string;
  editorSessionId: string;
  modId: string;
  sceneId: string;
  mode: EditorModeDto;
  tool: EditorToolDto;
  dirty: boolean;
  canUndo: boolean;
  canRedo: boolean;
  revision: number;
  transport: EditorFrameTransportKindDto;
  cursor: EditorCursorDto;
  hoveredControlId?: string | null;
  hoveredHandleId?: string | null;
  activeControlId?: string | null;
  activeHandleId?: string | null;
}

export type EditorCursorIconDto =
  | "default"
  | "select"
  | "move"
  | "move-x"
  | "move-y"
  | "rotate"
  | "scale"
  | "scale-x"
  | "scale-y"
  | "rect"
  | "pan"
  | "grab"
  | "grabbing"
  | "not-allowed";

export interface EditorCursorDto {
  icon: EditorCursorIconDto;
  visible: boolean;
  label?: string | null;
}

export type EditorControlStateDto =
  | "idle"
  | "hovered"
  | "active"
  | "disabled";

export type EditorControlKindDto =
  | "transform-2-d"
  | "text-2-d"
  | "vector-shape-2-d"
  | "sprite-2-d"
  | "tile-map-2-d"
  | "camera-2-d"
  | "ui-rect-2-d";

export type EditorControlSpaceDto = "scene-2-d" | "screen" | "world-3-d";

export interface OpenEditorModeSessionResultDto {
  session: EditorModeSessionDto;
  snapshot: EditorSceneSnapshotDto;
  frame: EditorFrameDto;
  diagnostics: EditorDiagnosticDto[];
}

export interface EditorFrameResultDto {
  ok: boolean;
  session?: EditorModeSessionDto | null;
  snapshot?: EditorSceneSnapshotDto | null;
  frame?: EditorFrameDto | null;
  diagnostics: EditorDiagnosticDto[];
  message?: string | null;
}

export interface EditorPointerModifiersDto {
  shift: boolean;
  ctrl: boolean;
  alt: boolean;
  meta: boolean;
}

export interface EditorPointerEventDto {
  type: "pointerDown" | "pointerMove" | "pointerUp" | "pointerCancel" | "wheel";
  x: number;
  y: number;
  sceneX?: number | null;
  sceneY?: number | null;
  frameX?: number | null;
  frameY?: number | null;
  button?: number | null;
  buttons?: number | null;
  pointerId: number;
  deltaX?: number | null;
  deltaY?: number | null;
  modifiers: EditorPointerModifiersDto;
  viewport: EditorViewportDto;
}

/**
 * Canvas implementation selected by editor-mode.
 *
 * 2D uses bounds2/local hit-test.
 * 2.5D and 3D must use separate picking/gizmo implementations.
 */
export type EditorSceneCanvasKindDto =
  | "2d"
  | "2.5d"
  | "3d";

export type EditorObjectPlacementKindDto =
  | "transform2"
  | "tilemap-marker"
  | "attached"
  | "ui-layout"
  | "computed-runtime"
  | "not-editable";

export type EditorObjectEditCommandKindDto =
  | "set-transform2"
  | "set-tilemap-marker-offset"
  | "set-attached-local-offset"
  | "set-ui-rect"
  | "set-tilemap-origin"
  | "locked";

/**
 * Source of editor geometry.
 *
 * runtime/document are real layout sources and may enable viewport interaction.
 * fallback/unavailable must never enable viewport picking or transform drag.
 */
export type EditorSceneSnapshotLayoutSourceDto =
  | "runtime"
  | "document"
  | "fallback"
  | "unavailable";

export interface EditorSceneSnapshotQualityDto {
  indexedEntities: number;
  objects: number;
  editableObjects: number;
  objectsWithoutTransform: number;
  objectsWithoutBounds: number;
  unsupportedBoundsProviders: number;
  diagnosticsByCode: Record<string, number>;
}

export interface EditorCameraDto {
  x: number;
  y: number;
  zoom: number;
  viewportWidth: number;
  viewportHeight: number;
}

export interface EditorTransform2Dto {
  x: number;
  y: number;
  rotation: number;
  scaleX: number;
  scaleY: number;
  zIndex?: number;
}

export interface EditorTransform3Dto {
  x: number;
  y: number;
  z: number;
  rotationX: number;
  rotationY: number;
  rotationZ: number;
  scaleX: number;
  scaleY: number;
  scaleZ: number;
}

export interface EditorBounds2Dto {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface EditorSceneObjectDto {
  entityId: string;
  name: string;
  visible: boolean;
  selectable: boolean;
  locked: boolean;
  movable: boolean;
  lockedReason?: string;
  category: string;
  componentTypes: string[];
  placementKind: EditorObjectPlacementKindDto;
  editCommandKind: EditorObjectEditCommandKindDto;
  /**
   * transform2 is the editor-space origin/pivot for 2D editing.
   * renderBounds2 is approximate visual bounds.
   * selectionBounds2 is the interactive hit-test/gizmo bounds.
   * bounds2 is kept as a compatibility alias for selectionBounds2.
   */
  transform2?: EditorTransform2Dto;
  transform3?: EditorTransform3Dto;
  bounds2?: EditorBounds2Dto;
  renderBounds2?: EditorBounds2Dto;
  selectionBounds2?: EditorBounds2Dto;
}

export type EditorGizmoKindDto =
  | "SelectionBounds2D"
  | "Move2D"
  | "Rotate2D"
  | "Scale2D"
  | "Rect2D";

export type EditorGizmoHandleKindDto =
  | "Body"
  | "AxisX"
  | "AxisY"
  | "PlaneXY"
  | "RotationRing"
  | "ScaleCornerNW"
  | "ScaleCornerNE"
  | "ScaleCornerSW"
  | "ScaleCornerSE"
  | "ScaleEdgeN"
  | "ScaleEdgeE"
  | "ScaleEdgeS"
  | "ScaleEdgeW";

export type EditorGizmoToneDto =
  | "neutral"
  | "selection"
  | "x"
  | "y"
  | "rotation"
  | "scale"
  | "warning"
  | "hoverX"
  | "hoverY"
  | "center"
  | "centerHover"
  | "centerActive"
  | "rotationHover"
  | "rotationActive"
  | "scaleHover"
  | "scaleActive"
  | "active";

export interface EditorGizmoPointDto {
  x: number;
  y: number;
}

export interface EditorGizmoRectDto {
  x: number;
  y: number;
  width: number;
  height: number;
}

export type EditorGizmoPrimitiveDto =
  | {
      type: "Line2D";
      from: EditorGizmoPointDto;
      to: EditorGizmoPointDto;
      tone: EditorGizmoToneDto;
    }
  | {
      type: "Arrow2D";
      from: EditorGizmoPointDto;
      to: EditorGizmoPointDto;
      tone: EditorGizmoToneDto;
    }
  | {
      type: "Rect2D";
      rect: EditorGizmoRectDto;
      tone: EditorGizmoToneDto;
    }
  | {
      type: "Circle2D";
      center: EditorGizmoPointDto;
      radius: number;
      tone: EditorGizmoToneDto;
    }
  | {
      type: "Ring2D";
      center: EditorGizmoPointDto;
      innerRadius: number;
      outerRadius: number;
      tone: EditorGizmoToneDto;
    };

export type EditorGizmoHitShapeDto =
  | {
      type: "Rect2D";
      rect: EditorGizmoRectDto;
    }
  | {
      type: "Circle2D";
      center: EditorGizmoPointDto;
      radius: number;
    }
  | {
      type: "Ring2D";
      center: EditorGizmoPointDto;
      innerRadius: number;
      outerRadius: number;
    };

export interface EditorGizmoHandleDto {
  id: string;
  kind: EditorGizmoHandleKindDto;
  cursor?: string | null;
  hitShape: EditorGizmoHitShapeDto;
}

export interface EditorGizmoDto {
  id: string;
  kind: EditorGizmoKindDto;
  entityId?: string | null;
  primitives: EditorGizmoPrimitiveDto[];
  handles: EditorGizmoHandleDto[];
}

export interface EditorSceneSnapshotDto {
  modId: string;
  sceneId: string;
  canvasKind: EditorSceneCanvasKindDto;
  layoutSource: EditorSceneSnapshotLayoutSourceDto;
  width: number;
  height: number;
  camera: EditorCameraDto;
  quality: EditorSceneSnapshotQualityDto;
  objects: EditorSceneObjectDto[];
  diagnostics: EditorDiagnosticDto[];
  gizmos: EditorGizmoDto[];
  selection: EditorSelectionDto;
  toolState: EditorToolStateDto;
}

export type EditorCommandDto =
  | {
      type: "SelectEntity";
      sceneId: string;
      entityId: string;
    }
  | {
      type: "SetEntityTransform2D";
      sceneId: string;
      entityId: string;
      transform: EditorTransform2Dto;
    }
  | {
      type: "MoveEntity2D";
      sceneId: string;
      entityId: string;
      dx: number;
      dy: number;
    }
  | {
      type: "SetTileMapMarker2D";
      sceneId: string;
      entityId: string;
      offset: {
        x: number;
        y: number;
      };
    }
  | {
      type: "SetAttachedLocalOffset2D";
      sceneId: string;
      entityId: string;
      localOffset: {
        x: number;
        y: number;
      };
    };

export interface EditorCommandResultDto {
  ok: boolean;
  sceneDirty: boolean;
  changedEntities: string[];
  snapshot?: EditorSceneSnapshotDto;
  diagnostics: EditorDiagnosticDto[];
  message?: string;
}
