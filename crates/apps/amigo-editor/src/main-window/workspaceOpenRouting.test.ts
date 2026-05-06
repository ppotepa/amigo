import { describe, expect, it } from "vitest";
import type {
  EditorModDetailsDto,
  EditorProjectTreeDto,
  ManagedAssetDto,
} from "../api/dto";
import {
  resolveManagedAssetOpenRequest,
  sceneForManagedAsset,
} from "./workspaceOpenRouting";

describe("workspaceOpenRouting", () => {
  it("routes scene managed assets to scene editor requests", () => {
    const request = resolveManagedAssetOpenRequest({
      asset: managedSceneAsset("main-menu"),
      details: modDetails(),
      projectTree: undefined,
    });

    expect(request.kind).toBe("scene");
    if (request.kind === "scene") {
      expect(request.scene.id).toBe("main-menu");
    }
  });

  it("finds scene by scene asset id", () => {
    const scene = sceneForManagedAsset(modDetails(), managedSceneAsset("main-menu"));
    expect(scene?.id).toBe("main-menu");
  });

  it("routes non-scene managed assets to project file requests", () => {
    const request = resolveManagedAssetOpenRequest({
      asset: managedImageAsset(),
      details: modDetails(),
      projectTree: undefined as EditorProjectTreeDto | undefined,
    });

    expect(request.kind).toBe("project-file");
    if (request.kind === "project-file") {
      expect(request.file.relativePath).toBe("images/logo/image.yml");
    }
  });
});

function modDetails(): EditorModDetailsDto {
  return {
    id: "they-are-rotten",
    name: "They Are Rotten",
    version: "0.1.0",
    description: "",
    authors: [],
    rootPath: "mods/they-are-rotten",
    dependencies: [],
    missingDependencies: [],
    capabilities: [],
    sceneCount: 1,
    visibleSceneCount: 1,
    status: "valid",
    diagnostics: [],
    previewStatus: "ready",
    contentSummary: {
      scenes: 1,
      sceneYaml: 1,
      scripts: 0,
      textures: 1,
      spritesheets: 0,
      audio: 0,
      fonts: 0,
      tilemaps: 0,
      tilesets: 0,
      packages: 0,
      unknownFiles: 0,
      totalFiles: 2,
    },
    scenes: [
      {
        id: "main-menu",
        label: "Main Menu",
        description: "",
        path: "scenes/main-menu",
        documentPath: "scenes/main-menu/scene.yml",
        scriptPath: "scenes/main-menu/scene.rhai",
        launcherVisible: true,
        status: "valid",
        previewCacheKey: "main-menu",
        previewFps: 0,
        diagnostics: [],
      },
    ],
  };
}

function managedSceneAsset(sceneId: string): ManagedAssetDto {
  return {
    assetKey: `they-are-rotten/scenes/${sceneId}`,
    assetId: sceneId,
    kind: "scene",
    domain: "scene",
    role: "family",
    label: "Main Menu",
    descriptorPath: `mods/they-are-rotten/scenes/${sceneId}/scene.yml`,
    descriptorRelativePath: `scenes/${sceneId}/scene.yml`,
    references: [],
    usedBy: [],
    sourceFiles: [],
    status: "valid",
    diagnostics: [],
  };
}

function managedImageAsset(): ManagedAssetDto {
  return {
    assetKey: "they-are-rotten/images/logo",
    assetId: "logo",
    kind: "image-2d",
    domain: "image",
    role: "family",
    label: "Logo",
    descriptorPath: "mods/they-are-rotten/images/logo/image.yml",
    descriptorRelativePath: "images/logo/image.yml",
    references: [],
    usedBy: [],
    sourceFiles: [],
    status: "valid",
    diagnostics: [],
  };
}
