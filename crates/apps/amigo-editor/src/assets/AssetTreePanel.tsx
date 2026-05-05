import { AlertTriangle, FileText, Package } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";
import type { AssetRegistryDto, ManagedAssetDto, RawAssetFileDto } from "../api/dto";
import { TreeView, treeRowStyle, type TreeNodeTone } from "../ui/TreeView";
import { assetFolderVisualForKind, assetVisualForKind } from "./assetVisualRegistry";
import { buildAssetTree, type AssetTreeNode } from "./assetTreeBuilder";

export function AssetTreePanel({
  registry,
  selectedAssetKey,
  selectedFilePath,
  onCreateDescriptor,
  onSelectAsset,
  onSelectRawFile,
}: {
  registry: AssetRegistryDto;
  selectedAssetKey?: string | null;
  selectedFilePath: string | null;
  onCreateDescriptor?: (file: RawAssetFileDto) => Promise<void>;
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
  onCreateDescriptor,
  onSelectAsset,
  onSelectRawFile,
  onToggle,
}: AssetTreeSectionProps & {
  iconTone: string;
  rootIcon: ReactNode;
  title: string;
}) {
  return (
    <section className="asset-tree-section" aria-label={title}>
      <div className="tree-view-row tree-view-row-root" style={treeRowStyle(0)}>
        <span className="tree-view-twist">▾</span>
        <span className={`dock-icon asset-status-icon ${iconTone}`}>{rootIcon}</span>
        <span className="tree-view-label">
          <strong>{title}</strong>
        </span>
        <TreeCountBadge count={totalCount} />
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
            className={`tree-view-row tree-view-row-${rowTone}`}
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
        </div>
      ) : (
        <button
          type="button"
          className={`tree-view-row tree-view-row-${rowTone}`}
          style={treeRowStyle(depth)}
          onClick={() => hasChildren ? onToggle() : undefined}
        >
          {content}
        </button>
      )}
    </div>
  );
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
  if (node.kind === "diagnostic") return <AlertTriangle size={13} />;
  if (node.kind === "descriptor") return <FileText size={13} />;
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
