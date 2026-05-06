import type {
  EditorDiagnosticDto,
  EditorProjectFileDto,
  EditorSceneEntityDto,
  EditorSceneSummaryDto,
  ManagedAssetDto,
  RawAssetFileDto,
} from "../../../api/dto";
import { findProjectFile, flattenProjectFiles, normalizePath } from "../../files/fileTreeSelectors";
import { isScriptProjectFile, sceneScriptSource } from "../../files/scriptSourceRefs";
import { relativeProjectPath, sceneYamlSource } from "../../files/yamlSourceRefs";
import type {
  SceneAssetGroup,
  SceneEntityNode,
  SceneScriptRef,
  SceneSourceModel,
} from "./sceneContextTypes";

const ASSET_GROUP_LABELS: Record<string, string> = {
  spritesheet: "Spritesheets",
  tilemap: "Tilemaps",
  audio: "Audio",
  font: "Fonts",
  scene: "Scenes",
  script: "Scripts",
  raw: "Raw Files",
};

const ASSET_GROUP_ORDER = [
  "spritesheet",
  "tilemap",
  "audio",
  "font",
  "raw",
  "script",
  "scene",
  "other",
];

export function selectSceneScripts(
  root: EditorProjectFileDto | undefined,
  scene: EditorSceneSummaryDto,
): SceneScriptRef[] {
  if (!root) return [];

  const allScripts = flattenProjectFiles(root).filter(isScriptProjectFile);
  const primary = selectSceneScriptFile(root, scene);
  const sceneDir = sceneDirectory(scene);

  const related = allScripts
    .map((file): SceneScriptRef | null => {
      const relative = normalizePath(file.relativePath);

      if (primary && file.relativePath === primary.relativePath) {
        return {
          id: file.relativePath,
          role: "primary",
          file,
        };
      }

      if (relative === "scripts/mod.rhai" || relative.endsWith("/scripts/mod.rhai")) {
        return {
          id: file.relativePath,
          role: "mod",
          file,
        };
      }

      if (relative.startsWith(`${sceneDir}/`) && relative.endsWith(".rhai")) {
        return {
          id: file.relativePath,
          role: "component",
          file,
        };
      }

      return null;
    })
    .filter((entry): entry is SceneScriptRef => Boolean(entry));

  return uniqueScripts(related).sort((left, right) =>
    scriptRoleRank(left.role) - scriptRoleRank(right.role) ||
    left.file.relativePath.localeCompare(right.file.relativePath),
  );
}

export function selectSceneScriptFile(
  root: EditorProjectFileDto | undefined,
  scene: EditorSceneSummaryDto,
): EditorProjectFileDto | null {
  if (!root || !scene.scriptPath) return null;
  const relativePath = relativeProjectPath(scene.scriptPath);
  return findProjectFile(root, relativePath);
}

export function selectSceneAssetGroups(
  scene: EditorSceneSummaryDto,
  managedAssets: ManagedAssetDto[],
  rawFiles: RawAssetFileDto[],
): SceneAssetGroup[] {
  const sceneAsset = managedAssets.find((asset) => isSceneAsset(asset, scene)) ?? null;
  const sceneReferenceKeys = sceneReferenceKeySet(scene);
  const referencedAssetKeys = new Set(sceneAsset?.references ?? []);

  const relatedManaged = managedAssets.filter((asset) => {
    if (sceneAsset && asset.assetKey === sceneAsset.assetKey) return false;
    if (referencedAssetKeys.has(asset.assetKey) || referencedAssetKeys.has(asset.assetId)) return true;
    return asset.usedBy.some((value) => sceneReferenceKeys.has(normalizePath(value)));
  });

  const relatedRaw = rawFiles.filter((file) =>
    file.referencedBy.some((value) => sceneReferenceKeys.has(normalizePath(value))),
  );

  const groups = new Map<string, SceneAssetGroup>();

  for (const asset of relatedManaged) {
    const id = asset.domain || "other";
    const group = ensureAssetGroup(groups, id);
    group.managedAssets.push(asset);
    group.count += 1;
  }

  for (const file of relatedRaw) {
    const id = rawGroupId(file);
    const group = ensureAssetGroup(groups, id);
    group.rawFiles.push(file);
    group.count += 1;
  }

  return Array.from(groups.values())
    .filter((group) => group.count > 0)
    .sort((left, right) => groupRank(left.id) - groupRank(right.id) || left.label.localeCompare(right.label));
}

