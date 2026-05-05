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
  category: string;
  componentTypes: string[];
  /**
   * transform2 is the editor-space origin/pivot for 2D editing.
   * bounds2 is the 2D selection/edit bounds. It is not guaranteed to be pixel-perfect render bounds.
   * If bounds2 is missing, frontend must not allow local 2D hit-test for this object.
   */
  transform2?: EditorTransform2Dto;
  transform3?: EditorTransform3Dto;
  bounds2?: EditorBounds2Dto;
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
}

export interface EditorViewportPointDto {
  x: number;
  y: number;
}

export interface EditorHitTestCandidateDto {
  entityId: string;
  name: string;
  depth: number;
  bounds2?: EditorBounds2Dto;
}

export interface EditorHitTestResultDto {
  hit: boolean;
  entityId?: string;
  candidates: EditorHitTestCandidateDto[];
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
    };

export interface EditorCommandResultDto {
  ok: boolean;
  sceneDirty: boolean;
  changedEntities: string[];
  snapshot?: EditorSceneSnapshotDto;
  diagnostics: EditorDiagnosticDto[];
  message?: string;
}

export type SceneEditorModeKindDto =
  | "document"
  | "live";

export type EditorLiveSceneSessionStatusDto =
  | "opening"
  | "ready"
  | "dirty"
  | "saving"
  | "closed"
  | "failed";

export interface EditorLiveSceneSessionDto {
  editorSceneSessionId: string;
  editorSessionId: string;
  sceneId: string;
  mode: "live";
  status: EditorLiveSceneSessionStatusDto;
  dirty: boolean;
  revision: number;
  openedAtMs: number;
}

export interface OpenEditorLiveSceneSessionResultDto {
  session: EditorLiveSceneSessionDto;
  snapshot: EditorSceneSnapshotDto;
  diagnostics: EditorDiagnosticDto[];
}

export interface EditorLiveCommandResultDto {
  ok: boolean;
  session?: EditorLiveSceneSessionDto;
  snapshot?: EditorSceneSnapshotDto;
  diagnostics: EditorDiagnosticDto[];
  message?: string;
}
