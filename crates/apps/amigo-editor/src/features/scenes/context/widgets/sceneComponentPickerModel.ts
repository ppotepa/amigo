import type { EditorComponentDescriptorDto } from "../../../metadata/editorMetadataTypes";
import { componentOwnerScopes, componentTypeName } from "../../../metadata/editorMetadataTypes";

export function filterSceneComponentDescriptors(
  descriptors: EditorComponentDescriptorDto[],
  query: string,
): EditorComponentDescriptorDto[] {
  const normalizedQuery = query.trim().toLowerCase();
  if (!normalizedQuery) return descriptors;

  return descriptors.filter((descriptor) => {
    const haystack = [
      descriptor.kind,
      componentTypeName(descriptor),
      descriptor.label,
      ...descriptor.domains,
    ].join(" ").toLowerCase();
    return haystack.includes(normalizedQuery);
  });
}

export function canAttachDescriptorToScene(
  descriptor: EditorComponentDescriptorDto,
): { allowed: boolean; reason?: string } {
  const scopes = componentOwnerScopes(descriptor);
  if (scopes.includes("scene")) return { allowed: true };
  return { allowed: false, reason: "Component is not allowed on scenes." };
}

export function groupSceneComponentDescriptors(
  descriptors: EditorComponentDescriptorDto[],
): Array<{ id: string; label: string; descriptors: EditorComponentDescriptorDto[] }> {
  const groups = new Map<string, EditorComponentDescriptorDto[]>();
  for (const descriptor of descriptors) {
    const id = descriptor.domains[0] ?? "Other";
    const bucket = groups.get(id) ?? [];
    bucket.push(descriptor);
    groups.set(id, bucket);
  }

  return Array.from(groups.entries())
    .map(([id, values]) => ({
      id,
      label: id,
      descriptors: values.sort((left, right) => left.label.localeCompare(right.label)),
    }))
    .sort((left, right) => left.label.localeCompare(right.label));
}
