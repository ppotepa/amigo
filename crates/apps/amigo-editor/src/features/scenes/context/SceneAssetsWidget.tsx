import { useMemo, useState } from "react";
import { FileText } from "lucide-react";
import type { ManagedAssetDto } from "../../../api/dto";
import type { EditorTargetRuntimeBridge } from "../../../main-window/workspaceRuntimeServices";
import { ContextMiniAction } from "../../../ui/context-dock/ContextRow";
import { ContextSearch } from "../../../ui/context-dock/ContextSearch";
import { ContextTree } from "../../../ui/context-dock/ContextTree";
import { WidgetFrame } from "../../../workbench/widgets/WidgetFrame";
import type { ContextTreeNode } from "../../../ui/context-dock/contextDockTypes";
import { assetYamlSource } from "../../files/yamlSourceRefs";
import { sceneContextIcon } from "./sceneContextIcons";
import type { SceneAssetGroup } from "./sceneContextTypes";

export function SceneAssetsWidget({
  groups,
  onSelectAsset,
  onShowYaml,
}: {
  groups: SceneAssetGroup[];
  onSelectAsset?: (asset: ManagedAssetDto) => void;
  onShowYaml?: EditorTargetRuntimeBridge["showYamlView"];
}) {
  const [query, setQuery] = useState("");
  const total = groups.reduce((sum, group) => sum + group.count, 0);
  const treeNodes = useMemo(() => {
    return groups
      .map((group): ContextTreeNode | null => groupToTreeNode(group, query, onSelectAsset, onShowYaml))
      .filter((node): node is ContextTreeNode => Boolean(node));
  }, [groups, onSelectAsset, onShowYaml, query]);

  return (
    <WidgetFrame
      id="scene-assets"
      title="Referenced Assets"
      icon={sceneContextIcon("assets")}
      badge={total}
      badgeTone={total ? "info" : "muted"}
    >
      <ContextSearch value={query} placeholder="Search assets..." onChange={setQuery} />
      <ContextTree nodes={treeNodes} />
    </WidgetFrame>
  );
}

function groupToTreeNode(
  group: SceneAssetGroup,
  query: string,
  onSelectAsset?: (asset: ManagedAssetDto) => void,
  onShowYaml?: EditorTargetRuntimeBridge["showYamlView"],
): ContextTreeNode | null {
  const normalized = query.trim().toLowerCase();
  const managed = group.managedAssets.filter((asset) =>
    !normalized ||
    asset.label.toLowerCase().includes(normalized) ||
    asset.assetKey.toLowerCase().includes(normalized) ||
    asset.kind.toLowerCase().includes(normalized),
  );
  const raw = group.rawFiles.filter((file) =>
    !normalized ||
    file.relativePath.toLowerCase().includes(normalized) ||
    file.mediaType.toLowerCase().includes(normalized),
  );

  const children: ContextTreeNode[] = [
    ...managed.map((asset): ContextTreeNode => ({
      id: asset.assetKey,
      title: asset.label,
      subtitle: asset.assetKey,
      icon: sceneContextIcon(asset.domain),
      badge: asset.kind,
      onSelect: () => onSelectAsset?.(asset),
      actions: (
        <ContextMiniAction
          title="Show asset YAML"
          disabled={!assetYamlSource(asset) || !onShowYaml}
          onClick={(event) => {
            event.stopPropagation();
            const source = assetYamlSource(asset);
            if (source) onShowYaml?.(source);
          }}
        >
          <FileText size={12} />
        </ContextMiniAction>
      ),
    })),
    ...raw.map((file): ContextTreeNode => ({
      id: file.relativePath,
      title: file.relativePath.split("/").filter(Boolean).pop() ?? file.relativePath,
      subtitle: file.relativePath,
      icon: sceneContextIcon(file.mediaType.startsWith("image/") ? "image" : "folder"),
      badge: file.width && file.height ? `${file.width}x${file.height}` : file.mediaType,
    })),
  ];

  if (!children.length) return null;

  return {
    id: group.id,
    title: group.label,
    icon: sceneContextIcon(group.id),
    badge: children.length,
    defaultExpanded: group.id === "spritesheet" || group.id === "tilemap",
    children,
  };
}
