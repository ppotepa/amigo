export type EditorMetadataCatalogDto = {
  components: EditorComponentDescriptorDto[];
};

export type EditorComponentDescriptorDto = {
  kind: string;
  typeName?: string;
  type_name?: string;
  label: string;
  domains: string[];
  capabilities: string[];
  assetRefs?: EditorAssetRefDescriptorDto[];
  asset_refs?: EditorAssetRefDescriptorDto[];
  properties: EditorPropertyDescriptorDto[];
  editorControls?: string[];
  editor_controls?: string[];
  patchOps?: string[];
  patch_ops?: string[];
};

export type EditorAssetRefDescriptorDto = {
  fieldPath?: string;
  field_path?: string;
  domain: string;
  required: boolean;
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
  patchOp?: string | null;
  patch_op?: string | null;
};

export function componentTypeName(descriptor: EditorComponentDescriptorDto): string {
  return descriptor.typeName ?? descriptor.type_name ?? descriptor.kind;
}

export function assetRefs(
  descriptor: EditorComponentDescriptorDto,
): EditorAssetRefDescriptorDto[] {
  return descriptor.assetRefs ?? descriptor.asset_refs ?? [];
}
