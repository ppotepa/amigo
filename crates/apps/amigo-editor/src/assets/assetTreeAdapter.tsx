import { AlertTriangle, FileText, Folder, Plus, Trash2, Wand2 } from "lucide-react";
import type { AddItemDialogRequest, AddItemScope } from "../add-item/addItemTypes";
import type { TreeNodeAction, TreeNodeAdapter, TreeNodeCapabilities } from "../ui/tree";
import { assetFolderVisualForKind, assetVisualForKind } from "./assetVisualRegistry";
import type { AssetDeleteTarget } from "./AssetTreePanel";
import type { AssetTreeNode } from "./assetTreeBuilder";

function assetCapabilities(node: AssetTreeNode, context: { hasChildren: boolean }): TreeNodeCapabilities {
  return {
    canExpand: context.hasChildren,
    canSelect: Boolean(node.asset || node.rawFile),
    canOpen: Boolean(node.asset || node.rawFile),
    canAddChild: node.kind === "category",
    canRename: false,
    canDelete: Boolean(deletableTargetForNode(node)),
    canDrag: Boolean(node.asset || node.rawFile),
    canDropOn: node.kind === "category" || node.kind === "group",
  };
}

// @codemap anchor:asset-tree-adapter domain:assets role:tree-adapter priority:P1 layer:app tags:tree,assets,adapter
export const assetTreeAdapter: TreeNodeAdapter<AssetTreeNode> = {
  getId: (node) => node.key,
  getLabel: (node) => node.label,
  getChildren: (node) => node.children,
  getIcon: (node) => <AssetTreeIcon node={node} />,
  getMeta: (node) => countForNode(node),
  getBadges: (node) => [
    {
      label: node.status,
      tone: node.status === "error" || node.status === "missing"
        ? "error"
        : node.status === "warning" || node.status === "orphan"
          ? "warning"
          : "valid",
      visible: Boolean(node.status),
    },
  ],
  getSubItems: (node) => [
    {
      key: "detail",
      label: nodeDetail(node),
      title: nodeDetail(node),
      visible: Boolean(nodeDetail(node)),
    },
  ],
  getClassName: (node) => assetTreeNodeClassName(node),
  getActions: (node) => assetNodeActions(node),
  getCapabilities: (node, context) => assetCapabilities(node, context),
};

export function selectedAssetTreeId(
  nodes: readonly AssetTreeNode[],
  selectedAssetKey: string | null | undefined,
  selectedFilePath: string | null | undefined,
): string | null {
  for (const node of nodes) {
    if (selectedAssetKey && node.assetKey === selectedAssetKey) return node.key;
    if (selectedFilePath && node.asset?.descriptorRelativePath === selectedFilePath) return node.key;
    if (selectedFilePath && node.rawFile?.relativePath === selectedFilePath) return node.key;
    const child = selectedAssetTreeId(node.children, selectedAssetKey, selectedFilePath);
    if (child) return child;
  }
  return null;
}

export function countTreeItems(nodes: AssetTreeNode[]): number {
  return nodes.reduce((total, node) => {
    const ownItem = node.rawFile || (node.asset && node.key === node.asset.assetKey) ? 1 : 0;
    return total + ownItem + countTreeItems(node.children);
  }, 0);
}

export function scopeForCategoryNode(key: string): AddItemScope | null {
  if (key === "category:scenes") return { kind: "asset-category", category: "scenes" };
  if (key === "category:ui") return { kind: "asset-category", category: "ui" };
  if (key === "category:spritesheets") return { kind: "asset-category", category: "spritesheets" };
  if (key === "category:tilemaps") return { kind: "asset-category", category: "tilemaps" };
  if (key === "category:audio") return { kind: "asset-category", category: "audio" };
  if (key === "category:fonts") return { kind: "asset-category", category: "fonts" };
  if (key === "category:scripts") return { kind: "asset-category", category: "scripts" };
  if (key === "category:raw") return { kind: "asset-category", category: "raw" };
  return { kind: "project-root" };
}

export function buildAddItemRequestForScope(scope: AddItemScope): AddItemDialogRequest {
  if (scope.kind !== "asset-category") return { mode: "catalog", scope };
  if (scope.category === "scenes") return { mode: "direct", scope, itemKind: "scene" };
  if (scope.category === "ui") return { mode: "direct", scope, itemKind: "ui-main-menu" };
  if (scope.category === "fonts") return { mode: "direct", scope, itemKind: "font" };
  if (scope.category === "raw") return { mode: "direct", scope, itemKind: "raw-source" };
  if (scope.category === "scripts") return { mode: "direct", scope, itemKind: "script" };
  if (scope.category === "spritesheets") {
    return { mode: "direct", scope, itemKind: "image", prefillDescriptorKind: "sprite" };
  }

  return { mode: "catalog", scope };
}

export function deletableTargetForNode(node: AssetTreeNode): AssetDeleteTarget | null {
  if (node.asset?.descriptorRelativePath) {
    return { relativePath: node.asset.descriptorRelativePath, label: node.label, kind: "asset" };
  }
  if (node.rawFile?.relativePath) {
    return { relativePath: node.rawFile.relativePath, label: node.label, kind: "raw" };
  }
  return null;
}

function assetNodeActions(node: AssetTreeNode): TreeNodeAction<AssetTreeNode>[] {
  const result: TreeNodeAction<AssetTreeNode>[] = [];

  if (node.kind === "category") {
    result.push({ id: "addItem", label: "Add", icon: <Plus size={13} />, tone: "primary" });
  }

  if (node.rawFile?.orphan && node.rawFile.mediaType.startsWith("image/")) {
    result.push({ id: "createDescriptor", label: "Descriptor", icon: <Wand2 size={13} />, tone: "success" });
  }

  if (deletableTargetForNode(node)) {
    result.push({ id: "delete", label: "Delete", icon: <Trash2 size={12} />, tone: "danger" });
  }

  return result;
}

function assetTreeNodeClassName(node: AssetTreeNode): string {
  const tone = assetVisualForKind(node.asset?.kind ?? node.rawFile?.mediaType ?? node.key).tone;
  const roleClass = node.kind === "category" ? "tree-view-item-category" : `tree-view-row-${node.role}`;
  return [`asset-tree-node-${node.kind}`, roleClass, tone].join(" ");
}

function AssetTreeIcon({ node }: { node: AssetTreeNode }) {
  if (node.kind === "diagnostic") return <AlertTriangle size={14} />;
  if (node.asset) return assetVisualForKind(node.asset.kind).icon;
  if (node.rawFile) return <FileText size={14} />;
  if (node.kind === "category" || node.kind === "group") return <Folder size={14} className={assetFolderVisualForKind(node.key).tone} />;
  return assetVisualForKind(node.kind).icon;
}

function nodeDetail(node: AssetTreeNode): string {
  if (node.kind === "diagnostic") return node.diagnostics?.[0]?.message ?? "";
  if (node.asset) {
    return node.role === "reference" || node.role === "usedBy"
      ? node.asset.assetKey
      : node.asset.descriptorRelativePath;
  }
  if (node.rawFile) return node.rawFile.relativePath;
  return "";
}

function countForNode(node: AssetTreeNode): string | null {
  if (node.kind === "category" || node.kind === "group") return String(node.children.length);
  return null;
}
