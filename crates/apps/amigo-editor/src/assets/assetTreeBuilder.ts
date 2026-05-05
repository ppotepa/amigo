import type {
  AssetRegistryDto,
  EditorDiagnosticDto,
  ManagedAssetDto,
  RawAssetFileDto,
} from "../api/dto";

export type AssetTreeStatus = "valid" | "warning" | "error" | "missing" | "orphan";

export type AssetTreeNode = {
  key: string;
  label: string;
  kind: string;
  role: string;
  children: AssetTreeNode[];
  status?: AssetTreeStatus;
  assetKey?: string;
  descriptorRelativePath?: string;
  asset?: ManagedAssetDto;
  rawFile?: RawAssetFileDto;
  references?: string[];
  usedBy?: string[];
  diagnostics?: EditorDiagnosticDto[];
};

type CategoryDefinition = {
  key: string;
  label: string;
  domain?: ManagedAssetDto["domain"];
  raw?: boolean;
};

const CATEGORY_DEFINITIONS: CategoryDefinition[] = [
  { key: "scenes", label: "Scenes", domain: "scene" },
  { key: "spritesheets", label: "Spritesheets", domain: "spritesheet" },
  { key: "tilemaps", label: "Tilemaps", domain: "tilemap" },
  { key: "audio", label: "Audio", domain: "audio" },
  { key: "fonts", label: "Fonts", domain: "font" },
  { key: "scripts", label: "Scripts", domain: "script" },
  { key: "raw", label: "Raw Sources", raw: true },
  { key: "unknown", label: "Unknown" },
];

const SUBASSET_BUCKETS = [
  { key: "tilesets", label: "Tilesets", predicate: (asset: ManagedAssetDto) => asset.kind.includes("tileset") && !asset.kind.includes("ruleset") },
  { key: "rulesets", label: "Rulesets", predicate: (asset: ManagedAssetDto) => asset.kind.includes("ruleset") },
  { key: "animations", label: "Animations", predicate: (asset: ManagedAssetDto) => asset.kind.includes("animation") },
  { key: "other", label: "Subassets", predicate: (_asset: ManagedAssetDto) => true },
];

export function buildAssetTree(registry: AssetRegistryDto): AssetTreeNode[] {
  const assetsByKey = new Map(registry.managedAssets.map((asset) => [asset.assetKey, asset]));
  const childrenByParent = new Map<string, ManagedAssetDto[]>();
  const rootsByDomain = new Map<ManagedAssetDto["domain"], ManagedAssetDto[]>();
  const unknownAssets: ManagedAssetDto[] = [];

  for (const asset of registry.managedAssets) {
    if (asset.parentKey) {
      if (assetsByKey.has(asset.parentKey)) {
        appendMapList(childrenByParent, asset.parentKey, asset);
      } else {
        unknownAssets.push(asset);
      }
      continue;
    }
    appendMapList(rootsByDomain, asset.domain, asset);
  }

  return CATEGORY_DEFINITIONS.map((category) => {
    const children = category.raw
      ? registry.rawFiles.map((file) => rawFileNode(file))
      : category.key === "unknown"
        ? unknownAssets.map((asset) => assetNode(asset, registry, childrenByParent, true))
        : (category.domain ? rootsByDomain.get(category.domain) ?? [] : [])
          .map((asset) => assetNode(asset, registry, childrenByParent, false));

    const sortedChildren = sortNodes(children);
    return {
      key: `category:${category.key}`,
      label: category.label,
      kind: "category",
      role: "group",
      status: aggregateStatus(sortedChildren),
      children: sortedChildren,
    };
  });
}

