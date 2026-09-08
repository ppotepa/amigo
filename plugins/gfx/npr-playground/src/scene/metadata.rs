use amigo_scene::*;

use super::document::NPR_SETTINGS_COMPONENT_TYPE;

const MODEL_OPTIONS: &[EditorPropertyOption] = &[
    EditorPropertyOption { id: "cube", label: "Cube" },
    EditorPropertyOption { id: "wedge", label: "Wedge" },
    EditorPropertyOption { id: "cylinder", label: "Cylinder" },
    EditorPropertyOption { id: "sphere", label: "Sphere" },
    EditorPropertyOption { id: "suzanne", label: "Suzanne" },
    EditorPropertyOption { id: "avocado", label: "Avocado" },
];

const CAMERA_DISTANCE: EditorNumberConstraints = EditorNumberConstraints {
    min: Some(0.1), max: Some(100.0), step: Some(0.1), clamp: true, unit: None, display_scale: 1.0,
};

const CAMERA_ANGLE: EditorNumberConstraints = EditorNumberConstraints {
    min: Some(-180.0), max: Some(180.0), step: Some(1.0), clamp: true, unit: Some("deg"), display_scale: 1.0,
};

const CAMERA_FOV: EditorNumberConstraints = EditorNumberConstraints {
    min: Some(15.0), max: Some(90.0), step: Some(1.0), clamp: true, unit: Some("deg"), display_scale: 1.0,
};

const REDRAW_HZ: EditorNumberConstraints = EditorNumberConstraints {
    min: Some(0.25), max: Some(20.0), step: Some(0.25), clamp: true, unit: Some("Hz"), display_scale: 1.0,
};

const UNIT_FRACTION: EditorNumberConstraints = EditorNumberConstraints {
    min: Some(0.0), max: Some(1.0), step: Some(0.01), clamp: true, unit: None, display_scale: 1.0,
};

const STROKE_MOTION_OPTIONS: &[EditorPropertyOption] = &[
    EditorPropertyOption { id: "stable", label: "Stable" },
    EditorPropertyOption { id: "redraw-on-motion", label: "Redraw on motion" },
];

