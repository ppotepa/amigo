import type { EditorProjectFileDto, ManagedAssetDto, RawAssetFileDto } from "../api/dto";

export function projectFileFromManagedAsset(asset: ManagedAssetDto): EditorProjectFileDto {
  return {
    name: asset.descriptorRelativePath.split("/").pop() ?? asset.assetId,
    path: asset.descriptorPath,
    relativePath: asset.descriptorRelativePath,
    kind: projectKindForManagedAsset(asset),
    isDir: false,
    sizeBytes: 0,
    children: [],
  };
}

export function projectFileFromRawAsset(file: RawAssetFileDto): EditorProjectFileDto {
  return {
    name: file.relativePath.split("/").pop() ?? file.relativePath,
    path: file.path,
    relativePath: file.relativePath,
    kind: file.mediaType.startsWith("image/") ? "rawImage" : "unknown",
    isDir: false,
    sizeBytes: 0,
    children: [],
  };
}

export function managedAssetFromProjectFile(modId: string, file: EditorProjectFileDto): ManagedAssetDto {
  const descriptorRelativePath = normalizeAssetPath(file.relativePath);
  const kind = managedAssetKindFromProjectFile(file);
  const parentKey = parentKeyFromProjectDescriptor(modId, descriptorRelativePath);
  const assetKey = assetKeyFromProjectDescriptor(modId, descriptorRelativePath);
  const assetId = assetKey.split("/").slice(1).join("/") || file.name;
  return {
    assetId,
    kind,
    label: assetId.split("/").pop() ?? assetId,
    assetKey,
    parentKey,
    references: [],
    usedBy: [],
    domain: domainForManagedAssetKind(kind),
    role: parentKey ? "subasset" : kind === "script" ? "file" : "family",
    descriptorPath: file.path,
    descriptorRelativePath: file.relativePath,
    sourceFiles: [],
    status: "valid",
    diagnostics: [],
  };
}

export function assetKeyFromProjectDescriptor(modId: string, descriptorRelativePath: string): string {
  const normalized = normalizeAssetPath(descriptorRelativePath);
  const withoutExtension = normalized.replace(/\.ya?ml$/i, "").replace(/\.rhai$/i, "");
  const parts = normalized.split("/");
  if (parts[0] === "spritesheets" && parts.length >= 3) {
    if (parts[2] === "spritesheet.yml" || parts[2] === "spritesheet.yaml") {
      return `${modId}/spritesheets/${parts[1]}`;
    }
    return `${modId}/${withoutExtension}`;
  }
  if (parts[0] === "fonts" && parts.length === 3 && parts[2] === "font.yml") {
    return `${modId}/fonts/${parts[1]}`;
  }
  if (parts[0] === "audio" && parts.length === 3 && parts[2] === "audio.yml") {
    return `${modId}/audio/${parts[1]}`;
  }
  if (parts[0] === "data" && parts[1] === "tilemaps" && /\.tilemap\.ya?ml$/i.test(normalized)) {
    return `${modId}/${withoutExtension.replace(/\.tilemap$/i, "")}`;
  }
  if (parts[0] === "scenes" && parts.length >= 3) {
    if (parts[2] === "scene.yml" || parts[2] === "scene.yaml") {
      return `${modId}/scenes/${parts[1]}`;
    }
    if (normalized.endsWith(".rhai")) {
      return `${modId}/scenes/${parts[1]}/scripts/${parts.slice(2).join("/").replace(/\.rhai$/i, "")}`;
    }
  }
  if (parts[0] === "scripts" && normalized.endsWith(".rhai")) {
    return `${modId}/${withoutExtension}`;
  }
  if (parts[0] === "packages" && (normalized.endsWith("/package.yml") || normalized.endsWith("/package.yaml"))) {
    return `${modId}/packages/${parts[1]}`;
  }
  if (parts[0] === "assets") {
    const area = parts[1] ?? "assets";
    const fileName = parts[parts.length - 1] ?? "asset";
    const assetId = fileName.replace(/\.(image|sprite|atlas|tileset|tile-ruleset|tilemap)\.ya?ml$/i, "");
    return `${modId}/${area}/${assetId}`;
  }
  return `${modId}/${withoutExtension}`;
}

export function projectKindForManagedAsset(asset: ManagedAssetDto): string {
  if (asset.kind === "image-2d") return "imageAsset";
  if (asset.kind === "tileset-2d") return "tileset";
  if (asset.kind === "tile-ruleset-2d") return "yaml";
  if (asset.kind === "tilemap-2d") return "tilemap";
  if (asset.kind === "audio") return "audio";
  if (asset.kind === "font-2d") return "font";
  if (asset.kind === "scene") return "sceneDocument";
  if (asset.kind === "script") return "script";
  if (asset.kind === "sprite-sheet-2d" || asset.kind === "spritesheet-2d") return "spritesheet";
  return "yaml";
}

function managedAssetKindFromProjectFile(file: EditorProjectFileDto): string {
  if (file.kind === "imageAsset") return "image-2d";
  if (file.kind === "audio") return "audio";
  if (file.kind === "font") return "font-2d";
  if (file.kind === "sceneDocument") return "scene";
  if (file.kind === "sceneScript" || file.kind === "script" || file.kind === "scriptPackage") return "script";
  if (file.kind === "spritesheet") return "spritesheet-2d";
  if (file.kind === "tilemap") return "tilemap-2d";
  if (file.kind === "tileset") {
    const normalized = normalizeAssetPath(file.relativePath).toLowerCase();
    if (
      normalized.endsWith(".tile-ruleset.yml") ||
      normalized.endsWith(".tile-ruleset.yaml") ||
      isSpritesheetSubassetPath(normalized, "rulesets")
    ) {
      return "tile-ruleset-2d";
    }
    return "tileset-2d";
  }
  return "yaml";
}

function domainForManagedAssetKind(kind: string): ManagedAssetDto["domain"] {
  if (kind === "tilemap-2d") return "tilemap";
  if (kind === "audio") return "audio";
  if (kind === "font-2d") return "font";
  if (kind === "scene") return "scene";
  if (kind === "script") return "script";
  if (kind === "spritesheet-2d" || kind === "tileset-2d" || kind === "tile-ruleset-2d") return "spritesheet";
  return "raw";
}

function parentKeyFromProjectDescriptor(modId: string, descriptorRelativePath: string): string | null {
  const parts = normalizeAssetPath(descriptorRelativePath).split("/");
  if (parts[0] === "spritesheets" && parts.length >= 4 && ["tilesets", "rulesets", "animations"].includes(parts[2])) {
    return `${modId}/spritesheets/${parts[1]}`;
  }
  if (parts[0] === "scenes" && parts.length >= 3 && parts[2] !== "scene.yml" && parts[2] !== "scene.yaml") {
    return `${modId}/scenes/${parts[1]}`;
  }
  return null;
}

function isSpritesheetSubassetPath(normalizedPath: string, subfolder: "tilesets" | "rulesets"): boolean {
  return normalizedPath.startsWith("spritesheets/") && normalizedPath.includes(`/${subfolder}/`);
}

function normalizeAssetPath(path: string): string {
  return path.replace(/\\/g, "/").replace(/^\/+/, "");
}
