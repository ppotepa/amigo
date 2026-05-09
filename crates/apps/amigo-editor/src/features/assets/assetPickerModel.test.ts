import { describe, expect, it } from "vitest";
import type { AssetRegistryDto } from "../../api/dto";
import {
  filterPickableAssets,
  pickableAssetsFromRegistry,
} from "./assetPickerModel";

const registry = {
  sessionId: "session",
  modId: "mod",
  rootPath: "D:/mod",
  managedAssets: [
    {
      assetKey: "mod/sprites/hero",
      assetId: "hero",
      label: "Hero",
      kind: "spritesheet",
      domain: "spritesheet",
      descriptorRelativePath: "assets/hero.asset.yml",
      descriptorPath: "D:/mod/assets/hero.asset.yml",
      references: [],
      usedBy: [],
      role: "family",
      sourceFiles: [],
      status: "valid",
      diagnostics: [],
    },
  ],
  rawFiles: [
    {
      path: "D:/mod/assets/raw.png",
      relativePath: "assets/raw.png",
      mediaType: "image/png",
      referencedBy: [],
      orphan: false,
    },
  ],
  diagnostics: [],
} as AssetRegistryDto;

describe("assetPickerModel", () => {
  it("creates pickable assets from managed and raw files", () => {
    const assets = pickableAssetsFromRegistry(registry);

    expect(assets.map((asset) => asset.key)).toEqual(
      expect.arrayContaining(["mod/sprites/hero", "assets/raw.png"]),
    );
  });

  it("filters by query and domain", () => {
    const assets = pickableAssetsFromRegistry(registry);

    expect(filterPickableAssets(assets, "hero", "spritesheet").map((asset) => asset.key))
      .toEqual(["mod/sprites/hero"]);
    expect(filterPickableAssets(assets, "hero", "audio")).toEqual([]);
  });
});
