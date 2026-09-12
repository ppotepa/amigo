use super::document::NPR_SETTINGS_COMPONENT_TYPE;
use amigo_scene::*;

fn descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor {
        kind_id: NPR_SETTINGS_COMPONENT_TYPE,
        type_name: "NprSettings",
        label: "NPR Settings",
        domains: &[ComponentDomain::Render3D],
        owner_scopes: ENTITY_OWNER_SCOPES,
        default_yaml: Some("type: amigo.gfx.npr-playground.NprSettings\nprofile: npr.scene.yml\n"),
        metadata_traits: &[MetadataTraitKind::GenericEditable],
        asset_refs: &[],
        properties: &[EditorPropertyDescriptor {
            path: "profile",
            label: "NPR scene profile",
            value_kind: EditorPropertyValueKind::String,
            access: EditorPropertyAccess::ReadOnly,
            editor: EditorPropertyEditorKind::ReadOnly,
            asset_domain: None,
            trait_kind: Some(MetadataTraitKind::GenericEditable),
            group: "npr.scene",
            patch_op: None,
            number_constraints: None,
            options: &[],
            visibility: EditorPropertyVisibility::Primary,
            order: 0,
            tags: &["Authored"],
            readonly_reason: Some("Edit the profile through NprPlayground."),
            binding_template: None,
        }],
        transform_policy: TransformPolicy::None,
        bounds_policy: BoundsPolicy::None,
        editor_controls: &[],
        patch_ops: &[],
    }
}
#[derive(Default)]
pub struct NprPlaygroundComponentMetadataProvider;

impl ComponentMetadataProvider for NprPlaygroundComponentMetadataProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.gfx.npr-playground"
    }
    fn register_component_metadata(&self, registry: &mut ComponentRegistry) {
        registry
            .try_insert(descriptor())
            .expect("duplicate NPR settings component metadata");
    }
}