export function selectSceneEntityNodes(
  entities: EditorSceneEntityDto[],
  selectedEntityId: string | null | undefined,
): SceneEntityNode[] {
  return entities.map((entity) => ({
    entity,
    selected: entity.id === selectedEntityId,
  }));
}

export function selectSceneDiagnostics(
  scene: EditorSceneSummaryDto,
  hierarchyDiagnostics: EditorDiagnosticDto[] = [],
): EditorDiagnosticDto[] {
  return [...scene.diagnostics, ...hierarchyDiagnostics];
}

export function selectSceneSourceModel(scene: EditorSceneSummaryDto): SceneSourceModel {
  return {
    yaml: sceneYamlSource(scene),
    script: sceneScriptSource(scene),
    folderPath: sceneDirectory(scene),
  };
}

function ensureAssetGroup(groups: Map<string, SceneAssetGroup>, id: string): SceneAssetGroup {
  const existing = groups.get(id);
  if (existing) return existing;

  const group: SceneAssetGroup = {
    id,
    label: ASSET_GROUP_LABELS[id] ?? titleCase(id),
    count: 0,
    managedAssets: [],
    rawFiles: [],
  };
  groups.set(id, group);
  return group;
}

function isSceneAsset(asset: ManagedAssetDto, scene: EditorSceneSummaryDto): boolean {
  const documentPath = relativeProjectPath(scene.documentPath);
  return asset.domain === "scene" && (
    normalizePath(asset.descriptorRelativePath) === documentPath ||
    normalizePath(asset.descriptorPath).endsWith(documentPath) ||
    asset.assetId === scene.id ||
    asset.assetKey.endsWith(`/scenes/${scene.id}`) ||
    asset.assetKey.endsWith(`/scenes/${scene.path}`)
  );
}

function sceneReferenceKeySet(scene: EditorSceneSummaryDto): Set<string> {
  return new Set([
    scene.id,
    scene.path,
    scene.label,
    relativeProjectPath(scene.path),
    relativeProjectPath(scene.documentPath),
    relativeProjectPath(scene.scriptPath),
  ].map((value) => normalizePath(value ?? "")).filter(Boolean));
}

function sceneDirectory(scene: EditorSceneSummaryDto): string {
  const documentPath = relativeProjectPath(scene.documentPath);
  return normalizePath(documentPath).replace(/\/scene\.ya?ml$/i, "");
}

function rawGroupId(file: RawAssetFileDto): string {
  if (file.mediaType.startsWith("image/")) return "spritesheet";
  if (file.mediaType.startsWith("audio/")) return "audio";
  return "raw";
}

function groupRank(id: string): number {
  const index = ASSET_GROUP_ORDER.indexOf(id);
  return index >= 0 ? index : ASSET_GROUP_ORDER.length;
}

function scriptRoleRank(role: SceneScriptRef["role"]): number {
  return role === "primary" ? 0 : role === "component" ? 1 : role === "mod" ? 2 : 3;
}

function uniqueScripts(scripts: SceneScriptRef[]): SceneScriptRef[] {
  const map = new Map<string, SceneScriptRef>();
  for (const script of scripts) map.set(script.file.relativePath, script);
  return Array.from(map.values());
}

function titleCase(value: string): string {
  return value.slice(0, 1).toUpperCase() + value.slice(1);
}
