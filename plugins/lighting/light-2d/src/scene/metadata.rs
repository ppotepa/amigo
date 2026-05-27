use amigo_scene::*;

macro_rules! p {
    ($path:literal, $label:literal, $kind:expr, $editor:expr, $trait_kind:expr, $group:literal) => {
        EditorPropertyDescriptor {
            path: $path,
            label: $label,
            value_kind: $kind,
            access: EditorPropertyAccess::Editable,
            editor: $editor,
            asset_domain: None,
            trait_kind: Some($trait_kind),
            group: $group,
            patch_op: None,
            number_constraints: None,
            options: &[],
            visibility: EditorPropertyVisibility::Advanced,
            order: 0,
            tags: &["Unsupported"],
            readonly_reason: Some("No live runtime binding yet"),
            binding_template: None,
        }
    };
}

pub fn global_light_2d_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor {
        kind_id: "GlobalLight2D",
        type_name: "GlobalLight2D",
        label: "Global Light 2D",
        domains: &[ComponentDomain::Render2D],
        owner_scopes: ENTITY_OWNER_SCOPES,
        default_yaml: None,
        metadata_traits: &[
            MetadataTraitKind::Renderable2D,
            MetadataTraitKind::RuntimeControllable,
            MetadataTraitKind::GenericEditable,
        ],
        asset_refs: &[],
        properties: &[
            p!(
                "id",
                "Id",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Renderable2D,
                "render2d.light"
            ),
            p!(
                "color",
                "Color",
                EditorPropertyValueKind::Color,
                EditorPropertyEditorKind::Color,
                MetadataTraitKind::Renderable2D,
                "render2d.light"
            ),
            p!(
                "intensity",
                "Intensity",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::RuntimeControllable,
                "render2d.light"
            ),
        ],
        transform_policy: TransformPolicy::None,
        bounds_policy: BoundsPolicy::None,
        editor_controls: &[EditorControlKind::InspectorOnly],
        patch_ops: &[],
    }
}

#[derive(Default)]
pub struct GlobalLight2dComponentMetadataProvider;

impl ComponentMetadataProvider for GlobalLight2dComponentMetadataProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.lighting.light-2d"
    }

    fn register_component_metadata(&self, registry: &mut ComponentRegistry) {
        registry
            .try_insert(global_light_2d_descriptor())
            .expect("duplicate GlobalLight2D component metadata");
    }
}
