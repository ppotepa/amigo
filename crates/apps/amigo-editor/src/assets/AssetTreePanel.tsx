import { useMemo, useState } from "react";
import type { AddItemDialogRequest } from "../add-item/addItemTypes";
import type { AssetRegistryDto, ManagedAssetDto, RawAssetFileDto } from "../api/dto";
import { TreeView, useTreeExpansion } from "../ui/tree";
import {
  assetTreeAdapter,
  buildAddItemRequestForScope,
  countTreeItems,
  deletableTargetForNode,
  scopeForCategoryNode,
  selectedAssetTreeId,
} from "./assetTreeAdapter";
import { buildAssetTree, type AssetTreeNode } from "./assetTreeBuilder";

export type AssetDeleteTarget = {
  relativePath: string;
  label: string;
  kind: "asset" | "raw";
};

export function AssetTreePanel({
  registry,
  selectedAssetKey,
  selectedFilePath,
  onCreateDescriptor,
  onDeleteProjectFile,
  onAddItem,
  onSelectAsset,
  onSelectRawFile,
}: {
  registry: AssetRegistryDto;
  selectedAssetKey?: string | null;
  selectedFilePath: string | null;
  onCreateDescriptor?: (file: RawAssetFileDto) => Promise<void>;
  onDeleteProjectFile?: (target: AssetDeleteTarget) => void;
  onAddItem?: (request: AddItemDialogRequest) => void;
  onSelectAsset: (asset: ManagedAssetDto) => void;
  onSelectRawFile: (file: RawAssetFileDto) => void;
}) {
  const [showAllCategories, setShowAllCategories] = useState(false);
  const allNodes = useMemo(() => buildAssetTree(registry), [registry]);
  const nodes = useMemo(
    () => showAllCategories ? allNodes : allNodes.filter((node) => node.children.length > 0),
    [allNodes, showAllCategories],
  );
  const sceneNodes = useMemo(() => nodes.find((node) => node.key === "category:scenes")?.children ?? [], [nodes]);
  const generalNodes = useMemo(() => nodes.filter((node) => node.key !== "category:scenes"), [nodes]);
  const totalScenes = countTreeItems(sceneNodes);
  const totalGeneralAssets = countTreeItems(generalNodes);
  const showSceneSection = showAllCategories || sceneNodes.length > 0;
  const showGeneralSection = showAllCategories || generalNodes.length > 0;

  function handleAction(actionId: string, node: AssetTreeNode) {
    if (actionId === "addItem") {
      const scope = scopeForCategoryNode(node.key);
      if (scope) onAddItem?.(buildAddItemRequestForScope(scope));
    }
    if (actionId === "createDescriptor" && node.rawFile) {
      void onCreateDescriptor?.(node.rawFile);
    }
    if (actionId === "delete") {
      const target = deletableTargetForNode(node);
      if (target) onDeleteProjectFile?.(target);
    }
  }

  function selectNode(node: AssetTreeNode) {
    if (node.asset) onSelectAsset(node.asset);
    if (node.rawFile) onSelectRawFile(node.rawFile);
  }

  return (
    <div className="asset-tree-panel">
      <div className="asset-tree-toolbar">
        <span>Categories</span>
        <button
          type="button"
          className={`asset-tree-toggle ${showAllCategories ? "active" : ""}`}
          aria-pressed={showAllCategories}
          onClick={() => setShowAllCategories((current) => !current)}
        >
          <span className="asset-tree-toggle-track">
            <span className="asset-tree-toggle-thumb" />
          </span>
          <span>Show all</span>
        </button>
      </div>

      {showSceneSection ? (
        <AssetTreeSection
          nodes={sceneNodes}
          selectedAssetKey={selectedAssetKey}
          selectedFilePath={selectedFilePath}
          title={`Scenes ${totalScenes ? `(${totalScenes})` : ""}`}
          onAction={handleAction}
          onOpen={selectNode}
          onSelect={selectNode}
        />
      ) : null}

      {showGeneralSection ? (
        <AssetTreeSection
          nodes={generalNodes}
          selectedAssetKey={selectedAssetKey}
          selectedFilePath={selectedFilePath}
          title={`Assets ${totalGeneralAssets ? `(${totalGeneralAssets})` : ""}`}
          onAction={handleAction}
          onOpen={selectNode}
          onSelect={selectNode}
        />
      ) : null}

      {!showSceneSection && !showGeneralSection ? (
        <p className="asset-tree-empty-note">No asset categories contain items. Enable Show all to create items in empty categories.</p>
      ) : null}
    </div>
  );
}

// @codemap anchor:asset-shared-tree-section domain:assets role:tree priority:P1 layer:app tags:tree,shared-tree,adapter
function AssetTreeSection({
  nodes,
  onAction,
  onOpen,
  onSelect,
  selectedAssetKey,
  selectedFilePath,
  title,
}: {
  nodes: AssetTreeNode[];
  onAction: (actionId: string, node: AssetTreeNode) => void;
  onOpen: (node: AssetTreeNode) => void;
  onSelect: (node: AssetTreeNode) => void;
  selectedAssetKey?: string | null;
  selectedFilePath: string | null;
  title: string;
}) {
  const selectedId = selectedAssetTreeId(nodes, selectedAssetKey, selectedFilePath);
  const { expandedIds, toggleExpanded } = useTreeExpansion({
    adapter: assetTreeAdapter,
    nodes,
    selectedId,
  });

  if (!nodes.length) return null;

  return (
    <section className="asset-tree-section">
      <header className="asset-tree-section-heading">
        <strong>{title}</strong>
      </header>
      <TreeView
        actions={{ onAction, onOpen, onSelect }}
        adapter={assetTreeAdapter}
        className="asset-registry-tree"
        expandedIds={expandedIds}
        nodes={nodes}
        onToggle={toggleExpanded}
        preset="explorer"
        selectedId={selectedId}
      />
    </section>
  );
}
