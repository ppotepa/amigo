import type {
  AssetRegistryDto,
  EditorResolvedAssetRefDto,
  EditorSceneComponentInstanceDto,
} from "../../api/dto";
import type {
  EditorComponentDescriptorDto,
  EditorMetadataCatalogDto,
} from "./editorMetadataTypes";
import { componentTypeName, propertyGroup, propertyTraitKind } from "./editorMetadataTypes";
import { AssetRefField } from "./AssetRefField";

type GenericPropertiesPanelProps = {
  metadata?: EditorMetadataCatalogDto | null;
  component?: EditorSceneComponentInstanceDto | null;
  components?: EditorSceneComponentInstanceDto[];
  componentTypes?: string[];
  assetRegistry?: AssetRegistryDto | null;
  resolveValue?: (componentType: string, path: string) => unknown;
  onRequestEdit?: (request: {
    component: EditorSceneComponentInstanceDto;
    path: string;
    value: unknown;
  }) => void;
  onRequestAssetAssign?: (request: {
    component: EditorSceneComponentInstanceDto;
    path: string;
    assetKey: string | null;
  }) => void;
};

type ReadOnlyPropertyRow = {
  path: string;
  label: string;
  value: unknown;
  editor?: string;
  access?: string;
  exists?: boolean;
  traitKind?: string | null;
  group: string;
  assetRef?: EditorResolvedAssetRefDto | null;
};

type ReadOnlyPropertyGroup = {
  id: string;
  label: string;
  traitKind?: string | null;
  rows: ReadOnlyPropertyRow[];
};

// @codemap:P1 generic-properties-readonly
// Metadata-driven read-only property renderer; future writes should go through generic backend patch commands, not component-specific React panels.
export function GenericPropertiesPanel({
  metadata = null,
  component = null,
  components = [],
  componentTypes = [],
  assetRegistry,
  resolveValue,
  onRequestEdit,
  onRequestAssetAssign,
}: GenericPropertiesPanelProps) {
  const resolvedComponents = component ? [component] : components;

  if (resolvedComponents.length) {
    return (
      <div className="generic-properties-panel">
        {resolvedComponents.map((item) => (
          <ComponentInstanceProperties
            assetRegistry={assetRegistry}
            component={item}
            key={`${item.typeName}:${item.componentIndex}`}
            onRequestAssetAssign={onRequestAssetAssign}
            onRequestEdit={onRequestEdit}
          />
        ))}
      </div>
    );
  }

  if (!metadata) {
    return <div className="metadata-empty">No component instance values available.</div>;
  }

  const descriptors = componentTypes
    .map((type) => findComponentDescriptor(metadata, type))
    .filter(Boolean) as EditorComponentDescriptorDto[];

  if (descriptors.length === 0) {
    return (
      <div className="metadata-empty">
        No metadata-backed properties for this selection.
      </div>
    );
  }

  return (
    <div className="generic-properties-panel">
      {descriptors.map((descriptor) => {
        const typeName = componentTypeName(descriptor);
        const rows = descriptor.properties.map((property) => ({
          path: property.path,
          label: property.label,
          editor: property.editor,
          access: property.access,
          exists: undefined,
          traitKind: propertyTraitKind(property),
          group: propertyGroup(property),
          value: resolveValue?.(typeName, property.path),
        }));

        return (
          <ReadOnlyPropertySection
            key={typeName}
            rows={rows}
            subtitle={typeName}
            title={descriptor.label}
          />
        );
      })}
    </div>
  );
}

function ComponentInstanceProperties({
  assetRegistry,
  component,
  onRequestAssetAssign,
  onRequestEdit,
}: {
  assetRegistry?: AssetRegistryDto | null;
  component: EditorSceneComponentInstanceDto;
  onRequestEdit?: GenericPropertiesPanelProps["onRequestEdit"];
  onRequestAssetAssign?: GenericPropertiesPanelProps["onRequestAssetAssign"];
}) {
  const rows = componentPropertyRows(component);

  return (
    <ReadOnlyPropertySection
      assetRegistry={assetRegistry}
      component={component}
      onRequestAssetAssign={onRequestAssetAssign}
      onRequestEdit={onRequestEdit}
      rows={rows}
      subtitle={`#${component.componentIndex} ${component.yamlPath}`}
      title={component.label || component.typeName}
    />
  );
}

