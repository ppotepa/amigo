import type {
  AssetRegistryDto,
  ManagedAssetDto,
  RawAssetFileDto,
} from "../../api/dto";

export type PickableAsset = {
  key: string;
  label: string;
  kind: string;
  path?: string;
  domain?: string | null;
};

export function pickableAssetsFromRegistry(
  registry: AssetRegistryDto | null | undefined,
): PickableAsset[] {
  if (!registry) return [];

  return [
    ...registry.managedAssets.map(managedPickableAsset),
    ...registry.rawFiles.map(rawPickableAsset),
  ].sort((left, right) => left.label.localeCompare(right.label));
}

export function filterPickableAssets(
  assets: PickableAsset[],
  query: string,
  domain?: string | null,
): PickableAsset[] {
  const normalizedQuery = query.trim().toLowerCase();
  const normalizedDomain = domain?.trim().toLowerCase() || null;

  return assets.filter((asset) => {
    if (normalizedDomain && asset.domain?.toLowerCase() !== normalizedDomain) {
      return false;
    }

    if (!normalizedQuery) return true;
    return [
      asset.key,
      asset.label,
      asset.kind,
      asset.path ?? "",
      asset.domain ?? "",
    ].join(" ").toLowerCase().includes(normalizedQuery);
  });
}

function managedPickableAsset(asset: ManagedAssetDto): PickableAsset {
  return {
    key: asset.assetKey,
    label: asset.label || asset.assetId || asset.assetKey,
    kind: asset.kind,
    path: asset.descriptorRelativePath || asset.descriptorPath,
    domain: asset.domain,
  };
}

function rawPickableAsset(asset: RawAssetFileDto): PickableAsset {
  return {
    key: asset.relativePath || asset.path,
    label: asset.relativePath || asset.path,
    kind: asset.mediaType || "raw",
    path: asset.relativePath || asset.path,
    domain: "raw",
  };
}
