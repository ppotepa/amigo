import type {
  EditorMetadataCatalogDto,
  EditorMetadataTraitDescriptorDto,
} from ./editorMetadataTypes;
import { findMetadataTraitDescriptor } from ./editorMetadataTypes;

export type ComposedMetadataTrait = {
  kind: string;
  descriptor: EditorMetadataTraitDescriptorDto | null;
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

function uniqueStrings(values: readonly string[]): string[] {
  return Array.from(new Set(values.filter(Boolean)));
}