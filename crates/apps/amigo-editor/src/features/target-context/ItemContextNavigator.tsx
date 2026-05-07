import type {
  EditorComponentDescriptorDto,
  EditorMetadataCatalogDto,
} from "../metadata/editorMetadataTypes";
import { assetRefs, componentTypeName } from "../metadata/editorMetadataTypes";

type ItemContextNavigatorProps = {
  metadata: EditorMetadataCatalogDto | null;
  target: unknown;
  componentTypes?: string[];
  onActivateTarget?: (targetRef: unknown, intent?: string) => void;
};

type ItemContextNode = {
  id: string;
  label: string;
  kind: "group" | "component" | "property" | "assetRef" | "target" | "diagnostic";
  subtitle?: string;
  targetRef?: unknown;
  children?: ItemContextNode[];
};

export function ItemContextNavigator({
  metadata,
  target,
  componentTypes = [],
  onActivateTarget,
}: ItemContextNavigatorProps) {
  const tree = buildComponentContextTree(metadata, target, componentTypes);

  return (
    <div className="item-context-navigator">
      <header className="item-context-nav-header">
        <span className="item-context-nav-title">Item Context</span>
        <span className="item-context-nav-subtitle">selection drilldown</span>
      </header>
      <div className="item-context-tree">
        {tree.map((node) => (
          <ContextNodeView
            key={node.id}
            node={node}
            depth={0}
            onActivateTarget={onActivateTarget}
          />
        ))}
      </div>
    </div>
  );
}

function buildComponentContextTree(
  metadata: EditorMetadataCatalogDto | null,
  target: unknown,
  componentTypes: string[],
): ItemContextNode[] {
  const componentNodes = componentTypes.map((typeName) => {
    const descriptor =
      metadata?.components.find(
        (item) => componentTypeName(item) === typeName || item.kind === typeName,
      ) ?? null;
    return componentNode(typeName, descriptor);
  });

  return [
    {
      id: "target",
      label: "Target",
      kind: "target",
      targetRef: target,
    },
    {
      id: "components",
      label: "Components",
      kind: "group",
      children: componentNodes,
    },
  ];
}

function componentNode(
  typeName: string,
  descriptor: EditorComponentDescriptorDto | null,
): ItemContextNode {
  if (!descriptor) {
    return {
      id: `component:${typeName}`,
      label: typeName,
      kind: "component",
      subtitle: "No metadata descriptor",
    };
  }

  return {
    id: `component:${typeName}`,
    label: descriptor.label,
    kind: "component",
    subtitle: componentTypeName(descriptor),
    children: [
      {
        id: `component:${typeName}:properties`,
        label: "Properties",
        kind: "group",
        children: descriptor.properties.map((property) => ({
          id: `component:${typeName}:property:${property.path}`,
          label: property.label,
          kind: "property",
          subtitle: property.path,
        })),
      },
      {
        id: `component:${typeName}:assetRefs`,
        label: "Asset References",
        kind: "group",
        children: assetRefs(descriptor).map((ref) => ({
          id: `component:${typeName}:assetRef:${ref.fieldPath ?? ref.field_path}`,
          label: ref.fieldPath ?? ref.field_path ?? "asset ref",
          kind: "assetRef",
          subtitle: ref.domain,
        })),
      },
    ],
  };
}

function ContextNodeView({
  node,
  depth,
  onActivateTarget,
}: {
  node: ItemContextNode;
  depth: number;
  onActivateTarget?: (targetRef: unknown, intent?: string) => void;
}) {
  const hasChildren = Boolean(node.children?.length);
  return (
    <div className="item-context-node" style={{ paddingLeft: depth * 12 }}>
      <button
        type="button"
        className={`item-context-node-button item-context-node-${node.kind}`}
        onClick={() => node.targetRef && onActivateTarget?.(node.targetRef, "inspect")}
      >
        <span className="item-context-node-marker">{hasChildren ? "v" : "-"}</span>
        <span className="item-context-node-label">{node.label}</span>
        {node.subtitle ? (
          <small className="item-context-node-subtitle">{node.subtitle}</small>
        ) : null}
      </button>
      {node.children?.map((child) => (
        <ContextNodeView
          key={child.id}
          node={child}
          depth={depth + 1}
          onActivateTarget={onActivateTarget}
        />
      ))}
    </div>
  );
}
