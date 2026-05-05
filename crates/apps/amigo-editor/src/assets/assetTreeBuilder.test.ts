import { describe, expect, it } from "vitest";
import type { AssetRegistryDto, ManagedAssetDto } from "../api/dto";
import { buildAssetTree, type AssetTreeNode } from "./assetTreeBuilder";

describe("buildAssetTree", () => {
  it("builds a nested logical tree from a flat registry", () => {
    const tree = buildAssetTree(registry({
      managedAssets: [
        asset({
          assetKey: "test/spritesheets/dirt",
          assetId: "dirt",
          label: "Dirt",
          kind: "spritesheet-2d",
          domain: "spritesheet",
          role: "family",
          references: ["raw/images/dirt.png"],
        }),
        asset({
          assetKey: "test/spritesheets/dirt/tilesets/platform/base",
          assetId: "platform/base",
          label: "Base",
          kind: "tileset-2d",
          domain: "spritesheet",
          role: "subasset",
          parentKey: "test/spritesheets/dirt",
        }),
        asset({
          assetKey: "test/spritesheets/dirt/rulesets/platform/solid",
          assetId: "platform/solid",
          label: "Solid",
          kind: "tile-ruleset-2d",
          domain: "spritesheet",
          role: "subasset",
          parentKey: "test/spritesheets/dirt",
          references: ["test/spritesheets/dirt/tilesets/platform/base"],
        }),
        asset({
          assetKey: "test/data/tilemaps/level-01",
          assetId: "level-01",
          label: "Level 01",
          kind: "tilemap-2d",
          domain: "tilemap",
          role: "family",
          references: ["test/spritesheets/dirt/tilesets/platform/base"],
        }),
      ],
      rawFiles: [
        {
          path: "/mod/raw/images/dirt.png",
          relativePath: "raw/images/dirt.png",
          mediaType: "image/png",
          width: 16,
          height: 16,
          referencedBy: ["test/spritesheets/dirt"],
          orphan: false,
        },
      ],
    }));

    const spritesheets = child(tree, "Spritesheets");
    const dirt = child(spritesheets.children, "Dirt");
    expect(child(dirt.children, "Tilesets").children.some((node) => node.label === "Base")).toBe(true);
    expect(child(dirt.children, "Rulesets").children.some((node) => node.label === "Solid")).toBe(true);

    const tilemap = child(child(tree, "Tilemaps").children, "Level 01");
    expect(tilemap.children.some((node) => node.label === "References")).toBe(false);

    const raw = child(child(tree, "Raw Sources").children, "dirt.png");
    expect(raw.status).toBe("valid");
    expect(raw.children.some((node) => node.label === "Used By")).toBe(false);
  });

  it("moves assets with missing parents to Unknown with diagnostics", () => {
    const tree = buildAssetTree(registry({
      managedAssets: [
        asset({
          assetKey: "test/spritesheets/missing/rulesets/platform/solid",
          assetId: "platform/solid",
          label: "Solid",
          kind: "tile-ruleset-2d",
          domain: "spritesheet",
          role: "subasset",
          parentKey: "test/spritesheets/missing",
        }),
      ],
    }));

    const unknown = child(tree, "Unknown");
    const solid = child(unknown.children, "Solid");
    expect(solid.status).toBe("missing");
    expect(child(solid.children, "Diagnostics").children.some((node) => node.label === "asset_parent_missing")).toBe(true);
  });

});

function registry(overrides: Partial<AssetRegistryDto>): AssetRegistryDto {
  return {
    sessionId: "session",
    modId: "test",
    rootPath: "/mod",
    managedAssets: [],
    rawFiles: [],
    diagnostics: [],
    ...overrides,
  };
}

function asset(overrides: Partial<ManagedAssetDto> & Pick<ManagedAssetDto, "assetKey" | "assetId" | "label" | "kind" | "domain" | "role">): ManagedAssetDto {
  return {
    descriptorPath: `/mod/${overrides.assetKey}.yml`,
    descriptorRelativePath: `${overrides.assetKey}.yml`,
    sourceFiles: [],
    status: "valid",
    diagnostics: [],
    parentKey: null,
    references: [],
    usedBy: [],
    ...overrides,
  };
}

function child(nodes: AssetTreeNode[], label: string): AssetTreeNode {
  const node = nodes.find((candidate) => candidate.label === label);
  if (!node) {
    throw new Error(`Expected node ${label}`);
  }
  return node;
}
