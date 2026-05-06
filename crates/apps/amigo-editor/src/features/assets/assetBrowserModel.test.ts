import { describe, expect, it } from "vitest";
import type { ManagedAssetDto, RawAssetFileDto } from "../../api/dto";
import {
  deriveAssetBrowserState,
  filterManagedAssets,
  filterRawAssets,
  groupManagedAssets,
  summarizeVisibleAssets,
} from "./assetBrowserModel";

describe("assetBrowserModel", () => {
  it("filters managed assets by issues, search and kind", () => {
    const managed = [
      asset({ assetKey: "sprites/hero", label: "Hero", kind: "sprite-sheet-2d", status: "warning" }),
      asset({ assetKey: "images/logo", label: "Logo", kind: "image-2d", status: "valid" }),
      asset({ assetKey: "sprites/villain", label: "Villain", kind: "sprite-sheet-2d", status: "valid" }),
    ];

    const filtered = filterManagedAssets(managed, {
      search: "hero",
      kindFilter: "sprite-sheet-2d",
      issuesOnly: true,
    });

    expect(filtered.map((entry) => entry.assetKey)).toEqual(["sprites/hero"]);
  });

  it("filters raw files by kind, orphan state and search", () => {
    const raw = [
      rawAsset({ relativePath: "raw/images/hero.png", mediaType: "image/png", orphan: true }),
      rawAsset({ relativePath: "raw/images/logo.png", mediaType: "image/png", orphan: false }),
      rawAsset({ relativePath: "raw/audio/theme.ogg", mediaType: "audio/ogg", orphan: true }),
    ];

    const filtered = filterRawAssets(raw, {
      search: "hero",
      kindFilter: "image-2d",
      issuesOnly: true,
    });

    expect(filtered.map((entry) => entry.relativePath)).toEqual(["raw/images/hero.png"]);
  });

  it("groups assets in stable visual order and sorts entries by label", () => {
    const grouped = groupManagedAssets([
      asset({ assetKey: "tilemaps/arena", label: "Arena", kind: "tilemap-2d" }),
      asset({ assetKey: "images/zeta", label: "Zeta", kind: "image-2d" }),
      asset({ assetKey: "images/alpha", label: "Alpha", kind: "image-2d" }),
    ]);

    expect(Array.from(grouped.keys())).toEqual(["image-2d", "tilemap-2d"]);
    expect(grouped.get("image-2d")?.map((entry) => entry.label)).toEqual(["Alpha", "Zeta"]);
  });

  it("keeps missing ancestors in tree state for filtered descendants", () => {
    const parent = asset({ assetKey: "sprites/hero", label: "Hero", kind: "sprite-sheet-2d" });
    const child = asset({
      assetKey: "sprites/hero/rulesets/run",
      assetId: "run",
      label: "Run",
      kind: "tile-ruleset-2d",
      parentKey: "sprites/hero",
    });

    const state = deriveAssetBrowserState([parent, child], [], {
      search: "run",
      kindFilter: "all",
      issuesOnly: false,
    });

    expect(state.filteredManaged.map((entry) => entry.assetKey)).toEqual(["sprites/hero/rulesets/run"]);
    expect(state.treeManaged.map((entry) => entry.assetKey)).toEqual(["sprites/hero", "sprites/hero/rulesets/run"]);
  });

  it("summarizes visible counts for the search banner", () => {
    expect(summarizeVisibleAssets(3, 0)).toBe("Showing 3 managed");
    expect(summarizeVisibleAssets(3, 2)).toBe("Showing 3 managed · 2 raw");
    expect(summarizeVisibleAssets(0, 0, 1)).toBe("Showing 0 managed · 1 scripts");
  });
});

function asset(overrides: Partial<ManagedAssetDto> & Pick<ManagedAssetDto, "assetKey" | "label" | "kind">): ManagedAssetDto {
  return {
    assetId: overrides.assetId ?? overrides.assetKey.split("/").pop() ?? "asset",
    parentKey: null,
    references: [],
    usedBy: [],
    domain: inferDomain(overrides.kind),
    role: "family",
    descriptorPath: `/mod/${overrides.assetKey}.yml`,
    descriptorRelativePath: `${overrides.assetKey}.yml`,
    sourceFiles: [],
    status: "valid",
    diagnostics: [],
    ...overrides,
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

function inferDomain(kind: string): ManagedAssetDto["domain"] {
  if (kind.includes("tilemap")) return "tilemap";
  if (kind.includes("audio")) return "audio";
  if (kind.includes("font")) return "font";
  if (kind.includes("scene")) return "scene";
  if (kind.includes("script")) return "script";
  return "spritesheet";
}
