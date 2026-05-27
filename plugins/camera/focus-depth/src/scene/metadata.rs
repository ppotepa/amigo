use amigo_scene::*;

pub fn depth_map_2d_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor {
        kind_id: "DepthMap2D",
        type_name: "DepthMap2D",
        label: "Depth Map 2D",
        domains: &[ComponentDomain::Render2D, ComponentDomain::Camera],
        owner_scopes: ENTITY_OWNER_SCOPES,
        default_yaml: Some(
            "type: DepthMap2D\nid: main-depth\nasset: \"\"\nsize: { x: 1280.0, y: 720.0 }\nviewport_fit: cover\nwhite_is_near: true\nz_index: -100.0\n",
        ),
        metadata_traits: &[
            MetadataTraitKind::Renderable2D,
            MetadataTraitKind::UsesTransform2D,
            MetadataTraitKind::Selectable,
            MetadataTraitKind::HasBounds2D,
            MetadataTraitKind::HasAssetRefs,
            MetadataTraitKind::GenericEditable,
        ],
        asset_refs: &[ComponentAssetRefDescriptor {
            field_path: "asset",
            domain: AssetDomain::Raw,
            required: true,
            trait_kind: MetadataTraitKind::HasAssetRefs,
            group: "assetRefs.primary",
        }],
        properties: &[
            EditorPropertyDescriptor {
                path: "asset",
                label: "Depth Map Asset",
                value_kind: EditorPropertyValueKind::AssetRef,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::AssetPicker,
                asset_domain: Some(AssetDomain::Raw),
                trait_kind: Some(MetadataTraitKind::HasAssetRefs),
                group: "assetRefs.primary",
                patch_op: None,
                number_constraints: None,
                options: &[],
                visibility: EditorPropertyVisibility::Primary,
                order: 0,
                tags: &[],
                readonly_reason: None,
                binding_template: None,
            },
            EditorPropertyDescriptor {
                path: "size",
                label: "Size",
                value_kind: EditorPropertyValueKind::Vec2,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Vec2,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::HasBounds2D),
                group: "bounds2.size",
                patch_op: None,
                number_constraints: None,
                options: &[],
                visibility: EditorPropertyVisibility::Primary,
                order: 0,
                tags: &[],
                readonly_reason: None,
                binding_template: None,
            },
            EditorPropertyDescriptor {
                path: "viewport_fit",
                label: "Viewport Fit",
                value_kind: EditorPropertyValueKind::String,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Text,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::Renderable2D),
                group: "render2d.viewport",
                patch_op: None,
                number_constraints: None,
                options: &[],
                visibility: EditorPropertyVisibility::Primary,
                order: 0,
                tags: &[],
                readonly_reason: None,
                binding_template: None,
            },
        ],
        transform_policy: TransformPolicy::UsesEntityTransform2,
        bounds_policy: BoundsPolicy::ComponentBounds2D { field: "size" },
        editor_controls: &[EditorControlKind::Transform2D, EditorControlKind::Rect2D],
        patch_ops: &[EditorPatchOpKind::SetTransform2],
    }
}

pub fn depth_aux_map_2d_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor {
        kind_id: "DepthAuxMap2D",
        type_name: "DepthAuxMap2D",
        label: "Depth Aux Map 2D",
        domains: &[ComponentDomain::Render2D, ComponentDomain::Camera],
        owner_scopes: ENTITY_OWNER_SCOPES,
        default_yaml: Some(
            "type: DepthAuxMap2D\nid: main-depth-aux\nasset: \"\"\nsize: { x: 1280.0, y: 720.0 }\nviewport_fit: cover\nchannels: { r: auxiliary_depth, g: local_height, b: occluder_strength, a: valid_mask }\nz_index: -99.5\n",
        ),
        metadata_traits: &[
            MetadataTraitKind::Renderable2D,
            MetadataTraitKind::UsesTransform2D,
            MetadataTraitKind::Selectable,
            MetadataTraitKind::HasBounds2D,
            MetadataTraitKind::HasAssetRefs,
            MetadataTraitKind::GenericEditable,
        ],
        asset_refs: &[ComponentAssetRefDescriptor {
            field_path: "asset",
            domain: AssetDomain::Raw,
            required: true,
            trait_kind: MetadataTraitKind::HasAssetRefs,
            group: "assetRefs.primary",
        }],
        properties: &[
            EditorPropertyDescriptor {
                path: "asset",
                label: "Depth Aux RGBA Asset",
                value_kind: EditorPropertyValueKind::AssetRef,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::AssetPicker,
                asset_domain: Some(AssetDomain::Raw),
                trait_kind: Some(MetadataTraitKind::HasAssetRefs),
                group: "assetRefs.primary",
                patch_op: None,
                number_constraints: None,
                options: &[],
                visibility: EditorPropertyVisibility::Primary,
                order: 0,
                tags: &[],
                readonly_reason: None,
                binding_template: None,
            },
            EditorPropertyDescriptor {
                path: "size",
                label: "Size",
                value_kind: EditorPropertyValueKind::Vec2,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Vec2,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::HasBounds2D),
                group: "bounds2.size",
                patch_op: None,
                number_constraints: None,
                options: &[],
                visibility: EditorPropertyVisibility::Primary,
                order: 0,
                tags: &[],
                readonly_reason: None,
                binding_template: None,
            },
            EditorPropertyDescriptor {
                path: "channels",
                label: "RGBA Channels",
                value_kind: EditorPropertyValueKind::String,
                access: EditorPropertyAccess::ReadOnly,
                editor: EditorPropertyEditorKind::Text,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::GenericEditable),
                group: "depthAux.channels",
                patch_op: None,
                number_constraints: None,
                options: &[],
                visibility: EditorPropertyVisibility::Advanced,
                order: 0,
                tags: &[],
                readonly_reason: Some(
                    "DepthAux channels are semantic labels for renderer/light pipeline consumers.",
                ),
                binding_template: None,
            },
        ],
        transform_policy: TransformPolicy::UsesEntityTransform2,
        bounds_policy: BoundsPolicy::ComponentBounds2D { field: "size" },
        editor_controls: &[EditorControlKind::Transform2D, EditorControlKind::Rect2D],
        patch_ops: &[EditorPatchOpKind::SetTransform2],
    }
}

#[derive(Default)]
pub struct DepthMap2dComponentMetadataProvider;

impl ComponentMetadataProvider for DepthMap2dComponentMetadataProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.camera.focus-depth.depth-map"
    }

    fn register_component_metadata(&self, registry: &mut ComponentRegistry) {
        registry
            .try_insert(depth_map_2d_descriptor())
            .expect("duplicate DepthMap2D component metadata");
    }
}

#[derive(Default)]
pub struct DepthAuxMap2dComponentMetadataProvider;

impl ComponentMetadataProvider for DepthAuxMap2dComponentMetadataProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.camera.focus-depth.depth-aux-map"
    }

    fn register_component_metadata(&self, registry: &mut ComponentRegistry) {
        registry
            .try_insert(depth_aux_map_2d_descriptor())
            .expect("duplicate DepthAuxMap2D component metadata");
    }
}
