import { AlertTriangle, FileText, Package } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";
import type { AddItemDialogRequest, AddItemScope } from "../add-item/addItemTypes";
import type { AssetRegistryDto, ManagedAssetDto, RawAssetFileDto } from "../api/dto";
import { semanticIconClass, toneForStatus } from "../theme/semanticColorRegistry";
import { TreeView, treeRowStyle, type TreeNodeTone } from "../ui/TreeView";
import { assetFolderVisualForKind, assetVisualForKind } from "./assetVisualRegistry";
import { buildAssetTree, type AssetTreeNode } from "./assetTreeBuilder";

export function AssetTreePanel({
  registry,
  selectedAssetKey,
  selectedFilePath,
  onCreateDescriptor,
  onAddItem,
  onSelectAsset,
  onSelectRawFile,
}: {
  registry: AssetRegistryDto;
  selectedAssetKey?: string | null;
  selectedFilePath: string | null;
  onCreateDescriptor?: (file: RawAssetFileDto) => Promise<void>;
  onAddItem?: (request: AddItemDialogRequest) => void;
  onSelectAsset: (asset: ManagedAssetDto) => void;
  onSelectRawFile: (file: RawAssetFileDto) => void;
}) {
  const nodes = useMemo(() => buildAssetTree(registry), [registry]);
  const sceneNodes = useMemo(() => nodes.find((node) => node.key === "category:scenes")?.children ?? [], [nodes]);
  const generalNodes = useMemo(() => nodes.filter((node) => node.key !== "category:scenes"), [nodes]);
  const [expandedKeys, setExpandedKeys] = useState<Set<string>>(() => defaultExpandedKeys(nodes, selectedAssetKey ?? null));
  const totalScenes = countTreeItems(sceneNodes);
  const totalGeneralAssets = countTreeItems(generalNodes);

  useEffect(() => {
    setExpandedKeys((current) => {
      const next = new Set(current);
      for (const key of defaultExpandedKeys(nodes, selectedAssetKey ?? null)) {
        next.add(key);
      }
      return next;
    });
  }, [nodes, selectedAssetKey]);

  function toggleNode(key: string) {
    setExpandedKeys((current) => {
      const next = new Set(current);
      if (next.has(key)) {
        next.delete(key);
      } else {
        next.add(key);
      }
      return next;
    });
  }

  return (
    <div className="asset-tree-panel">
      <SceneAssetExplorer
        expandedKeys={expandedKeys}
        nodes={sceneNodes}
        selectedFilePath={selectedFilePath}
        totalCount={totalScenes}
        onCreateDescriptor={onCreateDescriptor}
        onAddItem={onAddItem}
        onSelectAsset={onSelectAsset}
        onSelectRawFile={onSelectRawFile}
        onToggle={toggleNode}
      />
      <GeneralAssetExplorer
        expandedKeys={expandedKeys}
        nodes={generalNodes}
        selectedFilePath={selectedFilePath}
        totalCount={totalGeneralAssets}
        onCreateDescriptor={onCreateDescriptor}
        onAddItem={onAddItem}
        onSelectAsset={onSelectAsset}
        onSelectRawFile={onSelectRawFile}
        onToggle={toggleNode}
      />
    </div>
  );
}

function SceneAssetExplorer(props: AssetTreeSectionProps) {
  return (
    <AssetTreeSection
      {...props}
      addItemScope={{ kind: "asset-category", category: "scenes" }}
      iconTone={assetFolderVisualForKind("scenes").tone}
      rootIcon={assetFolderVisualForKind("scenes").icon}
      title="Scenes"
    />
  );
}

function GeneralAssetExplorer(props: AssetTreeSectionProps) {
  return (
    <AssetTreeSection
      {...props}
      addItemScope={{ kind: "project-root" }}
      iconTone={assetFolderVisualForKind("root").tone}
      rootIcon={<Package size={13} />}
      title="Assets"
    />
  );
}

type AssetTreeSectionProps = {
  expandedKeys: Set<string>;
  nodes: AssetTreeNode[];
  selectedFilePath: string | null;
  totalCount: number;
  onCreateDescriptor?: (file: RawAssetFileDto) => Promise<void>;
  onAddItem?: (request: AddItemDialogRequest) => void;
  onSelectAsset: (asset: ManagedAssetDto) => void;
  onSelectRawFile: (file: RawAssetFileDto) => void;
  onToggle: (key: string) => void;
};

