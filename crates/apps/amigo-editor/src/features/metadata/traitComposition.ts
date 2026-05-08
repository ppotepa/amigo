import type {
  EditorMetadataCatalogDto,
  EditorMetadataTraitDescriptorDto,
  EditorMetadataTraitEditorSectionDescriptorDto,
} from "./editorMetadataTypes";
import {
  findMetadataTraitDescriptor,
  traitEditorSections,
} from "./editorMetadataTypes";

export type ComposedMetadataTrait = {
  kind: string;
  descriptor: EditorMetadataTraitDescriptorDto | null;
};

export type ComposedEditorSection = {
  id: string;
  traitKind: string;
  label: string;
  placement: string;
  description: string;
  priority: number;
  defaultExpanded: boolean;
};

export function composeTraits(
  catalog: EditorMetadataCatalogDto | null | undefined,
  traitKinds: readonly string[],
): ComposedMetadataTrait[] {
  return uniqueStrings(traitKinds).map((kind) => ({
    kind,
    descriptor: findMetadataTraitDescriptor(catalog, kind),
  }));
}

export function composeEditorSectionsForTraits(
  catalog: EditorMetadataCatalogDto | null | undefined,
  traitKinds: readonly string[],
  placement?: string,
): ComposedEditorSection[] {
  return composeTraits(catalog, traitKinds)
    .flatMap(({ kind, descriptor }) => {
      if (!descriptor) return [];
      return traitEditorSections(descriptor)
        .filter((section) => !placement || section.placement === placement)
        .map((section) => editorSectionFromTraitSection(kind, section));
    })
    .sort((left, right) => left.priority - right.priority || left.label.localeCompare(right.label));
}

function editorSectionFromTraitSection(
  traitKind: string,
  section: EditorMetadataTraitEditorSectionDescriptorDto,
): ComposedEditorSection {
  return {
    id: section.id,
    traitKind,
    label: section.label,
    placement: section.placement,
    description: section.description,
    priority: section.priority,
    defaultExpanded: section.defaultExpanded ?? section.default_expanded ?? false,
  };
}

function uniqueStrings(values: readonly string[]): string[] {
  return Array.from(new Set(values.filter(Boolean)));
}