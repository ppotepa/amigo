use amigo_scene::*;

use super::document::NPR_SETTINGS_COMPONENT_TYPE;

fn descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor {
        kind_id: NPR_SETTINGS_COMPONENT_TYPE,
        type_name: "NprSettings",
        label: "NPR Settings",
        domains: &[ComponentDomain::Render3D],
        owner_scopes: ENTITY_OWNER_SCOPES,
        default_yaml: Some("type: amigo.gfx.npr-playground.NprSettings\ngallery: false\nselected: cube\nobjects: {}\n"),
        metadata_traits: &[MetadataTraitKind::GenericEditable, MetadataTraitKind::RuntimeControllable],
        asset_refs: &[],
        properties: &[],
        transform_policy: TransformPolicy::None,
        bounds_policy: BoundsPolicy::None,
        editor_controls: &[],
        patch_ops: &[],
    }
}

#[derive(Default)]
pub struct NprPlaygroundComponentMetadataProvider;

impl ComponentMetadataProvider for NprPlaygroundComponentMetadataProvider {
    fn provider_id(&self) -> &'static str { "amigo.gfx.npr-playground" }
    fn register_component_metadata(&self, registry: &mut ComponentRegistry) {
        registry.try_insert(descriptor()).expect("duplicate NPR settings component metadata");
    }
}