function AssetTreeSection({
  expandedKeys,
  iconTone,
  nodes,
  rootIcon,
  selectedFilePath,
  title,
  totalCount,
  onAddItem,
  onCreateDescriptor,
  onSelectAsset,
  onSelectRawFile,
  onToggle,
  addItemScope,
}: AssetTreeSectionProps & {
  addItemScope: AddItemScope;
  iconTone: string;
  rootIcon: ReactNode;
  title: string;
}) {
  return (
    <section className="asset-tree-section" aria-label={title}>
      <div className="asset-tree-section-header">
        <span className={`dock-icon asset-status-icon ${iconTone}`}>{rootIcon}</span>
        <span className="asset-tree-section-title">{title}</span>
        <TreeCountBadge count={totalCount} />
        {onAddItem ? (
          <button
            type="button"
            className="asset-tree-add-button"
            title={`Add item to ${title}`}
            onClick={() => onAddItem(buildAddItemRequestForScope(addItemScope))}
          >
            +
          </button>
        ) : null}
      </div>
      <TreeView
        expandedKeys={expandedKeys}
        nodes={nodes}
        onToggle={onToggle}
        renderNode={({ depth, expanded, hasChildren, node, toggle }) => (
          <AssetTreeNodeRow
            depth={depth}
            expanded={expanded}
            hasChildren={hasChildren}
            node={node}
            selectedFilePath={selectedFilePath}
            onCreateDescriptor={onCreateDescriptor}
            onSelectAsset={onSelectAsset}
            onSelectRawFile={onSelectRawFile}
            onAddItem={onAddItem}
            onToggle={toggle}
          />
        )}
      />
    </section>
  );
}

function AssetTreeNodeRow({
  node,
  depth,
  expanded,
  hasChildren,
  selectedFilePath,
  onCreateDescriptor,
  onSelectAsset,
  onSelectRawFile,
  onAddItem,
  onToggle,
}: {
  node: AssetTreeNode;
  depth: number;
  expanded: boolean;
  hasChildren: boolean;
  selectedFilePath: string | null;
  onCreateDescriptor?: (file: RawAssetFileDto) => Promise<void>;
  onSelectAsset: (asset: ManagedAssetDto) => void;
  onSelectRawFile: (file: RawAssetFileDto) => void;
  onAddItem?: (request: AddItemDialogRequest) => void;
  onToggle: () => void;
}) {
  const selected = isSelectedNode(node, selectedFilePath);
  const clickable = Boolean(node.asset || node.rawFile);
  const detail = nodeDetail(node);
  const rowTone = rowToneForNode(node);
  const content = (
    <>
      <span className="tree-view-twist">{hasChildren ? (expanded ? "▾" : "▸") : ""}</span>
      <span className={`dock-icon asset-status-icon ${visualToneForNode(node)} ${statusClass(node.status)}`} title={statusTitle(node.status)}>
        {iconForNode(node)}
      </span>
      <span className="tree-view-label">
        <strong>{node.label}</strong>
        {detail ? <small>{detail}</small> : null}
      </span>
      {countForNode(node) !== null ? <TreeCountBadge count={countForNode(node)!} /> : null}
      {countForNode(node) === null && node.status ? <small className="tree-view-meta asset-row-status">{node.status}</small> : null}
    </>
  );

  return (
    <div className="asset-tree-node">
      {clickable ? (
        <div className={`tree-view-item ${selected ? "selected" : ""}`} style={treeRowStyle(depth)}>
          <button
            className={`tree-view-row tree-view-row-${rowTone} ${depth > 0 ? "tree-view-row-nested" : ""}`}
            type="button"
            onClick={() => {
              if (hasChildren) {
                onToggle();
              }
              if (node.asset) {
                onSelectAsset(node.asset);
              } else if (node.rawFile) {
                onSelectRawFile(node.rawFile);
              }
            }}
          >
            {content}
          </button>
          {node.rawFile?.orphan && node.rawFile.mediaType.startsWith("image/") && onCreateDescriptor ? (
            <button type="button" className="asset-tree-action" onClick={() => void onCreateDescriptor(node.rawFile!)}>
              descriptor
            </button>
          ) : null}
          {node.kind === "category" && onAddItem ? (
            <button
              type="button"
              className="asset-tree-action"
              onClick={() => {
                const scope = scopeForCategoryNode(node.key);
                if (scope) onAddItem(buildAddItemRequestForScope(scope));
              }}
            >
              +
            </button>
          ) : null}
        </div>
      ) : (
        <div className="tree-view-item" style={treeRowStyle(depth)}>
          <button
            type="button"
            className={`tree-view-row tree-view-row-${rowTone} ${depth > 0 ? "tree-view-row-nested" : ""}`}
            onClick={() => hasChildren ? onToggle() : undefined}
          >
            {content}
          </button>
          {node.kind === "category" && onAddItem ? (
            <button
              type="button"
              className="asset-tree-action"
              title={`Add item to ${node.label}`}
              onClick={() => {
                const scope = scopeForCategoryNode(node.key);
                if (scope) onAddItem(buildAddItemRequestForScope(scope));
              }}
            >
              +
            </button>
          ) : null}
        </div>
      )}
    </div>
  );
}

