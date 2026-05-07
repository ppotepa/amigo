import type { EditorUiNodeDto } from "../../api/dto";
import type { TreeNodeAdapter, TreeNodeCapabilities } from "../../ui/tree";
import { uiNodeCapabilitiesForKind } from "./uiNodeCapabilities";
import { UiNodeKindIcon, uiNodeKindLabel } from "./uiNodeTreeIcons";

function uiDocumentTreeCapabilities(
  node: EditorUiNodeDto,
  context: { hasChildren: boolean },
): TreeNodeCapabilities {
  const nodeCaps = uiNodeCapabilitiesForKind(node.kind);

  return {
    canExpand: context.hasChildren,
    canSelect: true,
    canOpen: true,
    canAddChild: nodeCaps.canAddChild,
    canRename: true,
    canDelete: node.path !== "root",
    canDrag: node.path !== "root",
    canDropOn: nodeCaps.canAddChild,
  };
}

// @codemap anchor:ui-document-tree-adapter domain:ui-document role:tree-adapter priority:P1 layer:app tags:tree,adapter,capabilities
export const uiDocumentTreeAdapter: TreeNodeAdapter<EditorUiNodeDto> = {
  getId: (node) => node.path,
  getLabel: (node) => node.label,
  getChildren: (node) => node.children,
  getIcon: (node) => <UiNodeKindIcon kind={node.kind} />,
  getMeta: (node) => uiNodeKindLabel(node.kind),
  getClassName: (node) => `ui-node-tree-kind-${node.kind}`,
  getCapabilities: (node, context) => uiDocumentTreeCapabilities(node, context),
};
