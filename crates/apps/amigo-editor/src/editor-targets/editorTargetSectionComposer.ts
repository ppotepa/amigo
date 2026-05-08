import type { EditorMetadataCatalogDto } from "../features/metadata/editorMetadataTypes";
import { composeEditorSectionsForTraits } from "../features/metadata/traitComposition";
import type { ResolvedEditorTarget } from "./editorTargetTypes";

export type EditorTargetComposedSection = {
  id: string;
  label: string;
  placement: string;
  traitKind: string;
  description: string;
  priority: number;
  defaultExpanded: boolean;
};

export function composeSectionsForResolvedTarget(
  target: ResolvedEditorTarget,
  metadata: EditorMetadataCatalogDto | null | undefined,
): EditorTargetComposedSection[] {
  return composeEditorSectionsForTraits(metadata, target.metadataTraits).map((section) => ({
    id: section.id,
    label: section.label,
    placement: section.placement,
    traitKind: section.traitKind,
    description: section.description,
    priority: section.priority,
    defaultExpanded: section.defaultExpanded,
  }));
}