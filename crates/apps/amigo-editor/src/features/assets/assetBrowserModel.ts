import type { ManagedAssetDto, RawAssetFileDto } from "../../api/dto";
import { assetVisualForKind } from "../../assets/assetVisualRegistry";

export interface AssetBrowserFilterOptions {
  search: string;
  kindFilter: string;
  issuesOnly: boolean;
}

export interface AssetBrowserDerivedState {
  filteredManaged: ManagedAssetDto[];
  filteredRaw: RawAssetFileDto[];
  groupedManaged: globalThis.Map<string, ManagedAssetDto[]>;
  treeManaged: ManagedAssetDto[];
}

export function deriveAssetBrowserState(
  managed: ManagedAssetDto[],
  raw: RawAssetFileDto[],
  options: AssetBrowserFilterOptions,
): AssetBrowserDerivedState {
  const filteredManaged = filterManagedAssets(managed, options);
  const filteredRaw = filterRawAssets(raw, options);
  return {
    filteredManaged,
    filteredRaw,
    groupedManaged: groupManagedAssets(filteredManaged),
    treeManaged: includeManagedAncestors(filteredManaged, managed),
  };
}

export function filterManagedAssets(
  managed: ManagedAssetDto[],
  options: AssetBrowserFilterOptions,
): ManagedAssetDto[] {
  return managed
    .filter((asset) => {
      if (options.issuesOnly && asset.status === "valid") return false;
      if (options.kindFilter !== "all" && asset.kind !== options.kindFilter) return false;
      return matchesSearch([asset.label, asset.assetKey, asset.descriptorRelativePath, asset.kind], options.search);
    })
    .sort(compareManagedAssets);
}

export function filterRawAssets(
  raw: RawAssetFileDto[],
  options: AssetBrowserFilterOptions,
): RawAssetFileDto[] {
  return raw
    .filter((file) => {
      if (options.kindFilter !== "all" && options.kindFilter !== "image-2d") return false;
      if (options.kindFilter === "image-2d" && !file.mediaType.startsWith("image/")) return false;
      if (options.issuesOnly && !file.orphan) return false;
      return matchesSearch([file.relativePath, file.mediaType, ...file.referencedBy], options.search);
    })
    .sort(compareRawAssets);
}

export function groupManagedAssets(assets: ManagedAssetDto[]): globalThis.Map<string, ManagedAssetDto[]> {
  const grouped = new globalThis.Map<string, ManagedAssetDto[]>();
  for (const asset of assets) {
    const list = grouped.get(asset.kind) ?? [];
    list.push(asset);
    grouped.set(asset.kind, list);
  }

  const sortedEntries = Array.from(grouped.entries())
    .map(([kind, groupAssets]) => [kind, [...groupAssets].sort(compareManagedAssets)] as const)
    .sort(([leftKind], [rightKind]) => compareGroupKinds(leftKind, rightKind));

  return new globalThis.Map(sortedEntries);
}

export function includeManagedAncestors(filtered: ManagedAssetDto[], all: ManagedAssetDto[]): ManagedAssetDto[] {
  const byKey = new globalThis.Map(all.map((asset) => [asset.assetKey, asset]));
  const result = new globalThis.Map(filtered.map((asset) => [asset.assetKey, asset]));
  for (const asset of filtered) {
    let parentKey = asset.parentKey ?? null;
    while (parentKey) {
      const parent = byKey.get(parentKey);
      if (!parent || result.has(parent.assetKey)) break;
      result.set(parent.assetKey, parent);
      parentKey = parent.parentKey ?? null;
    }
  }

  return Array.from(result.values()).sort(compareManagedAssets);
}

export function summarizeVisibleAssets(managedCount: number, rawCount: number, scriptCount = 0): string {
  const parts = [`${managedCount} managed`];
  if (rawCount > 0) {
    parts.push(`${rawCount} raw`);
  }
  if (scriptCount > 0) {
    parts.push(`${scriptCount} scripts`);
  }
  return `Showing ${parts.join(" · ")}`;
}

function compareManagedAssets(left: ManagedAssetDto, right: ManagedAssetDto): number {
  return (
    left.label.localeCompare(right.label, undefined, { sensitivity: "base" }) ||
    left.assetKey.localeCompare(right.assetKey, undefined, { sensitivity: "base" })
  );
}

function compareRawAssets(left: RawAssetFileDto, right: RawAssetFileDto): number {
  return left.relativePath.localeCompare(right.relativePath, undefined, { sensitivity: "base" });
}

function compareGroupKinds(leftKind: string, rightKind: string): number {
  const leftLabel = assetVisualForKind(leftKind).label;
  const rightLabel = assetVisualForKind(rightKind).label;
  return leftLabel.localeCompare(rightLabel, undefined, { sensitivity: "base" }) || leftKind.localeCompare(rightKind);
}

function matchesSearch(values: string[], search: string): boolean {
  const query = search.trim().toLowerCase();
  return !query || values.some((value) => value.toLowerCase().includes(query));
}
