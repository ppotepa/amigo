import type {
  AssetRegistryDto,
  EditorDiagnosticDto,
  EditorProjectFileDto,
  EditorProjectTreeDto,
  EditorSceneSummaryDto,
  ManagedAssetDto,
  RawAssetFileDto,
} from "../../api/dto";
import { findProjectFile, flattenProjectFiles, normalizePath } from "../files/fileTreeSelectors";
import { findYamlSourceFile, relativeProjectPath, sceneYamlSource } from "../files/yamlSourceRefs";

export type SceneContextTab = "scripts" | "assets" | "entities" | "diagnostics" | "source";

export function sceneYamlFile(
  projectTree: EditorProjectTreeDto | undefined,
  scene: EditorSceneSummaryDto | null | undefined,
): EditorProjectFileDto | null {
  if (!projectTree?.root || !scene) return null;
  return findYamlSourceFile(projectTree.root, sceneYamlSource(scene));
}

export function sceneScriptFile(
  projectTree: EditorProjectTreeDto | undefined,
  scene: EditorSceneSummaryDto | null | undefined,
): EditorProjectFileDto | null {
  if (!projectTree?.root || !scene?.scriptPath) return null;

  const relative = relativeProjectPath(scene.scriptPath);
  if (!/\.rhai$/i.test(relative)) return null;

  const file = findProjectFile(projectTree.root, relative);
  if (!file || !isScriptFile(file)) return null;
  return file;
}

export function sceneRelatedScripts(
  projectTree: EditorProjectTreeDto | undefined,
  scene: EditorSceneSummaryDto | null | undefined,
): EditorProjectFileDto[] {
  if (!projectTree?.root || !scene) return [];

  const primary = sceneScriptFile(projectTree, scene);
  const sceneDir = sceneDirectory(scene);
  const allScripts = flattenProjectFiles(projectTree.root).filter(isScriptFile);
  const related = allScripts.filter((file) => {
    const relative = normalizePath(file.relativePath);
    return (
      file.relativePath === primary?.relativePath ||
      relative.startsWith(`${sceneDir}/`) ||
      relative === "scripts/mod.rhai" ||
      relative === "script/mod.rhai"
    );
  });

  return uniqueFiles([primary, ...related]);
}

export function sceneRelatedManagedAssets(
  registry: AssetRegistryDto | null | undefined,
  scene: EditorSceneSummaryDto | null | undefined,
): ManagedAssetDto[] {
  if (!registry || !scene) return [];

  const sceneAsset = sceneManagedAsset(registry, scene);
  const referencedKeys = new Set(sceneAsset?.references ?? []);

  return registry.managedAssets
    .filter((asset) => {
      if (asset.assetKey === sceneAsset?.assetKey) return false;
      if (referencedKeys.has(asset.assetKey) || referencedKeys.has(asset.assetId)) return true;
      return referencesScene(asset.usedBy, scene);
    })
    .sort((left, right) => left.label.localeCompare(right.label) || left.assetKey.localeCompare(right.assetKey));
}

export function sceneRelatedRawFiles(
  registry: AssetRegistryDto | null | undefined,
  scene: EditorSceneSummaryDto | null | undefined,
): RawAssetFileDto[] {
  if (!registry || !scene) return [];

  return registry.rawFiles
    .filter((file) => referencesScene(file.referencedBy, scene))
    .sort((left, right) => left.relativePath.localeCompare(right.relativePath));
}

export function sceneDiagnostics(scene: EditorSceneSummaryDto | null | undefined): EditorDiagnosticDto[] {
  return scene?.diagnostics ?? [];
}

export function isScriptFile(file: EditorProjectFileDto): boolean {
  return file.kind === "script" || file.kind === "sceneScript" || file.kind === "scriptPackage" || /\.rhai$/i.test(file.name);
}

function sceneManagedAsset(
  registry: AssetRegistryDto,
  scene: EditorSceneSummaryDto,
): ManagedAssetDto | null {
  const documentPath = relativeProjectPath(scene.documentPath);

  return registry.managedAssets.find((asset) => (
    asset.domain === "scene" &&
    (
      normalizePath(asset.descriptorRelativePath) === documentPath ||
      normalizePath(asset.descriptorPath).endsWith(documentPath) ||
      asset.assetId === scene.id ||
      asset.assetKey.endsWith(`/scenes/${scene.id}`) ||
      asset.assetKey.endsWith(`/scenes/${scene.path}`)
    )
  )) ?? null;
}

function referencesScene(values: string[], scene: EditorSceneSummaryDto): boolean {
  const keys = sceneReferenceKeys(scene);
  return values.some((value) => {
    const normalized = normalizePath(value);
    return keys.some((key) => normalized === key || normalized.endsWith(key) || key.endsWith(normalized));
  });
}

function sceneReferenceKeys(scene: EditorSceneSummaryDto): string[] {
  return [
    scene.id,
    scene.path,
    scene.label,
    relativeProjectPath(scene.path),
    relativeProjectPath(scene.documentPath),
    relativeProjectPath(scene.scriptPath),
  ]
    .map((value) => normalizePath(value ?? ""))
    .filter(Boolean);
}

function sceneDirectory(scene: EditorSceneSummaryDto): string {
  const documentPath = relativeProjectPath(scene.documentPath);
  return normalizePath(documentPath).replace(/\/scene\.ya?ml$/i, "");
}

function uniqueFiles(files: Array<EditorProjectFileDto | null | undefined>): EditorProjectFileDto[] {
  const result = new Map<string, EditorProjectFileDto>();
  for (const file of files) {
    if (file) result.set(file.relativePath, file);
  }
  return Array.from(result.values()).sort((left, right) => left.relativePath.localeCompare(right.relativePath));
}
