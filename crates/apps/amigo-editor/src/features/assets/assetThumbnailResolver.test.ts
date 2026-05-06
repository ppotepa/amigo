import { describe, expect, it } from "vitest";
import { vi } from "vitest";
import type {
  EditorSceneSummaryDto,
  ManagedAssetDto,
  RawAssetFileDto,
} from "../../api/dto";
import {
  resolveManagedAssetThumbnail,
  resolveRawAssetThumbnail,
  resolveSceneThumbnail,
} from "./assetThumbnailResolver";

vi.mock("../../utils/fileSrc", () => ({
  fileSrc: (path: string) => `asset://${path}`,
}));

describe("assetThumbnailResolver", () => {
  it("resolves managed image-like assets from the first existing image source", () => {
    const thumbnail = resolveManagedAssetThumbnail(asset({
      assetKey: "sprites/hero",
      label: "Hero",
      kind: "spritesheet-2d",
      sourceFiles: [
        {
          path: "/project/mods/demo/spritesheets/hero/spritesheet.yml",
          relativePath: "spritesheets/hero/spritesheet.yml",
          exists: true,
          role: "descriptor",
        },
        {
          path: "/project/mods/demo/spritesheets/hero/hero.png",
          relativePath: "spritesheets/hero/hero.png",
          exists: true,
          role: "source",
        },
      ],
    }));

    expect(thumbnail.status).toBe("ready");
    expect(thumbnail.mode).toBe("pixel");
    expect(thumbnail.src).toContain("/project/mods/demo/spritesheets/hero/hero.png");
  });

  it("ignores missing managed image sources and falls back to icon rendering", () => {
    const thumbnail = resolveManagedAssetThumbnail(asset({
      assetKey: "sprites/missing",
      label: "Missing",
      kind: "image-2d",
      status: "missingSource",
      sourceFiles: [
        {
          path: "/project/mods/demo/sprites/missing.png",
          relativePath: "sprites/missing.png",
          exists: false,
          role: "source",
        },
      ],
    }));

    expect(thumbnail.status).toBe("missing");
    expect(thumbnail.src).toBeUndefined();
  });

  it("resolves raw image files directly", () => {
    const thumbnail = resolveRawAssetThumbnail(rawAsset({
      relativePath: "raw/splash.webp",
      mediaType: "image/webp",
      orphan: true,
    }));

    expect(thumbnail.status).toBe("ready");
    expect(thumbnail.mode).toBe("pixel");
    expect(thumbnail.src).toContain("/mod/raw/splash.webp");
  });

  it("falls back for non-image raw files", () => {
    const thumbnail = resolveRawAssetThumbnail(rawAsset({
      relativePath: "audio/jump.wav",
      mediaType: "audio/wav",
      orphan: false,
    }));

    expect(thumbnail.status).toBe("fallback");
    expect(thumbnail.src).toBeUndefined();
  });

  it("uses already-generated scene preview image URLs", () => {
    const thumbnail = resolveSceneThumbnail(scene({
      id: "menu",
      label: "Menu",
      previewImageUrl: "asset://scene-preview/menu.png",
    }));

    expect(thumbnail.status).toBe("ready");
    expect(thumbnail.mode).toBe("cover");
    expect(thumbnail.src).toBe("asset://scene-preview/menu.png");
  });
});

function asset(overrides: Partial<ManagedAssetDto> & Pick<ManagedAssetDto, "assetKey" | "label" | "kind">): ManagedAssetDto {
  const { assetKey, kind, label, ...rest } = overrides;
  return {
    assetId: rest.assetId ?? assetKey.split("/").pop() ?? "asset",
    assetKey,
    kind,
    label,
    parentKey: null,
    references: [],
    usedBy: [],
    domain: inferDomain(kind),
    role: "family",
    descriptorPath: `/mod/${assetKey}.yml`,
    descriptorRelativePath: `${assetKey}.yml`,
    sourceFiles: [],
    status: "valid",
    diagnostics: [],
    ...rest,
  };
}

function rawAsset(overrides: Partial<RawAssetFileDto> & Pick<RawAssetFileDto, "relativePath" | "mediaType" | "orphan">): RawAssetFileDto {
  return {
    path: `/mod/${overrides.relativePath}`,
    width: null,
    height: null,
    referencedBy: [],
    ...overrides,
  };
}

function scene(overrides: Partial<EditorSceneSummaryDto> & Pick<EditorSceneSummaryDto, "id" | "label">): EditorSceneSummaryDto {
  const { id, label, ...rest } = overrides;
  return {
    id,
    label,
    description: null,
    path: `scenes/${id}`,
    documentPath: `scenes/${id}/scene.yml`,
    scriptPath: "",
    launcherVisible: true,
    status: "valid",
    previewCacheKey: id,
    previewFps: 60,
    diagnostics: [],
    previewImageUrl: null,
    ...rest,
  };
}

function inferDomain(kind: string): ManagedAssetDto["domain"] {
  if (kind.includes("tilemap")) return "tilemap";
  if (kind.includes("audio")) return "audio";
  if (kind.includes("font")) return "font";
  if (kind.includes("scene")) return "scene";
  if (kind.includes("script")) return "script";
  return "spritesheet";
}
