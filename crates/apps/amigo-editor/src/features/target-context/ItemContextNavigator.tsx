import type { EditorSceneComponentInstanceDto } from "../../api/dto";
import type {
  EditorTargetIntent,
  EditorTargetRef,
} from "../../editor-targets/editorTargetTypes";
import type {
  EditorComponentDescriptorDto,
  EditorMetadataCatalogDto,
  EditorMetadataTraitDescriptorDto,
} from "../metadata/editorMetadataTypes";
import {
  assetRefs,
  boundsPolicy,
  componentMetadataTraits,
  componentTypeName,
  controlPatchOp,
  controlTargetScope,
  editorControls,
  findMetadataTraitDescriptor,
  patchOps,
  patchOpTargetScope,
  traitEditorSections,
  traitPropertyGroups,
} from "../metadata/editorMetadataTypes";

type ItemContextNavigatorProps = {
  metadata: EditorMetadataCatalogDto | null;
  target: EditorTargetRef;
  components?: EditorSceneComponentInstanceDto[];
  componentTypes?: string[];
  onActivateTarget?: (targetRef: EditorTargetRef, intent?: EditorTargetIntent) => void;
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
    | "diagnostic"
    | "trait"
    | "propertyGroup"
    | "editorSection";
  subtitle?: string;
  targetRef?: EditorTargetRef;
  children?: ItemContextNode[];
};

export function ItemContextNavigator({
  metadata,
  target,
  components = [],
  componentTypes = [],
  onActivateTarget,
}: ItemContextNavigatorProps) {
  const tree = buildComponentContextTree(metadata, target, components, componentTypes);

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
  target: EditorTargetRef,
  components: EditorSceneComponentInstanceDto[],
  componentTypes: string[],
): ItemContextNode[] {
  const componentNodes = components.length
    ? components.map((component) => componentInstanceNode(component, metadata, target))
    : componentTypes.map((typeName) => {
        const descriptor = findComponentDescriptor(metadata, typeName);
        return componentTypeNode(typeName, descriptor);
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
      subtitle: `${componentNodes.length}`,
      children: componentNodes,
    },
  ];
}

function componentInstanceNode(
  component: EditorSceneComponentInstanceDto,
  metadata: EditorMetadataCatalogDto | null,
  target: EditorTargetRef,
): ItemContextNode {
  const descriptor = findComponentDescriptor(metadata, component.typeName);
  const traitNodes = componentTraitNodes(component, metadata);
  const propertyNodes = component.properties.length
    ? component.properties.map((property) => ({
        id: `component:${component.componentIndex}:property:${property.path}`,
        label: property.label,
        kind: "property" as const,
        subtitle: `${property.path} = ${formatReadOnlyValue(property.value)}`,
      }))
    : rawValueNodes(component);
  const assetRefNodes = component.assetRefs.map((ref) => ({
    id: `component:${component.componentIndex}:assetRef:${ref.fieldPath}`,
    label: ref.fieldPath,
    kind: "assetRef" as const,
    subtitle: ref.value
      ? `${ref.domain} -> ${ref.value}`
      : ref.required
        ? `${ref.domain} required`
        : ref.domain,
  }));
  const diagnosticsNodes = component.diagnostics.map((diagnostic, index) => ({
    id: `component:${component.componentIndex}:diagnostic:${diagnostic.code}:${index}`,
    label: diagnostic.code,
    kind: "diagnostic" as const,
    subtitle: diagnostic.message,
  }));

  return {
    id: `component:${component.componentIndex}:${component.typeName}`,
    label: component.label || component.typeName,
    kind: "component",
    targetRef: componentTargetFor(target, component),
    subtitle: `#${component.componentIndex} ${component.yamlPath}`,
    children: [
      {
        id: `component:${component.componentIndex}:traits`,
        label: "Traits",
        kind: "group",
        subtitle: traitNodes.length ? `${traitNodes.length}` : "none",
        children: traitNodes,
      },
      {
        id: `component:${component.componentIndex}:values`,
        label: "Properties",
        kind: "group",
        subtitle: propertyNodes.length ? `${propertyNodes.length}` : "none",
        children: propertyNodes,
      },
      {
        id: `component:${component.componentIndex}:assetRefs`,
        label: "Asset References",
        kind: "group",
        subtitle: assetRefNodes.length ? `${assetRefNodes.length}` : "none",
        children: assetRefNodes,
      },
      ...(diagnosticsNodes.length
        ? [{
            id: `component:${component.componentIndex}:diagnostics`,
            label: "Diagnostics",
            kind: "group" as const,
            subtitle: `${diagnosticsNodes.length}`,
            children: diagnosticsNodes,
          }]
        : []),
      ...(descriptor ? descriptorDetailNodes(component.typeName, descriptor) : []),
    ],
  };
}

function componentTraitNodes(
  component: EditorSceneComponentInstanceDto,
  metadata: EditorMetadataCatalogDto | null | undefined,
): ItemContextNode[] {
  return (component.metadataTraits ?? []).map((traitKind) => {
    const traitDescriptor = findMetadataTraitDescriptor(metadata, traitKind);
    const matchingProperties = component.properties.filter(
      (property) => property.traitKind === traitKind,
    );
    const matchingAssetRefs = component.assetRefs.filter(
      (assetRef) => assetRef.traitKind === traitKind,
    );

    return {
      id: `component:${component.componentIndex}:trait:${traitKind}`,
      label: traitDescriptor?.label ?? traitKind,
      kind: "trait" as const,
      subtitle: traitDescriptor?.description ?? undefined,
      children: [
        ...traitEditorSectionNodes(component, traitKind, traitDescriptor),
        ...traitPropertyGroupNodes(component, traitKind, traitDescriptor, matchingProperties),
        ...traitAssetRefNodes(component, traitKind, matchingAssetRefs),
      ],
    };
  });
}

