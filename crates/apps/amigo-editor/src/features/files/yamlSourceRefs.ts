import type {
  EditorProjectFileDto,
  EditorSceneSummaryDto,
  ManagedAssetDto,
} from "../../api/dto";
import { findProjectFile, normalizePath } from "./fileTreeSelectors";

export type YamlSourceRef = {
  label: string;
  path: string;
  title?: string;
};

export function isYamlPath(path: string | null | undefined): boolean {
  return /\.ya?ml$/i.test(path ?? "");
}

const PROJECT_PATH_PREFIXES = [
  "scenes/",
  "raw/",
  "spritesheets/",
  "audio/",
  "fonts/",
  "scripts/",
  "data/",
  "docs/",
  "custom/",
  "packages/",
];

export function relativeProjectPath(path: string | null | undefined): string {
  const normalized = normalizePath(path ?? "");
  for (const prefix of PROJECT_PATH_PREFIXES) {
    const index = normalized.indexOf(prefix);
    if (index >= 0) {
      return normalized.slice(index);
    }
  }
  return normalized;
}

export function sceneYamlSource(
  scene: EditorSceneSummaryDto | null | undefined,
): YamlSourceRef | null {
  if (!scene?.documentPath || !isYamlPath(scene.documentPath)) return null;

  return {
    label: "Scene YAML",
    path: relativeProjectPath(scene.documentPath),
    title: scene.documentPath,
  };
}

export function assetYamlSource(
  asset: ManagedAssetDto | null | undefined,
): YamlSourceRef | null {
  if (!asset?.descriptorRelativePath || !isYamlPath(asset.descriptorRelativePath)) return null;

  return {
    label: "Asset YAML",
    path: relativeProjectPath(asset.descriptorRelativePath),
    title: asset.descriptorPath,
  };
}

export function projectFileYamlSource(
  file: EditorProjectFileDto | null | undefined,
): YamlSourceRef | null {
  if (!file || !isYamlProjectFile(file)) return null;

  return {
    label: "YAML",
    path: file.relativePath,
    title: file.path,
  };
}

export function isYamlProjectFile(file: EditorProjectFileDto): boolean {
  return /\.ya?ml$/i.test(file.name);
}

export function findYamlSourceFile(
  root: EditorProjectFileDto | undefined,
  source: YamlSourceRef | null | undefined,
): EditorProjectFileDto | null {
  if (!root || !source?.path) return null;

  const normalized = relativeProjectPath(source.path);
  if (!isYamlPath(normalized)) return null;

  const file = findProjectFile(root, normalized);
  if (!file || !isYamlProjectFile(file)) return null;
  return file;
}
