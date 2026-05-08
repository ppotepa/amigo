export type EditorMetadataCatalogDto = {
  targetKinds?: EditorTargetKindDescriptorDto[];
  target_kinds?: EditorTargetKindDescriptorDto[];
  components: EditorComponentDescriptorDto[];
  metadataTraits?: EditorMetadataTraitDescriptorDto[];
  metadata_traits?: EditorMetadataTraitDescriptorDto[];
  assetKinds?: EditorAssetKindDescriptorDto[];
  asset_kinds?: EditorAssetKindDescriptorDto[];
  documentKinds?: EditorDocumentKindDescriptorDto[];
  document_kinds?: EditorDocumentKindDescriptorDto[];
  controls?: EditorControlDescriptorDto[];
  patchOps?: EditorPatchOpDescriptorDto[];
  patch_ops?: EditorPatchOpDescriptorDto[];
};

export type EditorTargetKindDescriptorDto = {
  kind: string;
  label: string;
  sourceSurfaces?: string[];
  source_surfaces?: string[];
  allowedIntents?: string[];
  allowed_intents?: string[];
  capabilities: string[];
  primaryContextRole?: string;
  primary_context_role?: string;
  relatedContextRole?: string;
  related_context_role?: string;
};

export type EditorComponentDescriptorDto = {
  kind: string;
  typeName?: string;
  type_name?: string;
  label: string;
  domains: string[];
  capabilities: string[];
  metadataTraits?: string[];
  metadata_traits?: string[];
  assetRefs?: EditorAssetRefDescriptorDto[];
  asset_refs?: EditorAssetRefDescriptorDto[];
  properties: EditorPropertyDescriptorDto[];
  transformPolicy?: string;
  transform_policy?: string;
  boundsPolicy?: EditorBoundsPolicyDto;
  bounds_policy?: EditorBoundsPolicyDto;
  editorControls?: EditorControlRefDto[];
  editor_controls?: EditorControlRefDto[];
  patchOps?: EditorPatchOpRefDto[];
  patch_ops?: EditorPatchOpRefDto[];
};

export type EditorAssetKindDescriptorDto = {
  kind: string;
  label: string;
  domain: string;
  previewKind?: string;
  preview_kind?: string;
  documentPolicy?: string;
  document_policy?: string;
  usagesSupported?: boolean;
  usages_supported?: boolean;
  canCreate?: boolean;
  can_create?: boolean;
};

export type EditorDocumentKindDescriptorDto = {
  kind: string;
  label: string;
  extensions: string[];
  targetKind?: string;
  target_kind?: string;
  patchSink?: string;
  patch_sink?: string;
  supportsReload?: boolean;
  supports_reload?: boolean;
  supportsValidation?: boolean;
  supports_validation?: boolean;
};

export type EditorControlDescriptorDto = {
  kind: string;
  label: string;
  targetScope?: string;
  target_scope?: string;
  handles: string[];
  defaultPatchOp?: string | null;
  default_patch_op?: string | null;
  requiresBounds?: boolean;
  requires_bounds?: boolean;
  viewportVisible?: boolean;
  viewport_visible?: boolean;
};

export type EditorPatchOpDescriptorDto = {
  kind: string;
  label: string;
  targetScope?: string;
  target_scope?: string;
  valueKind?: string;
  value_kind?: string;
  persistence: string;
  risk: string;
};

export type EditorControlRefDto = {
  kind: string;
  targetScope?: string;
  target_scope?: string;
  handles: string[];
  patchOp?: string | null;
  patch_op?: string | null;
};

export type EditorPatchOpRefDto = {
  kind: string;
  targetScope?: string;
  target_scope?: string;
  persistence: string;
};

export type EditorBoundsPolicyDto = {
  kind: string;
  field?: string | null;
};

export type EditorAssetRefDescriptorDto = {
  fieldPath?: string;
  field_path?: string;
  domain: string;
  required: boolean;
  traitKind?: string;
  trait_kind?: string;
  group?: string;
};

export type EditorPropertyDescriptorDto = {
  path: string;
  label: string;
  valueKind?: string;
  value_kind?: string;
  access: string;
  editor: string;
  assetDomain?: string | null;
  asset_domain?: string | null;
  traitKind?: string | null;
  trait_kind?: string | null;
  group?: string;
  patchOp?: string | null;
  patch_op?: string | null;
};

export type EditorMetadataTraitDescriptorDto = {
  kind: string;
  label: string;
  description: string;
  appliesTo?: string[];
  applies_to?: string[];
  expectedYamlShapes?: string[];
  expected_yaml_shapes?: string[];
  propertyGroups?: EditorMetadataTraitPropertyGroupDescriptorDto[];
  property_groups?: EditorMetadataTraitPropertyGroupDescriptorDto[];
  editorSections?: EditorMetadataTraitEditorSectionDescriptorDto[];
  editor_sections?: EditorMetadataTraitEditorSectionDescriptorDto[];
  controls?: string[];
  patchOps?: string[];
  patch_ops?: string[];
  diagnostics?: EditorMetadataTraitDiagnosticDescriptorDto[];
};

