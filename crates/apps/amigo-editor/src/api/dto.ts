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

export type AssetStatusDto = "valid" | "warning" | "error" | "missingSource";

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

export interface AssetMigrationEntryDto {
  action: string;
  fromPath?: string | null;
  toPath?: string | null;
  assetKind?: string | null;
  reason: string;
}

export interface AssetMigrationPlanDto {
  sessionId: string;
  modId: string;
  rootPath: string;
  entries: AssetMigrationEntryDto[];
}

export interface AssetMigrationResultDto {
  dryRun: boolean;
  appliedEntries: number;
  reportPath?: string | null;
}
