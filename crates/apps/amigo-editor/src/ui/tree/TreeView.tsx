import { ChevronDown, ChevronRight, Plus, Trash2 } from "lucide-react";
import type {
  TreeActionHandlers,
  TreeNodeAction,
  TreeNodeAdapter,
  TreeNodeBadge,
  TreeNodeCapabilities,
  TreeNodeSubItem,
  TreeViewPreset,
} from "./treeTypes";

export type TreeViewProps<TNode> = {
  adapter: TreeNodeAdapter<TNode>;
  actions?: TreeActionHandlers<TNode>;
  className?: string;
  expandedIds: ReadonlySet<string>;
  nodes: readonly TNode[];
  onToggle: (id: string) => void;
  preset?: TreeViewPreset;
  selectedId?: string | null;
};

// @codemap anchor:shared-tree-view domain:workspace role:tree-view priority:P1 layer:app tags:tree,adapter,capabilities,actions
export function TreeView<TNode>({
  adapter,
  actions,
  className,
  expandedIds,
  nodes,
  onToggle,
  preset = "explorer",
  selectedId,
}: TreeViewProps<TNode>) {
  return (
    <div className={["tree-view", `tree-view-${preset}`, className].filter(Boolean).join(" ")}>
      {nodes.map((node) => (
        <TreeViewBranch
          key={adapter.getId(node)}
          actions={actions}
          adapter={adapter}
          depth={0}
          expandedIds={expandedIds}
          node={node}
          onToggle={onToggle}
          parent={null}
          selectedId={selectedId}
        />
      ))}
    </div>
  );
}

function TreeViewBranch<TNode>({
  actions,
  adapter,
  depth,
  expandedIds,
  node,
  onToggle,
  parent,
  selectedId,
}: {
  actions?: TreeActionHandlers<TNode>;
  adapter: TreeNodeAdapter<TNode>;
  depth: number;
  expandedIds: ReadonlySet<string>;
  node: TNode;
  onToggle: (id: string) => void;
  parent: TNode | null;
  selectedId?: string | null;
}) {
  const id = adapter.getId(node);
  const children = adapter.getChildren(node);
  const hasChildren = children.length > 0;
  const expanded = expandedIds.has(id);
  const selected = selectedId === id;
  const context = { depth, expanded, hasChildren, parent, selected };
  const capabilities = adapter.getCapabilities(node, context);

  return (
    <div className="tree-view-branch">
      <TreeViewRow
        actions={actions}
        adapter={adapter}
        badges={adapter.getBadges?.(node, context) ?? []}
        capabilities={capabilities}
        depth={depth}
        expanded={expanded}
        extraClassName={adapter.getClassName?.(node, context) ?? null}
        hasChildren={hasChildren}
        node={node}
        nodeActions={adapter.getActions?.(node, context)}
        onToggle={() => onToggle(id)}
        selected={selected}
        subItems={adapter.getSubItems?.(node, context) ?? []}
      />

      {hasChildren && expanded ? (
        <div className="tree-view-children">
          {children.map((child) => (
            <TreeViewBranch
              key={adapter.getId(child)}
              actions={actions}
              adapter={adapter}
              depth={depth + 1}
              expandedIds={expandedIds}
              node={child}
              onToggle={onToggle}
              parent={node}
              selectedId={selectedId}
            />
          ))}
        </div>
      ) : null}
    </div>
  );
}

