import { Box, Boxes, Code2, Copy, FileCode2, FileCog, Folder, Image as ImageIcon, Link, Map as MapIcon, Package, Play, Plug, Plus, TriangleAlert } from "lucide-react";
import type { TreeNodeAction, TreeNodeAdapter, TreeNodeCapabilities } from "../../ui/tree";
import { semanticIconClass, toneForFileKind, toneForStatus } from "../../theme/semanticColorRegistry";
import type { ProjectExplorerTreeNode } from "./projectTreeModel";
import { projectNodeKindLabel } from "./projectTreeModel";

function projectNodeCapabilities(
  node: ProjectExplorerTreeNode,
  context: { hasChildren: boolean },
): TreeNodeCapabilities {
  const canOpen = Boolean(
    (node.file && !node.file.isDir) ||
    node.scene ||
    ["overview", "capabilities", "dependencies", "diagnostics"].includes(node.kind),
  );

  return {
    canExpand: context.hasChildren,
    canSelect: true,
    canOpen,
    canAddChild: false,
    canRename: false,
    canDelete: false,
    canDrag: false,
    canDropOn: Boolean(node.file?.isDir),
  };
}

// @codemap anchor:project-tree-adapter domain:project role:tree-adapter priority:P1 layer:app tags:tree,project,adapter
export const projectTreeAdapter: TreeNodeAdapter<ProjectExplorerTreeNode> = {
  getId: (node) => node.id,
  getLabel: (node) => node.label,
  getChildren: (node) => node.children ?? [],
  getIcon: (node) => <ProjectNodeIcon node={node} />,
  getMeta: (node) => node.count != null ? String(node.count) : projectNodeKindLabel(node.kind),
  getSubItems: (node) => [
    {
      key: "path",
      label: node.expectedPath && !node.exists ? node.expectedPath : node.path ?? "",
      title: node.expectedPath ?? node.path ?? "",
      tone: node.ghost ? "warning" : "muted",
      visible: Boolean((node.expectedPath && !node.exists) || node.path),
    },
  ],
  getBadges: (node) => [
    { label: "missing", tone: "warning", visible: Boolean(node.ghost) },
    { label: node.status, tone: node.status === "error" ? "error" : node.status === "warn" ? "warning" : "valid", visible: Boolean(node.status && !node.ghost) },
  ],
  getActions: (node) => {
    const actions: TreeNodeAction<ProjectExplorerTreeNode>[] = [];
    if (node.ghost && node.expectedPath) {
      actions.push({ id: "createExpectedFolder", label: "Create", icon: <Plus size={13} />, tone: "primary" });
    }
    if ((node.file && !node.file.isDir) || node.scene) {
      actions.push({ id: "open", label: "Open", icon: <Play size={13} />, tone: "primary" });
    }
    if (node.path || node.expectedPath) {
      actions.push({ id: "copyPath", label: "Copy", icon: <Copy size={13} /> });
    }
    return actions;
  },
  getClassName: (node) => `project-tree-node project-tree-node-${node.kind}`,
  getCapabilities: (node, context) => projectNodeCapabilities(node, context),
};

function ProjectNodeIcon({ node }: { node: ProjectExplorerTreeNode }) {
  const size = 14;
  const icon = node.icon.toLowerCase();
  const statusTone = node.status === "error" || node.status === "warn" ? toneForStatus(node.status === "warn" ? "warning" : node.status) : null;
  if (node.ghost) return <TriangleAlert size={size} />;
  if (node.kind === "manifest" || icon === "toml") return <FileCog size={size} className="semantic-icon domain-modding" />;
  if (node.kind === "scene" || icon === "play") return <Play size={size} className={semanticIconClass(statusTone ?? "domain-scene")} />;
  if (node.kind === "sceneDocument" || icon === "yml") return <FileCode2 size={size} className={semanticIconClass(toneForFileKind("sceneDocument"))} />;
  if (node.kind === "sceneScript" || icon === "rh") return <Code2 size={size} className={semanticIconClass(toneForFileKind("sceneScript"))} />;
  if (icon === "img") return <ImageIcon size={size} className="semantic-icon asset-image" />;
  if (icon === "map") return <MapIcon size={size} className="semantic-icon asset-tilemap" />;
  if (node.kind === "scriptPackage" || icon === "pkg") return <Package size={size} className="semantic-icon domain-scripting" />;
  if (node.kind === "capabilities" || icon === "plug") return <Plug size={size} className="semantic-icon domain-project" />;
  if (node.kind === "dependencies" || icon === "link") return <Link size={size} className="semantic-icon domain-modding" />;
  if (node.kind === "modRoot") return <Boxes size={size} className="semantic-icon domain-modding" />;
  if (node.children?.length || node.kind === "expectedFolder" || node.kind === "folder") return <Folder size={size} className="semantic-icon domain-project" />;
  return <Box size={size} className="semantic-icon neutral" />;
}