function assetNode(
  asset: ManagedAssetDto,
  registry: AssetRegistryDto,
  childrenByParent: Map<string, ManagedAssetDto[]>,
  missingParent: boolean,
): AssetTreeNode {
  const ownedChildren = childrenByParent.get(asset.assetKey) ?? [];
  const childNodes: AssetTreeNode[] = [];
  const diagnostics = missingParent
    ? [missingParentDiagnostic(asset), ...asset.diagnostics]
    : asset.diagnostics;

  childNodes.push({
    key: `${asset.assetKey}:descriptor`,
    label: "Descriptor",
    kind: "descriptor",
    role: "file",
    status: statusForAsset(asset, missingParent),
    assetKey: asset.assetKey,
    descriptorRelativePath: asset.descriptorRelativePath,
    asset,
    references: asset.references,
    usedBy: asset.usedBy,
    diagnostics,
    children: [],
  });

  let remainingChildren = ownedChildren;
  for (const bucket of SUBASSET_BUCKETS) {
    const bucketAssets = bucket.key === "other"
      ? remainingChildren
      : remainingChildren.filter(bucket.predicate);
    if (bucketAssets.length === 0) continue;
    if (bucket.key !== "other") {
      const bucketKeys = new Set(bucketAssets.map((child) => child.assetKey));
      remainingChildren = remainingChildren.filter((child) => !bucketKeys.has(child.assetKey));
    }
    childNodes.push(groupNode(
      `${asset.assetKey}:${bucket.key}`,
      bucket.label,
      bucketAssets.map((child) => assetNode(child, registry, childrenByParent, false)),
    ));
  }

  if (diagnostics.length) {
    childNodes.push(groupNode(
      `${asset.assetKey}:diagnostics`,
      "Diagnostics",
      diagnostics.map((diagnostic, index) => diagnosticNode(asset.assetKey, diagnostic, index)),
    ));
  }

  return {
    key: asset.assetKey,
    label: asset.label || asset.assetId || asset.assetKey,
    kind: asset.kind,
    role: asset.role,
    status: statusForAsset(asset, missingParent),
    assetKey: asset.assetKey,
    descriptorRelativePath: asset.descriptorRelativePath,
    asset,
    references: asset.references,
    usedBy: asset.usedBy,
    diagnostics,
    children: orderFamilyChildren(childNodes),
  };
}

function rawFileNode(file: RawAssetFileDto): AssetTreeNode {
  return {
    key: file.relativePath,
    label: file.relativePath.split("/").pop() ?? file.relativePath,
    kind: file.mediaType,
    role: "file",
    status: file.orphan ? "orphan" : "valid",
    rawFile: file,
    usedBy: file.referencedBy,
    children: [],
  };
}

function groupNode(key: string, label: string, children: AssetTreeNode[]): AssetTreeNode {
  return {
    key,
    label,
    kind: "group",
    role: "group",
    status: aggregateStatus(children),
    children: sortNodes(children),
  };
}

function diagnosticNode(ownerKey: string, diagnostic: EditorDiagnosticDto, index: number): AssetTreeNode {
  return {
    key: `${ownerKey}:diagnostic:${diagnostic.code}:${index}`,
    label: diagnostic.code,
    kind: "diagnostic",
    role: "reference",
    status: diagnostic.level === "error" ? "error" : "warning",
    diagnostics: [diagnostic],
    children: [],
  };
}

function statusForAsset(asset: ManagedAssetDto, missingParent: boolean): AssetTreeStatus {
  if (missingParent) return "missing";
  if (asset.status === "missingSource") return "missing";
  if (asset.status === "error") return "error";
  if (asset.status === "warning" || asset.diagnostics.length > 0) return "warning";
  return "valid";
}

function aggregateStatus(children: AssetTreeNode[]): AssetTreeStatus {
  if (children.some((child) => child.status === "error")) return "error";
  if (children.some((child) => child.status === "missing")) return "missing";
  if (children.some((child) => child.status === "warning" || child.status === "orphan")) return "warning";
  return "valid";
}

function orderFamilyChildren(children: AssetTreeNode[]): AssetTreeNode[] {
  const order = ["Descriptor", "Tilesets", "Rulesets", "Animations", "Diagnostics"];
  return [...children].sort((left, right) => {
    const leftOrder = order.indexOf(left.label);
    const rightOrder = order.indexOf(right.label);
    return (leftOrder === -1 ? 999 : leftOrder) - (rightOrder === -1 ? 999 : rightOrder)
      || left.label.localeCompare(right.label)
      || left.key.localeCompare(right.key);
  });
}

function sortNodes(nodes: AssetTreeNode[]): AssetTreeNode[] {
  return [...nodes].sort((left, right) => left.label.localeCompare(right.label) || left.key.localeCompare(right.key));
}

function appendMapList<TKey, TValue>(map: Map<TKey, TValue[]>, key: TKey, value: TValue) {
  const list = map.get(key) ?? [];
  list.push(value);
  map.set(key, list);
}

function missingParentDiagnostic(asset: ManagedAssetDto): EditorDiagnosticDto {
  return {
    level: "warning",
    code: "asset_parent_missing",
    message: `Asset ${asset.assetKey} references missing parent ${asset.parentKey ?? "unknown"}.`,
    path: asset.descriptorRelativePath,
  };
}
