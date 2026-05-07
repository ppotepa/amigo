import type {
  EditorComponentDescriptorDto,
  EditorMetadataCatalogDto,
} from "./editorMetadataTypes";
import { componentTypeName } from "./editorMetadataTypes";

type GenericPropertiesPanelProps = {
  metadata: EditorMetadataCatalogDto | null;
  componentTypes: string[];
  resolveValue?: (componentType: string, path: string) => unknown;
};

export function GenericPropertiesPanel({
  metadata,
  componentTypes,
  resolveValue,
}: GenericPropertiesPanelProps) {
  if (!metadata) {
    return <div className="metadata-empty">Metadata catalog is not loaded.</div>;
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
        return (
          <section className="generic-properties-section" key={typeName}>
            <header className="generic-properties-section-header">
              <span>{descriptor.label}</span>
              <small>{typeName}</small>
            </header>
            <div className="generic-properties-grid">
              {descriptor.properties.map((property) => {
                const value = resolveValue?.(typeName, property.path);
                return (
                  <label
                    className="generic-property-row"
                    key={`${typeName}:${property.path}`}
                  >
                    <span className="generic-property-label">{property.label}</span>
                    <span className="generic-property-value">
                      {formatValue(value)}
                    </span>
                    <span className="generic-property-editor">
                      {property.editor}
                    </span>
                  </label>
                );
              })}
            </div>
          </section>
        );
      })}
    </div>
  );
}

function findComponentDescriptor(
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

function formatValue(value: unknown): string {
  if (value === undefined) return "-";
  if (value === null) return "null";
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  return JSON.stringify(value);
}