function scopeForCategoryNode(key: string): AddItemScope | null {
  if (key === "category:scenes") return { kind: "asset-category", category: "scenes" };
  if (key === "category:spritesheets") return { kind: "asset-category", category: "spritesheets" };
  if (key === "category:tilemaps") return { kind: "asset-category", category: "tilemaps" };
  if (key === "category:audio") return { kind: "asset-category", category: "audio" };
  if (key === "category:fonts") return { kind: "asset-category", category: "fonts" };
  if (key === "category:scripts") return { kind: "asset-category", category: "scripts" };
  if (key === "category:raw") return { kind: "asset-category", category: "raw" };
  return { kind: "project-root" };
}

function buildAddItemRequestForScope(scope: AddItemScope): AddItemDialogRequest {
  if (scope.kind !== "asset-category") {
    return { mode: "catalog", scope };
  }

  if (scope.category === "scenes") {
    return { mode: "direct", scope, itemKind: "scene" };
  }
  if (scope.category === "fonts") {
    return { mode: "direct", scope, itemKind: "font" };
  }
  if (scope.category === "raw") {
    return { mode: "direct", scope, itemKind: "raw-source" };
  }
  if (scope.category === "scripts") {
    return { mode: "direct", scope, itemKind: "script" };
  }
  if (scope.category === "spritesheets") {
    return {
      mode: "direct",
      scope,
      itemKind: "image",
      prefillDescriptorKind: "sprite",
    };
  }

  return { mode: "catalog", scope };
}

function TreeCountBadge({ count }: { count: number }) {
  return <small className={`tree-view-count ${count === 0 ? "tree-view-count-empty" : ""}`}>{count}</small>;
}

function countTreeItems(nodes: AssetTreeNode[]): number {
  return nodes.reduce((total, node) => {
    const ownItem = node.rawFile || (node.asset && node.key === node.asset.assetKey) ? 1 : 0;
    return total + ownItem + countTreeItems(node.children);
  }, 0);
}

function isSelectedNode(node: AssetTreeNode, selectedFilePath: string | null): boolean {
  if (!selectedFilePath) return false;
  return node.asset?.descriptorRelativePath === selectedFilePath || node.rawFile?.relativePath === selectedFilePath;
}

function nodeDetail(node: AssetTreeNode): string {
  if (node.kind === "diagnostic") {
    return node.diagnostics?.[0]?.message ?? "";
  }
  if (node.asset) {
    return node.role === "reference" || node.role === "usedBy"
      ? node.asset.assetKey
      : node.asset.descriptorRelativePath;
  }
  if (node.rawFile) {
    return node.rawFile.relativePath;
  }
  if (node.children.length) {
    return "";
  }
  return "";
}

function countForNode(node: AssetTreeNode): number | null {
  if (node.kind === "category" || node.kind === "group") {
    return node.children.length;
  }
  return null;
}

function iconForNode(node: AssetTreeNode) {
  if (node.kind === "diagnostic") return <AlertTriangle size={13} className={semanticIconClass(toneForStatus(node.status))} />;
  if (node.kind === "descriptor") return <FileText size={13} className="semantic-icon domain-assets" />;
  if (node.kind === "category" || node.kind === "group") return assetFolderVisualForKind(node.key).icon;
  return assetVisualForKind(node.kind).icon;
}

function visualToneForNode(node: AssetTreeNode): string {
  if (node.kind === "category" || node.kind === "group") {
    return assetFolderVisualForKind(node.key).tone;
  }
  return assetVisualForKind(node.kind).tone;
}

function rowToneForNode(node: AssetTreeNode): TreeNodeTone {
  if (node.kind === "category") return "folder";
  if (node.kind === "group") return "group";
  if (node.kind === "diagnostic" || node.role === "reference" || node.role === "usedBy") return "meta";
  return "item";
}

function statusClass(status?: string): string {
  if (status === "error") return "asset-status-error";
  if (status === "missing") return "asset-status-missingSource";
  if (status === "warning" || status === "orphan") return "asset-status-warning";
  return "asset-status-valid";
}

function statusTitle(status?: string): string {
  if (status === "error") return "Error";
  if (status === "missing") return "Missing reference or source";
  if (status === "warning") return "Warning";
  if (status === "orphan") return "Orphan raw source";
  return "Valid";
}

function defaultExpandedKeys(nodes: AssetTreeNode[], selectedAssetKey: string | null): Set<string> {
  const keys = new Set<string>();
  for (const node of nodes) {
    keys.add(node.key);
  }
  if (selectedAssetKey) {
    addAncestorsForSelected(nodes, selectedAssetKey, keys);
  }
  return keys;
}

function addAncestorsForSelected(nodes: AssetTreeNode[], selectedAssetKey: string, keys: Set<string>): boolean {
  for (const node of nodes) {
    const selected = node.assetKey === selectedAssetKey || node.asset?.assetKey === selectedAssetKey;
    const childSelected = addAncestorsForSelected(node.children, selectedAssetKey, keys);
    if (selected || childSelected) {
      keys.add(node.key);
      return true;
    }
  }
  return false;
}