export type EditorMetadataTraitPropertyGroupDescriptorDto = {
  id: string;
  label: string;
  description: string;
};

export type EditorMetadataTraitEditorSectionDescriptorDto = {
  id: string;
  label: string;
  placement: string;
  description: string;
  priority: number;
  defaultExpanded?: boolean;
  default_expanded?: boolean;
};

export type EditorMetadataTraitDiagnosticDescriptorDto = {
  code: string;
  label: string;
  description: string;
};

export function componentTypeName(descriptor: EditorComponentDescriptorDto): string {
  return descriptor.typeName ?? descriptor.type_name ?? descriptor.kind;
}

export function assetRefs(
  descriptor: EditorComponentDescriptorDto,
): EditorAssetRefDescriptorDto[] {
  return descriptor.assetRefs ?? descriptor.asset_refs ?? [];
}

export function editorControls(
  descriptor: EditorComponentDescriptorDto,
): EditorControlRefDto[] {
  return descriptor.editorControls ?? descriptor.editor_controls ?? [];
}

export function patchOps(
  descriptor: EditorComponentDescriptorDto,
): EditorPatchOpRefDto[] {
  return descriptor.patchOps ?? descriptor.patch_ops ?? [];
}

export function metadataTraits(
  catalog: EditorMetadataCatalogDto,
): EditorMetadataTraitDescriptorDto[] {
  return catalog.metadataTraits ?? catalog.metadata_traits ?? [];
}

export function componentMetadataTraits(
  descriptor: EditorComponentDescriptorDto,
): string[] {
  return descriptor.metadataTraits ?? descriptor.metadata_traits ?? [];
}

export function findMetadataTraitDescriptor(
  catalog: EditorMetadataCatalogDto | null | undefined,
  kind: string,
): EditorMetadataTraitDescriptorDto | null {
  if (!catalog) return null;
  return metadataTraits(catalog).find((trait) => trait.kind === kind) ?? null;
}

export function traitPropertyGroups(
  descriptor: EditorMetadataTraitDescriptorDto,
): EditorMetadataTraitPropertyGroupDescriptorDto[] {
  return descriptor.propertyGroups ?? descriptor.property_groups ?? [];
}

export function traitEditorSections(
  descriptor: EditorMetadataTraitDescriptorDto,
): EditorMetadataTraitEditorSectionDescriptorDto[] {
  return descriptor.editorSections ?? descriptor.editor_sections ?? [];
}

export function traitPatchOps(
  descriptor: EditorMetadataTraitDescriptorDto,
): string[] {
  return descriptor.patchOps ?? descriptor.patch_ops ?? [];
}

export function traitEditorControls(
  descriptor: EditorMetadataTraitDescriptorDto,
): string[] {
  return descriptor.controls ?? [];
}

export function propertyTraitKind(property: EditorPropertyDescriptorDto): string | null {
  return property.traitKind ?? property.trait_kind ?? null;
}

export function propertyGroup(property: EditorPropertyDescriptorDto): string {
  return property.group ?? "default";
}

export function assetRefTraitKind(ref: EditorAssetRefDescriptorDto): string | null {
  return ref.traitKind ?? ref.trait_kind ?? null;
}

export function targetKinds(
  catalog: EditorMetadataCatalogDto,
): EditorTargetKindDescriptorDto[] {
  return catalog.targetKinds ?? catalog.target_kinds ?? [];
}

export function assetKinds(
  catalog: EditorMetadataCatalogDto,
): EditorAssetKindDescriptorDto[] {
  return catalog.assetKinds ?? catalog.asset_kinds ?? [];
}

export function documentKinds(
  catalog: EditorMetadataCatalogDto,
): EditorDocumentKindDescriptorDto[] {
  return catalog.documentKinds ?? catalog.document_kinds ?? [];
}

export function catalogPatchOps(
  catalog: EditorMetadataCatalogDto,
): EditorPatchOpDescriptorDto[] {
  return catalog.patchOps ?? catalog.patch_ops ?? [];
}

export function controlTargetScope(control: EditorControlRefDto | EditorControlDescriptorDto): string {
  return control.targetScope ?? control.target_scope ?? "unknown";
}

export function controlPatchOp(control: EditorControlRefDto | EditorControlDescriptorDto): string | null {
  if ("patchOp" in control || "patch_op" in control) {
    return (control as EditorControlRefDto).patchOp ?? (control as EditorControlRefDto).patch_op ?? null;
  }

  return (
    (control as EditorControlDescriptorDto).defaultPatchOp ??
    (control as EditorControlDescriptorDto).default_patch_op ??
    null
  );
}

export function patchOpTargetScope(op: EditorPatchOpRefDto | EditorPatchOpDescriptorDto): string {
  return op.targetScope ?? op.target_scope ?? "unknown";
}

export function boundsPolicy(
  descriptor: EditorComponentDescriptorDto,
): EditorBoundsPolicyDto {
  return descriptor.boundsPolicy ?? descriptor.bounds_policy ?? { kind: "None" };
}