const SEED: EditorNumberConstraints = EditorNumberConstraints {
    min: Some(0.0), max: Some(u32::MAX as f32), step: Some(1.0), clamp: true, unit: None, display_scale: 1.0,
};

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
        properties: &[
            EditorPropertyDescriptor {
                path: "gallery", label: "Gallery", value_kind: EditorPropertyValueKind::Bool,
                access: EditorPropertyAccess::Editable, editor: EditorPropertyEditorKind::Checkbox,
                asset_domain: None, trait_kind: Some(MetadataTraitKind::GenericEditable),
                group: "npr.scene", patch_op: None, number_constraints: None, options: &[],
                visibility: EditorPropertyVisibility::Primary, order: 0, tags: &["Live"],
                readonly_reason: None, binding_template: Some(EditorRuntimeBindingTemplate::ComponentRuntimeField),
            },
            EditorPropertyDescriptor {
                path: "selected", label: "Selected Model", value_kind: EditorPropertyValueKind::Enum,
                access: EditorPropertyAccess::Editable, editor: EditorPropertyEditorKind::EnumSelect,
                asset_domain: None, trait_kind: Some(MetadataTraitKind::GenericEditable),
                group: "npr.scene", patch_op: None, number_constraints: None, options: MODEL_OPTIONS,
                visibility: EditorPropertyVisibility::Primary, order: 1, tags: &["Live"],
                readonly_reason: None, binding_template: Some(EditorRuntimeBindingTemplate::ComponentRuntimeField),
            },
            EditorPropertyDescriptor {
                path: "seed", label: "Drawing Seed", value_kind: EditorPropertyValueKind::Number,
                access: EditorPropertyAccess::Editable, editor: EditorPropertyEditorKind::Number,
                asset_domain: None, trait_kind: Some(MetadataTraitKind::GenericEditable),
                group: "npr.drawing", patch_op: None, number_constraints: Some(SEED), options: &[],
                visibility: EditorPropertyVisibility::Primary, order: 2, tags: &["Live"],
                readonly_reason: None, binding_template: Some(EditorRuntimeBindingTemplate::ComponentRuntimeField),
            },
            EditorPropertyDescriptor {
                path: "camera.distance", label: "Camera Distance", value_kind: EditorPropertyValueKind::Number,
                access: EditorPropertyAccess::Editable, editor: EditorPropertyEditorKind::Number,
                asset_domain: None, trait_kind: Some(MetadataTraitKind::GenericEditable),
                group: "npr.camera", patch_op: None, number_constraints: Some(CAMERA_DISTANCE), options: &[],
                visibility: EditorPropertyVisibility::Primary, order: 0, tags: &["Live"],
                readonly_reason: None, binding_template: Some(EditorRuntimeBindingTemplate::ComponentRuntimeField),
            },
            EditorPropertyDescriptor {
                path: "camera.yaw", label: "Camera Yaw", value_kind: EditorPropertyValueKind::Number,
                access: EditorPropertyAccess::Editable, editor: EditorPropertyEditorKind::Number,
                asset_domain: None, trait_kind: Some(MetadataTraitKind::GenericEditable),
                group: "npr.camera", patch_op: None, number_constraints: Some(CAMERA_ANGLE), options: &[],
                visibility: EditorPropertyVisibility::Advanced, order: 1, tags: &["Live"],
                readonly_reason: None, binding_template: Some(EditorRuntimeBindingTemplate::ComponentRuntimeField),
            },
            EditorPropertyDescriptor {
                path: "camera.pitch", label: "Camera Pitch", value_kind: EditorPropertyValueKind::Number,
                access: EditorPropertyAccess::Editable, editor: EditorPropertyEditorKind::Number,
                asset_domain: None, trait_kind: Some(MetadataTraitKind::GenericEditable),
                group: "npr.camera", patch_op: None, number_constraints: Some(CAMERA_ANGLE), options: &[],
                visibility: EditorPropertyVisibility::Advanced, order: 2, tags: &["Live"],
                readonly_reason: None, binding_template: Some(EditorRuntimeBindingTemplate::ComponentRuntimeField),
            },
            EditorPropertyDescriptor {
                path: "camera.fov", label: "Camera Field of View", value_kind: EditorPropertyValueKind::Number,
                access: EditorPropertyAccess::Editable, editor: EditorPropertyEditorKind::Number,
                asset_domain: None, trait_kind: Some(MetadataTraitKind::GenericEditable),
                group: "npr.camera", patch_op: None, number_constraints: Some(CAMERA_FOV), options: &[],
                visibility: EditorPropertyVisibility::Advanced, order: 3, tags: &["Live"],
                readonly_reason: None, binding_template: Some(EditorRuntimeBindingTemplate::ComponentRuntimeField),
            },
            EditorPropertyDescriptor {
                path: "motion.mode", label: "Stroke Motion", value_kind: EditorPropertyValueKind::Enum,
                access: EditorPropertyAccess::Editable, editor: EditorPropertyEditorKind::EnumSelect,
                asset_domain: None, trait_kind: Some(MetadataTraitKind::GenericEditable),
                group: "npr.motion", patch_op: None, number_constraints: None, options: STROKE_MOTION_OPTIONS,
                visibility: EditorPropertyVisibility::Primary, order: 0, tags: &["Live"],
                readonly_reason: None, binding_template: Some(EditorRuntimeBindingTemplate::ComponentRuntimeField),
            },
            EditorPropertyDescriptor {
                path: "motion.redraw_hz", label: "Stroke Redraw Rate", value_kind: EditorPropertyValueKind::Number,
                access: EditorPropertyAccess::Editable, editor: EditorPropertyEditorKind::Number,
                asset_domain: None, trait_kind: Some(MetadataTraitKind::GenericEditable),
                group: "npr.motion", patch_op: None, number_constraints: Some(REDRAW_HZ), options: &[],
                visibility: EditorPropertyVisibility::Advanced, order: 1, tags: &["Live"],
                readonly_reason: None, binding_template: Some(EditorRuntimeBindingTemplate::ComponentRuntimeField),
            },
            EditorPropertyDescriptor {
                path: "motion.redraw_strength", label: "Stroke Redraw Strength", value_kind: EditorPropertyValueKind::Number,
                access: EditorPropertyAccess::Editable, editor: EditorPropertyEditorKind::Number,
                asset_domain: None, trait_kind: Some(MetadataTraitKind::GenericEditable),
                group: "npr.motion", patch_op: None, number_constraints: Some(UNIT_FRACTION), options: &[],
                visibility: EditorPropertyVisibility::Advanced, order: 2, tags: &["Live"],
                readonly_reason: None, binding_template: Some(EditorRuntimeBindingTemplate::ComponentRuntimeField),
            },
            EditorPropertyDescriptor {
                path: "objects", label: "Object Overrides", value_kind: EditorPropertyValueKind::String,
                access: EditorPropertyAccess::ReadOnly, editor: EditorPropertyEditorKind::ReadOnly,
                asset_domain: None, trait_kind: Some(MetadataTraitKind::GenericEditable),
                group: "npr.drawing", patch_op: None, number_constraints: None, options: &[],
                visibility: EditorPropertyVisibility::Advanced, order: 3, tags: &["Structured"],
                readonly_reason: Some("Surface-anchor authoring is available at runtime; persistent scene edits require the editor authoring transaction service."), binding_template: None,
            },
        ],
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
