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

pub fn beacon_light_2d_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor {
        kind_id: "BeaconLight2D",
        type_name: "BeaconLight2D",
        label: "Beacon Light 2D",
        domains: &[ComponentDomain::Render2D],
        owner_scopes: ENTITY_OWNER_SCOPES,
        default_yaml: None,
        metadata_traits: &[
            MetadataTraitKind::Renderable2D,
            MetadataTraitKind::RenderLayered2D,
            MetadataTraitKind::UsesTransform2D,
            MetadataTraitKind::Selectable,
            MetadataTraitKind::HasBounds2D,
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
                "render2d.beacon"
            ),
            p!(
                "render_layer",
                "Render Layer",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::RenderLayered2D,
                "render2d.order"
            ),
            p!(
                "color",
                "Color",
                EditorPropertyValueKind::Color,
                EditorPropertyEditorKind::Color,
                MetadataTraitKind::Renderable2D,
                "render2d.beacon"
            ),
            p!(
                "base_intensity",
                "Base Intensity",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::RuntimeControllable,
                "render2d.beacon"
            ),
            p!(
                "frequency_hz",
                "Frequency Hz",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::RuntimeControllable,
                "render2d.beacon"
            ),
            p!(
                "rise_seconds",
                "Rise Seconds",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::RuntimeControllable,
                "render2d.beacon"
            ),
            p!(
                "fall_seconds",
                "Fall Seconds",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::RuntimeControllable,
                "render2d.beacon"
            ),
            p!(
                "duty_cycle",
                "Duty Cycle",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::RuntimeControllable,
                "render2d.beacon"
            ),
            p!(
                "core_radius_px",
                "Core Radius",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::RuntimeControllable,
                "render2d.beacon"
            ),
            p!(
                "halo_radius_px",
                "Halo Radius",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::RuntimeControllable,
                "render2d.beacon"
            ),
            p!(
                "glow_strength",
                "Glow Strength",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::RuntimeControllable,
                "render2d.beacon"
            ),
            p!(
                "beam_enabled",
                "Beam Enabled",
                EditorPropertyValueKind::Bool,
                EditorPropertyEditorKind::Checkbox,
                MetadataTraitKind::RuntimeControllable,
                "render2d.beacon"
            ),
            p!(
                "beam_length_px",
                "Beam Length",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::RuntimeControllable,
                "render2d.beacon"
            ),
            p!(
                "beam_width_degrees",
                "Beam Width Degrees",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::RuntimeControllable,
                "render2d.beacon"
            ),
            p!(
                "beam_strength",
                "Beam Strength",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::RuntimeControllable,
                "render2d.beacon"
            ),
            p!(
                "aberration_px",
                "Aberration",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::RuntimeControllable,
                "render2d.beacon"
            ),
            p!(
                "bloom",
                "Bloom",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::RuntimeControllable,
                "render2d.beacon"
            ),
            p!(
                "camera_response",
                "Camera Response",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::ReadOnly,
                MetadataTraitKind::Renderable2D,
                "render2d.beacon"
            ),
        ],
        transform_policy: TransformPolicy::UsesEntityTransform2,
        bounds_policy: BoundsPolicy::EntityTransformPoint,
        editor_controls: &[
            EditorControlKind::Transform2D,
            EditorControlKind::InspectorOnly,
        ],
        patch_ops: &[EditorPatchOpKind::SetTransform2],
    }
}

#[derive(Default)]
pub struct BeaconLight2dComponentMetadataProvider;

impl ComponentMetadataProvider for BeaconLight2dComponentMetadataProvider {
    fn provider_id(&self) -> &'static str {
        "amigo.lighting.beacon-light-2d"
    }

    fn register_component_metadata(&self, registry: &mut ComponentRegistry) {
        registry
            .try_insert(beacon_light_2d_descriptor())
            .expect("duplicate BeaconLight2D component metadata");
    }
}
