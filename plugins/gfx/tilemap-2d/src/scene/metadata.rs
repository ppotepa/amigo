use amigo_scene::*;

pub fn tile_map_2d_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor {
        kind_id: "TileMap2D",
        type_name: "TileMap2D",
        label: "Tile Map 2D",
        domains: &[ComponentDomain::Render2D],
        owner_scopes: ENTITY_OWNER_SCOPES,
        default_yaml: None,
        metadata_traits: &[
            MetadataTraitKind::Renderable2D,
            MetadataTraitKind::RenderLayered2D,
            MetadataTraitKind::UsesTransform2D,
            MetadataTraitKind::Selectable,
            MetadataTraitKind::HasBounds2D,
            MetadataTraitKind::HasAssetRefs,
            MetadataTraitKind::HasEditorControls,
            MetadataTraitKind::GenericEditable,
        ],
        asset_refs: &[
            ComponentAssetRefDescriptor {
                field_path: "tileset",
                domain: AssetDomain::TileSet,
                required: true,
                trait_kind: MetadataTraitKind::HasAssetRefs,
                group: "assetRefs.primary",
            },
            ComponentAssetRefDescriptor {
                field_path: "ruleset",
                domain: AssetDomain::TileRuleSet,
                required: false,
                trait_kind: MetadataTraitKind::HasAssetRefs,
                group: "assetRefs.optional",
            },
        ],
        properties: &[
            EditorPropertyDescriptor {
                path: "tileset",
                label: "Tileset",
                value_kind: EditorPropertyValueKind::AssetRef,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::AssetPicker,
                asset_domain: Some(AssetDomain::TileSet),
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
                path: "ruleset",
                label: "Ruleset",
                value_kind: EditorPropertyValueKind::AssetRef,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::AssetPicker,
                asset_domain: Some(AssetDomain::TileRuleSet),
                trait_kind: Some(MetadataTraitKind::HasAssetRefs),
                group: "assetRefs.optional",
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
                path: "tile_size",
                label: "Tile Size",
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
                path: "render_layer",
                label: "Render Layer",
                value_kind: EditorPropertyValueKind::String,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Text,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::RenderLayered2D),
                group: "render2d.order",
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
                path: "grid",
                label: "Grid",
                value_kind: EditorPropertyValueKind::String,
                access: EditorPropertyAccess::ReadOnly,
                editor: EditorPropertyEditorKind::ReadOnly,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::Renderable2D),
                group: "render2d.content",
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
        bounds_policy: BoundsPolicy::DerivedFromTileMap,
        editor_controls: &[
            EditorControlKind::Transform2D,
            EditorControlKind::TileMapBrush2D,
        ],
        patch_ops: &[
            EditorPatchOpKind::SetTransform2,
            EditorPatchOpKind::SetTileCell,
            EditorPatchOpKind::ResizeTileMap,
        ],
    }
}

pub struct TileMap2dComponentMetadataProvider;

impl ComponentMetadataProvider for TileMap2dComponentMetadataProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.gfx.tilemap-2d.component-metadata"
    }

    fn register_component_metadata(&self, registry: &mut ComponentRegistry) {
        registry.insert(tile_map_2d_descriptor());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_runtime::{RuntimePlugin, ServiceRegistry};

    #[test]
    fn provider_registers_tilemap_descriptor() {
        let mut registry = ComponentRegistry::new([]);

        TileMap2dComponentMetadataProvider.register_component_metadata(&mut registry);

        let descriptor = registry
            .descriptor("TileMap2D")
            .expect("tilemap metadata descriptor should be registered");
        assert_eq!(descriptor.type_name, "TileMap2D");
        assert!(descriptor.metadata_traits.contains(&MetadataTraitKind::Renderable2D));
        assert!(descriptor.metadata_traits.contains(&MetadataTraitKind::HasAssetRefs));
    }

    #[test]
    fn plugin_registers_tilemap_metadata_provider() {
        let mut registry = ServiceRegistry::default();
        registry
            .register(ComponentMetadataProviderRegistry::default())
            .unwrap();
        registry
            .register(RuntimeSceneCommandHandlerRegistry::new())
            .unwrap();

        crate::tilemap::TileMap2dPlugin.register(&mut registry).unwrap();

        let providers = registry
            .resolve::<ComponentMetadataProviderRegistry>()
            .expect("metadata provider registry should remain registered");
        assert_eq!(
            providers.provider_ids(),
            vec!["amigo.gfx.tilemap-2d.component-metadata"]
        );

        let component_registry = component_registry_with_providers(Some(providers.as_ref()));
        assert!(component_registry.descriptor("TileMap2D").is_some());
    }
}
