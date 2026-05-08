import type {
  EditorComponentDescriptorDto,
  EditorMetadataCatalogDto,
} from "../metadata/editorMetadataTypes";
import {
  assetRefs,
  boundsPolicy,
  componentTypeName,
  controlPatchOp,
  controlTargetScope,
  editorControls,
  patchOps,
  patchOpTargetScope,
} from "../metadata/editorMetadataTypes";

type ItemContextNavigatorProps = {
  metadata: EditorMetadataCatalogDto | null;
  target: unknown;
  componentTypes?: string[];
  onActivateTarget?: (targetRef: unknown, intent?: string) => void;
};

type ItemContextNode = {
  id: string;
  label: string;
  kind:
    | "group"
    | "component"
    | "property"
    | "assetRef"
    | "control"
    | "patchOp"
    | "capability"
    | "policy"
    | "target"
    | "diagnostic";
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
    const descriptor = findComponentDescriptor(metadata, typeName);
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

function findComponentDescriptor(
  metadata: EditorMetadataCatalogDto | null,
  typeName: string,
): EditorComponentDescriptorDto | null {
  return (
    metadata?.components.find(
      (item) => componentTypeName(item) === typeName || item.kind === typeName,
    ) ?? null
  );
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

  const descriptorType = componentTypeName(descriptor);
  const bounds = boundsPolicy(descriptor);

  return {
    id: `component:${typeName}`,
    label: descriptor.label,
    kind: "component",
    subtitle: descriptorType,
    children: [
      {
        id: `component:${typeName}:capabilities`,
        label: "Capabilities",
        kind: "group",
        children: descriptor.capabilities.map((capability) => ({
          id: `component:${typeName}:capability:${capability}`,
          label: capability,
          kind: "capability",
        })),
      },
      {
        id: `component:${typeName}:policies`,
        label: "Policies",
        kind: "group",
        children: [
          {
            id: `component:${typeName}:policy:transform`,
            label: "Transform",
            kind: "policy",
            subtitle: descriptor.transformPolicy ?? descriptor.transform_policy ?? "None",
          },
          {
            id: `component:${typeName}:policy:bounds`,
            label: "Bounds",
            kind: "policy",
            subtitle: bounds.field ? `${bounds.kind}.${bounds.field}` : bounds.kind,
          },
        ],
      },
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
          subtitle: ref.required ? `${ref.domain} required` : ref.domain,
        })),
      },
      {
        id: `component:${typeName}:controls`,
        label: "Controls",
        kind: "group",
        children: editorControls(descriptor).map((control) => {
          const patchOp = controlPatchOp(control);
          return {
            id: `component:${typeName}:control:${control.kind}`,
            label: control.kind,
            kind: "control",
            subtitle: patchOp
              ? `${controlTargetScope(control)} -> ${patchOp}`
              : controlTargetScope(control),
            children: control.handles.map((handle) => ({
              id: `component:${typeName}:control:${control.kind}:handle:${handle}`,
              label: handle,
              kind: "control",
            })),
          } satisfies ItemContextNode;
        }),
      },
      {
        id: `component:${typeName}:patchOps`,
        label: "Patch Operations",
        kind: "group",
        children: patchOps(descriptor).map((op) => ({
          id: `component:${typeName}:patchOp:${op.kind}`,
          label: op.kind,
          kind: "patchOp",
          subtitle: `${patchOpTargetScope(op)} -> ${op.persistence}`,
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