function TreeViewRow<TNode>({
  actions,
  adapter,
  badges,
  capabilities,
  depth,
  expanded,
  extraClassName,
  hasChildren,
  node,
  nodeActions,
  onToggle,
  selected,
  subItems,
}: {
  actions?: TreeActionHandlers<TNode>;
  adapter: TreeNodeAdapter<TNode>;
  badges: TreeNodeBadge[];
  capabilities: TreeNodeCapabilities;
  depth: number;
  expanded: boolean;
  extraClassName: string | null;
  hasChildren: boolean;
  node: TNode;
  nodeActions?: TreeNodeAction<TNode>[];
  onToggle: () => void;
  selected: boolean;
  subItems: TreeNodeSubItem[];
}) {
  const label = adapter.getLabel(node);
  const icon = adapter.getIcon?.(node);
  const meta = adapter.getMeta?.(node);
  const canToggle = hasChildren && capabilities.canExpand;
  const visibleActions = (nodeActions ?? defaultActions(capabilities)).filter(
    (action) => action.visible ?? true,
  );

  function selectOrToggle() {
    if (capabilities.canSelect) {
      actions?.onSelect?.(node);
      return;
    }

    if (canToggle) onToggle();
  }

  return (
    <div
      className={["tree-view-item", selected ? "selected" : "", extraClassName].filter(Boolean).join(" ")}
      style={{ paddingLeft: `${depth * 14 + 8}px` }}
      onContextMenu={(event) => actions?.onContextMenu?.(node, event)}
    >
      <button
        className="tree-view-row"
        type="button"
        onClick={selectOrToggle}
        onDoubleClick={() => {
          if (capabilities.canOpen) actions?.onOpen?.(node);
        }}
      >
        <span
          className="tree-view-twist"
          aria-hidden="true"
          onClick={(event) => {
            if (!canToggle) return;
            event.stopPropagation();
            onToggle();
          }}
        >
          {canToggle ? expanded ? <ChevronDown size={13} /> : <ChevronRight size={13} /> : null}
        </span>

        {icon ? <span className="tree-view-icon">{icon}</span> : null}

        <span className="tree-view-label">
          <strong>{label}</strong>
          {subItems.length ? (
            <span className="tree-view-subitems">
              {subItems.filter((item) => item.visible ?? true).map((item) => (
                <small
                  key={item.key}
                  className={`tree-view-subitem tree-view-subitem-${item.tone ?? "muted"}`}
                  title={item.title}
                >
                  {item.label}
                </small>
              ))}
            </span>
          ) : null}
        </span>

        {meta ? <span className="tree-view-meta">{meta}</span> : null}

        {badges.length ? (
          <span className="tree-view-badges">
            {badges.filter((badge) => badge.visible ?? true).map((badge, index) => (
              <em
                key={index}
                className={`tree-view-badge tree-view-badge-${badge.tone ?? "default"}`}
                title={badge.title}
              >
                {badge.label}
              </em>
            ))}
          </span>
        ) : null}
      </button>

      {visibleActions.length ? (
        <span className="tree-view-actions">
          {visibleActions.map((action) => (
            <button
              key={action.id}
              className={`tree-view-action tree-view-action-${action.tone ?? "default"}`}
              disabled={action.disabled}
              title={action.title ?? action.label}
              type="button"
              onClick={(event) => {
                event.stopPropagation();
                invokeTreeAction(action, node, actions);
              }}
            >
              {action.icon ?? action.label}
            </button>
          ))}
        </span>
      ) : null}
    </div>
  );
}

function defaultActions<TNode>(capabilities: TreeNodeCapabilities): TreeNodeAction<TNode>[] {
  const result: TreeNodeAction<TNode>[] = [];

  if (capabilities.canAddChild) {
    result.push({ id: "addChild", label: "Add", icon: <Plus size={13} />, tone: "primary" });
  }

  if (capabilities.canDelete) {
    result.push({ id: "delete", label: "Delete", icon: <Trash2 size={12} />, tone: "danger" });
  }

  return result;
}

function invokeTreeAction<TNode>(
  action: TreeNodeAction<TNode>,
  node: TNode,
  handlers: TreeActionHandlers<TNode> | undefined,
) {
  handlers?.onAction?.(action.id, node, action);

  if (action.id === "addChild") handlers?.onAddChild?.(node);
  if (action.id === "rename") handlers?.onRename?.(node);
  if (action.id === "delete") handlers?.onDelete?.(node);
  if (action.id === "open") handlers?.onOpen?.(node);
}