function traitEditorSectionNodes(
  component: EditorSceneComponentInstanceDto,
  traitKind: string,
  traitDescriptor: EditorMetadataTraitDescriptorDto | null,
): ItemContextNode[] {
  if (!traitDescriptor) return [];

  return traitEditorSections(traitDescriptor).map((section) => ({
    id: `component:${component.componentIndex}:trait:${traitKind}:section:${section.id}`,
    label: section.label,
    kind: "editorSection" as const,
    subtitle: `${section.placement} / priority ${section.priority}`,
  }));
}

function traitPropertyGroupNodes(
  component: EditorSceneComponentInstanceDto,
  traitKind: string,
  traitDescriptor: EditorMetadataTraitDescriptorDto | null,
  properties: EditorSceneComponentInstanceDto["properties"],
): ItemContextNode[] {
  if (!traitDescriptor) {
    return properties.map((property) => propertyNode(component, property));
  }

  return traitPropertyGroups(traitDescriptor).map((group) => {
    const groupProperties = properties.filter((property) => property.group === group.id);

    return {
      id: `component:${component.componentIndex}:trait:${traitKind}:group:${group.id}`,
      label: group.label,
      kind: "propertyGroup" as const,
      subtitle: group.description,
      children: groupProperties.map((property) => propertyNode(component, property)),
    };
  });
}

function traitAssetRefNodes(
  component: EditorSceneComponentInstanceDto,
  traitKind: string,
  assetRefs: EditorSceneComponentInstanceDto["assetRefs"],
): ItemContextNode[] {
  if (!assetRefs.length) return [];

  return [
    {
      id: `component:${component.componentIndex}:trait:${traitKind}:assetRefs`,
      label: "Asset References",
      kind: "group" as const,
      subtitle: `${assetRefs.length}`,
      children: assetRefs.map((ref) => ({
        id: `component:${component.componentIndex}:assetRef:${ref.fieldPath}`,
        label: ref.fieldPath,
        kind: "assetRef" as const,
        subtitle: ref.value
          ? `${ref.domain} -> ${ref.value}`
          : ref.required
            ? `${ref.domain} required`
            : ref.domain,
      })),
    },
  ];
}

function propertyNode(
  component: EditorSceneComponentInstanceDto,
  property: EditorSceneComponentInstanceDto["properties"][number],
): ItemContextNode {
  return {
    id: `component:${component.componentIndex}:property:${property.path}`,
    label: property.label,
    kind: "property",
    subtitle: `${property.path} = ${formatReadOnlyValue(property.value)}`,
  };
}

function componentTargetFor(
  target: EditorTargetRef,
  component: EditorSceneComponentInstanceDto,
): EditorTargetRef | undefined {
  if (target.kind === "sceneEntity") {
    return {
      kind: "component",
      sceneId: target.sceneId,
      entityId: target.entityId,
      componentIndex: component.componentIndex,
      componentType: component.typeName,
    };
  }

  if (target.kind === "component") {
    return {
      kind: "component",
      sceneId: target.sceneId,
      entityId: target.entityId,
      componentIndex: component.componentIndex,
      componentType: component.typeName,
    };
  }

  return undefined;
}

function rawValueNodes(component: EditorSceneComponentInstanceDto): ItemContextNode[] {
  const values = asRecord(component.values);
  if (!values) return [];

  return Object.entries(values)
    .filter(([key]) => key !== "type" && key !== "kind")
    .map(([key, value]) => ({
      id: `component:${component.componentIndex}:raw:${key}`,
      label: key,
      kind: "property" as const,
      subtitle: formatReadOnlyValue(value),
    }));
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

function componentTypeNode(
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
    children: descriptorDetailNodes(typeName, descriptor),
  };
}

function descriptorDetailNodes(
  typeName: string,
  descriptor: EditorComponentDescriptorDto,
): ItemContextNode[] {
  const bounds = boundsPolicy(descriptor);

  return [
    {
      id: `component:${typeName}:metadataTraits`,
      label: "Metadata Traits",
      kind: "group",
      children: componentMetadataTraits(descriptor).map((traitKind) => ({
        id: `component:${typeName}:metadataTrait:${traitKind}`,
        label: traitKind,
        kind: "trait",
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
      id: `component:${typeName}:descriptorProperties`,
      label: "Descriptor Properties",
      kind: "group",
      children: descriptor.properties.map((property) => ({
        id: `component:${typeName}:property:${property.path}`,
        label: property.label,
        kind: "property",
        subtitle: property.path,
      })),
    },
    {
      id: `component:${typeName}:descriptorAssetRefs`,
      label: "Descriptor Asset References",
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
  ];
}

function ContextNodeView({
  node,
  depth,
  onActivateTarget,
}: {
  node: ItemContextNode;
  depth: number;
  onActivateTarget?: (targetRef: EditorTargetRef, intent?: EditorTargetIntent) => void;
}) {
  const hasChildren = Boolean(node.children?.length);
  return (
    <div className="item-context-node" style={{ paddingLeft: depth * 12 }}>
      <button
        type="button"
        className={`item-context-node-button item-context-node-${node.kind}`}
        onClick={() => node.targetRef && onActivateTarget?.(node.targetRef, "select")}
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

function asRecord(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return null;
  }

  return value as Record<string, unknown>;
}

function formatReadOnlyValue(value: unknown): string {
  if (value === undefined || value === null) return "-";
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  return JSON.stringify(value);
}
