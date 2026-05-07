import type { MouseEvent, ReactNode } from "react";

// @codemap anchor:shared-tree-types domain:workspace role:model priority:P1 layer:app tags:tree,adapter,capabilities,actions
export type TreeViewPreset = "compact" | "explorer" | "outline";

export type TreeNodeActionTone = "default" | "primary" | "danger" | "warning" | "success" | "muted";

export type TreeNodeAction<TNode> = {
  id: string;
  label: string;
  icon?: ReactNode;
  title?: string;
  tone?: TreeNodeActionTone;
  disabled?: boolean;
  visible?: boolean;
  payload?: unknown;
};

export type TreeNodeBadgeTone = "default" | "valid" | "warning" | "error" | "muted" | "info";

export type TreeNodeBadge = {
  label: ReactNode;
  title?: string;
  tone?: TreeNodeBadgeTone;
  visible?: boolean;
};

export type TreeNodeSubItem = {
  key: string;
  label: ReactNode;
  title?: string;
  tone?: TreeNodeBadgeTone;
  visible?: boolean;
};

export type TreeNodeCapabilities = {
  canExpand: boolean;
  canSelect: boolean;
  canOpen: boolean;
  canAddChild: boolean;
  canRename: boolean;
  canDelete: boolean;
  canDrag: boolean;
  canDropOn: boolean;
};

export type TreeCapabilityContext<TNode> = {
  depth: number;
  hasChildren: boolean;
  selected: boolean;
  expanded: boolean;
  parent: TNode | null;
};

export type TreeNodeAdapter<TNode> = {
  getId: (node: TNode) => string;
  getLabel: (node: TNode) => ReactNode;
  getChildren: (node: TNode) => readonly TNode[];
  getIcon?: (node: TNode) => ReactNode;
  getMeta?: (node: TNode) => ReactNode;
  getBadges?: (node: TNode, context: TreeCapabilityContext<TNode>) => TreeNodeBadge[];
  getSubItems?: (node: TNode, context: TreeCapabilityContext<TNode>) => TreeNodeSubItem[];
  getClassName?: (node: TNode, context: TreeCapabilityContext<TNode>) => string | null;
  getActions?: (node: TNode, context: TreeCapabilityContext<TNode>) => TreeNodeAction<TNode>[];
  getCapabilities: (node: TNode, context: TreeCapabilityContext<TNode>) => TreeNodeCapabilities;
};

export type TreeActionHandlers<TNode> = {
  onSelect?: (node: TNode) => void;
  onOpen?: (node: TNode) => void;
  onAddChild?: (node: TNode) => void;
  onRename?: (node: TNode) => void;
  onDelete?: (node: TNode) => void;
  onAction?: (actionId: string, node: TNode, action: TreeNodeAction<TNode>) => void;
  onContextMenu?: (node: TNode, event: MouseEvent) => void;
};

export const TREE_NODE_CAPABILITIES_NONE: TreeNodeCapabilities = {
  canExpand: false,
  canSelect: false,
  canOpen: false,
  canAddChild: false,
  canRename: false,
  canDelete: false,
  canDrag: false,
  canDropOn: false,
};
