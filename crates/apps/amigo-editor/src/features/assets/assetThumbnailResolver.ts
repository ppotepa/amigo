import type {
  EditorSceneSummaryDto,
  ManagedAssetDto,
  RawAssetFileDto,
} from "../../api/dto";
import type { FolderViewThumbnailMode } from "../../ui/folder-view/folderViewTypes";
import { fileSrc } from "../../utils/fileSrc";

export type AssetThumbnailStatus = "ready" | "fallback" | "missing";

export type AssetThumbnailDescriptor = {
  src?: string;
  mode: FolderViewThumbnailMode;
  status: AssetThumbnailStatus;
  reason?: string;
};

const IMAGE_SOURCE_RE = /\.(png|jpe?g|webp|gif|bmp|avif)$/i;

export function resolveManagedAssetThumbnail(asset: ManagedAssetDto): AssetThumbnailDescriptor {
  const directImage = firstExistingImageSource(asset);
  if (directImage) {
    return {
      src: fileSrc(directImage.path),
      mode: thumbnailModeForAssetKind(asset.kind),
      status: "ready",
    };
  }

  return {
    mode: thumbnailModeForAssetKind(asset.kind),
    status: asset.status === "missingSource" ? "missing" : "fallback",
    reason: "No existing image source was found for this asset.",
  };
}

export function resolveRawAssetThumbnail(file: RawAssetFileDto): AssetThumbnailDescriptor {
  if (isImageMediaType(file.mediaType)) {
    return {
      src: fileSrc(file.path),
      mode: "pixel",
      status: "ready",
    };
  }

  return {
    mode: "contain",
    status: "fallback",
    reason: "Raw file is not an image media type.",
  };
}

export function resolveSceneThumbnail(scene: EditorSceneSummaryDto): AssetThumbnailDescriptor {
  if (scene.previewImageUrl) {
    return {
      src: scene.previewImageUrl,
      mode: "cover",
      status: "ready",
    };
  }

  return {
    mode: "cover",
    status: "fallback",
    reason: "Scene preview has not been generated yet.",
  };
}

export function managedAssetHasThumbnail(asset: ManagedAssetDto): boolean {
  return resolveManagedAssetThumbnail(asset).status === "ready";
}

export function rawAssetHasThumbnail(file: RawAssetFileDto): boolean {
  return resolveRawAssetThumbnail(file).status === "ready";
}

function firstExistingImageSource(asset: ManagedAssetDto): ManagedAssetDto["sourceFiles"][number] | null {
  const exactImage = asset.sourceFiles.find((file) => file.exists && isImagePath(file.relativePath));
  if (exactImage) return exactImage;

  const textureReference = asset.sourceFiles.find((file) => {
    if (!file.exists) return false;
    const normalized = file.relativePath.toLowerCase();
    return (
      normalized.includes("/textures/") ||
      normalized.includes("/sprites/") ||
      normalized.includes("/spritesheets/") ||
      normalized.includes("/tilesets/") ||
      normalized.includes("/images/")
    ) && isImagePath(normalized);
  });

  return textureReference ?? null;
}

function thumbnailModeForAssetKind(kind: string): FolderViewThumbnailMode {
  const normalized = kind.toLowerCase();
  if (
    normalized.includes("sprite") ||
    normalized.includes("tileset") ||
    normalized.includes("tilemap") ||
    normalized.includes("image")
  ) {
    return "pixel";
  }

  if (
    normalized.includes("scene") ||
    normalized.includes("material") ||
    normalized.includes("mesh")
  ) {
    return "cover";
  }

  return "contain";
}

function isImageMediaType(mediaType: string): boolean {
  return mediaType.toLowerCase().startsWith("image/");
}

function isImagePath(path: string): boolean {
  return IMAGE_SOURCE_RE.test(path);
}