function ReadOnlyPropertySection({
  assetRegistry,
  component,
  onRequestAssetAssign,
  onRequestEdit: _onRequestEdit,
  rows,
  subtitle,
  title,
}: {
  assetRegistry?: AssetRegistryDto | null;
  component?: EditorSceneComponentInstanceDto;
  onRequestEdit?: GenericPropertiesPanelProps["onRequestEdit"];
  onRequestAssetAssign?: GenericPropertiesPanelProps["onRequestAssetAssign"];
  rows: ReadOnlyPropertyRow[];
  subtitle: string;
  title: string;
}) {
  const groups = groupRowsByTraitAndGroup(rows);

  return (
    <section className="generic-properties-section">
      <header className="generic-properties-section-header">
        <span>{title}</span>
        <small>{subtitle}</small>
      </header>

      {rows.length ? (
        <div className="generic-property-groups">
          {groups.map((group) => (
            <section
              className="generic-property-group"
              key={`${group.traitKind ?? "none"}:${group.id}`}
            >
              <header className="generic-property-group-header">
                <span>{group.label}</span>
                {group.traitKind ? <small>{group.traitKind}</small> : null}
              </header>
              <div className="generic-properties-grid">
                {group.rows.map((property) => (
                  <div className="generic-property-row" key={property.path}>
                    <span className="generic-property-label">{property.label}</span>
                    <span className="generic-property-value">
                      {property.assetRef && component && onRequestAssetAssign ? (
                        <AssetRefField
                          value={property.assetRef.value}
                          assetDomain={property.assetRef.domain}
                          required={property.assetRef.required}
                          registry={assetRegistry}
                          onAssign={(assetKey) => onRequestAssetAssign({
                            component,
                            path: property.assetRef?.fieldPath ?? property.path,
                            assetKey,
                          })}
                        />
                      ) : property.exists === false ? "-" : formatPropertyValue(property.value)}
                    </span>
                    <span className="generic-property-editor">
                      {[property.editor, property.access].filter(Boolean).join(" / ")}
                    </span>
                  </div>
                ))}
              </div>
            </section>
          ))}
        </div>
      ) : (
        <p className="muted workspace-note">No descriptor properties yet.</p>
      )}
    </section>
  );
}

function componentPropertyRows(component: EditorSceneComponentInstanceDto): ReadOnlyPropertyRow[] {
  if (component.properties.length) {
    return component.properties.map((property) => ({
      path: property.path,
      label: property.label,
      value: property.value,
      editor: property.editor,
      access: property.access,
      exists: property.exists,
      traitKind: property.traitKind ?? null,
      group: property.group ?? "missing.group",
      assetRef: assetRefForProperty(component, property.path),
    }));
  }

  const raw = asRecord(component.values);
  if (!raw) return [];

  return Object.entries(raw)
    .filter(([key]) => key !== "type" && key !== "kind")
    .map(([key, value]) => ({
      path: key,
      label: key,
      value,
      editor: "ReadOnly",
      access: "ReadOnly",
      exists: true,
      traitKind: null,
      group: "raw",
      assetRef: assetRefForProperty(component, key),
    }));
}

function assetRefForProperty(
  component: EditorSceneComponentInstanceDto,
  path: string,
): EditorResolvedAssetRefDto | null {
  return component.assetRefs.find((assetRef) => assetRef.fieldPath === path) ?? null;
}

function groupRowsByTraitAndGroup(rows: ReadOnlyPropertyRow[]): ReadOnlyPropertyGroup[] {
  const groups = new Map<string, ReadOnlyPropertyGroup>();

  for (const row of rows) {
    const groupId = row.group ?? "missing.group";
    const traitKind = row.traitKind ?? null;
    const key = `${traitKind ?? "none"}:${groupId}`;

    if (!groups.has(key)) {
      groups.set(key, {
        id: groupId,
        label: groupId,
        traitKind,
        rows: [],
      });
    }

    groups.get(key)?.rows.push(row);
  }

  return Array.from(groups.values());
}

export function findComponentDescriptor(
  metadata: EditorMetadataCatalogDto,
  typeName: string,
): EditorComponentDescriptorDto | null {
  return (
    metadata.components.find((descriptor) => {
      const descriptorType = componentTypeName(descriptor);
      return descriptor.kind === typeName || descriptorType === typeName;
    }) ?? null
  );
}

function asRecord(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return null;
  }

  return value as Record<string, unknown>;
}

export function formatPropertyValue(value: unknown): string {
  if (value === undefined || value === null) return "-";
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  return JSON.stringify(value);
}
