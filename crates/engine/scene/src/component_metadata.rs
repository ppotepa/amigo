use crate::MetadataTraitKind;

mod model;

pub use model::*;

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
    (live $path:literal, $label:literal, $kind:expr, $editor:expr, $trait_kind:expr, $group:literal, $binding:expr) => {
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
            tags: &["Live"],
            readonly_reason: None,
            binding_template: Some($binding),
        }
    };
    (num $path:literal, $label:literal, $kind:expr, $editor:expr, $trait_kind:expr, $group:literal, $constraints:expr) => {
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
            number_constraints: Some($constraints),
            options: &[],
            visibility: EditorPropertyVisibility::Primary,
            order: 0,
            tags: &["Unsupported"],
            readonly_reason: Some("No live runtime binding yet"),
            binding_template: None,
        }
    };
    (live num $path:literal, $label:literal, $kind:expr, $editor:expr, $trait_kind:expr, $group:literal, $constraints:expr, $binding:expr) => {
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
            number_constraints: Some($constraints),
            options: &[],
            visibility: EditorPropertyVisibility::Primary,
            order: 0,
            tags: &["Live"],
            readonly_reason: None,
            binding_template: Some($binding),
        }
    };
    (ro $path:literal, $label:literal, $trait_kind:expr, $group:literal) => {
        EditorPropertyDescriptor {
            path: $path,
            label: $label,
            value_kind: EditorPropertyValueKind::String,
            access: EditorPropertyAccess::ReadOnly,
            editor: EditorPropertyEditorKind::ReadOnly,
            asset_domain: None,
            trait_kind: Some($trait_kind),
            group: $group,
            patch_op: None,
            number_constraints: None,
            options: &[],
            visibility: EditorPropertyVisibility::Advanced,
            order: 0,
            tags: &["Readonly"],
            readonly_reason: Some("Descriptor metadata"),
            binding_template: None,
        }
    };
}

pub fn lightmap_2d_source_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "LightMap2DSource",
        "LightMap2DSource",
        "Light Map 2D Source",
        &[ComponentDomain::Render2D],
        &[
            MetadataTraitKind::Renderable2D,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "id",
                "Id",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Renderable2D,
                "render2d.lightmap"
            ),
            p!(ro "source", "Source", MetadataTraitKind::Renderable2D, "render2d.lightmap"),
            p!(ro "channels", "Channels", MetadataTraitKind::Renderable2D, "render2d.lightmap"),
        ],
        &[],
        TransformPolicy::None,
        BoundsPolicy::None,
        &[EditorControlKind::InspectorOnly],
        &[],
    )
}

pub fn trigger_2d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "Trigger2D",
        "Trigger2D",
        "Trigger 2D",
        &[ComponentDomain::Physics2D],
        &[
            MetadataTraitKind::Collidable2D,
            MetadataTraitKind::Trigger2D,
            MetadataTraitKind::EventSource,
            MetadataTraitKind::UsesTransform2D,
            MetadataTraitKind::Selectable,
            MetadataTraitKind::HasBounds2D,
            MetadataTraitKind::HasEditorControls,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "size",
                "Size",
                EditorPropertyValueKind::Vec2,
                EditorPropertyEditorKind::Vec2,
                MetadataTraitKind::HasBounds2D,
                "bounds2.size"
            ),
            p!(
                "offset",
                "Offset",
                EditorPropertyValueKind::Vec2,
                EditorPropertyEditorKind::Vec2,
                MetadataTraitKind::HasBounds2D,
                "bounds2.offset"
            ),
            p!(
                "layer",
                "Layer",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Collidable2D,
                "collision.layer"
            ),
            p!(ro "mask", "Mask", MetadataTraitKind::Collidable2D, "collision.mask"),
            p!(
                "event",
                "Event",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::EventSource,
                "events.source"
            ),
        ],
        &[],
        TransformPolicy::UsesEntityTransform2,
        BoundsPolicy::DerivedFromCollider2D,
        &[EditorControlKind::Transform2D, EditorControlKind::Trigger2D],
        &[
            EditorPatchOpKind::SetTransform2,
            EditorPatchOpKind::SetColliderShape,
        ],
    )
}

pub fn script_component_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor {
        kind_id: "ScriptComponent",
        type_name: "ScriptComponent",
        label: "Script Component",
        domains: &[ComponentDomain::Scripting],
        owner_scopes: ENTITY_OWNER_SCOPES,
        default_yaml: None,
        metadata_traits: &[
            MetadataTraitKind::Scriptable,
            MetadataTraitKind::HasAssetRefs,
            MetadataTraitKind::GenericEditable,
        ],
        asset_refs: &[ComponentAssetRefDescriptor {
            field_path: "script",
            domain: AssetDomain::Script,
            required: true,
            trait_kind: MetadataTraitKind::HasAssetRefs,
            group: "assetRefs.primary",
        }],
        properties: &[
            EditorPropertyDescriptor {
                path: "script",
                label: "Script",
                value_kind: EditorPropertyValueKind::AssetRef,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::AssetPicker,
                asset_domain: Some(AssetDomain::Script),
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
                path: "params",
                label: "Params",
                value_kind: EditorPropertyValueKind::String,
                access: EditorPropertyAccess::ReadOnly,
                editor: EditorPropertyEditorKind::ReadOnly,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::Scriptable),
                group: "script.conditions",
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
        transform_policy: TransformPolicy::None,
        bounds_policy: BoundsPolicy::None,
        editor_controls: &[EditorControlKind::InspectorOnly],
        patch_ops: &[],
    }
}

fn generic_component_descriptor(
    kind_id: &'static str,
    type_name: &'static str,
    label: &'static str,
    domains: &'static [ComponentDomain],
    metadata_traits: &'static [MetadataTraitKind],
    properties: &'static [EditorPropertyDescriptor],
    asset_refs: &'static [ComponentAssetRefDescriptor],
    transform_policy: TransformPolicy,
    bounds_policy: BoundsPolicy,
    editor_controls: &'static [EditorControlKind],
    patch_ops: &'static [EditorPatchOpKind],
) -> ComponentTypeDescriptor {
    ComponentTypeDescriptor {
        kind_id,
        type_name,
        label,
        domains,
        owner_scopes: ENTITY_OWNER_SCOPES,
        default_yaml: None,
        metadata_traits,
        asset_refs,
        properties,
        transform_policy,
        bounds_policy,
        editor_controls,
        patch_ops,
    }
}

pub fn aabb_collider_2d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "AabbCollider2D",
        "AabbCollider2D",
        "AABB Collider 2D",
        &[ComponentDomain::Physics2D],
        &[
            MetadataTraitKind::Collidable2D,
            MetadataTraitKind::UsesTransform2D,
            MetadataTraitKind::Selectable,
            MetadataTraitKind::HasBounds2D,
            MetadataTraitKind::HasEditorControls,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "size",
                "Size",
                EditorPropertyValueKind::Vec2,
                EditorPropertyEditorKind::Vec2,
                MetadataTraitKind::HasBounds2D,
                "bounds2.size"
            ),
            p!(
                "offset",
                "Offset",
                EditorPropertyValueKind::Vec2,
                EditorPropertyEditorKind::Vec2,
                MetadataTraitKind::HasBounds2D,
                "bounds2.offset"
            ),
            p!(
                "layer",
                "Layer",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Collidable2D,
                "collision.layer"
            ),
            p!(ro "mask", "Mask", MetadataTraitKind::Collidable2D, "collision.mask"),
        ],
        &[],
        TransformPolicy::UsesEntityTransform2,
        BoundsPolicy::DerivedFromCollider2D,
        &[
            EditorControlKind::Transform2D,
            EditorControlKind::Collider2D,
        ],
        &[
            EditorPatchOpKind::SetTransform2,
            EditorPatchOpKind::SetColliderShape,
        ],
    )
}

pub fn circle_collider_2d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "CircleCollider2D",
        "CircleCollider2D",
        "Circle Collider 2D",
        &[ComponentDomain::Physics2D],
        &[
            MetadataTraitKind::Collidable2D,
            MetadataTraitKind::UsesTransform2D,
            MetadataTraitKind::Selectable,
            MetadataTraitKind::HasBounds2D,
            MetadataTraitKind::HasEditorControls,
            MetadataTraitKind::GenericEditable,
        ],
        &[p!(
            "radius",
            "Radius",
            EditorPropertyValueKind::Number,
            EditorPropertyEditorKind::Number,
            MetadataTraitKind::HasBounds2D,
            "bounds2.radius"
        )],
        &[],
        TransformPolicy::UsesEntityTransform2,
        BoundsPolicy::DerivedFromCollider2D,
        &[
            EditorControlKind::Transform2D,
            EditorControlKind::Collider2D,
        ],
        &[
            EditorPatchOpKind::SetTransform2,
            EditorPatchOpKind::SetColliderShape,
        ],
    )
}

pub fn input_action_map_descriptor() -> ComponentTypeDescriptor {
    let mut descriptor = generic_component_descriptor(
        "InputActionMap",
        "InputActionMap",
        "Input Action Map",
        &[ComponentDomain::Data],
        &[
            MetadataTraitKind::InputBindable,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "id",
                "Map ID",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::InputBindable,
                "input.identity"
            ),
            p!(
                "active",
                "Active",
                EditorPropertyValueKind::Bool,
                EditorPropertyEditorKind::Checkbox,
                MetadataTraitKind::InputBindable,
                "input.state"
            ),
            p!(ro "actions", "Actions", MetadataTraitKind::InputBindable, "input.selection"),
        ],
        &[],
        TransformPolicy::None,
        BoundsPolicy::None,
        &[EditorControlKind::InspectorOnly],
        &[],
    );
    descriptor.owner_scopes = SCENE_OWNER_SCOPES;
    descriptor.default_yaml = Some(
        "type: InputActionMap
id: input_map
active: true",
    );
    descriptor
}

fn generic_data_descriptor(
    kind_id: &'static str,
    type_name: &'static str,
    label: &'static str,
    traits: &'static [MetadataTraitKind],
    properties: &'static [EditorPropertyDescriptor],
) -> ComponentTypeDescriptor {
    generic_component_descriptor(
        kind_id,
        type_name,
        label,
        &[ComponentDomain::Data],
        traits,
        properties,
        &[],
        TransformPolicy::None,
        BoundsPolicy::None,
        &[EditorControlKind::InspectorOnly],
        &[],
    )
}

pub fn behavior_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "Behavior",
        "Behavior",
        "Behavior",
        &[ComponentDomain::Scripting, ComponentDomain::Data],
        &[
            MetadataTraitKind::UsesTransform2D,
            MetadataTraitKind::Selectable,
            MetadataTraitKind::Scriptable,
            MetadataTraitKind::EventSource,
            MetadataTraitKind::EventListener,
            MetadataTraitKind::InputBindable,
            MetadataTraitKind::HasAssetRefs,
            MetadataTraitKind::HasEditorControls,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "kind",
                "Kind",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Scriptable,
                "script.conditions"
            ),
            p!(
                "action",
                "Action",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Scriptable,
                "script.conditions"
            ),
            p!(
                "scene",
                "Scene",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Scriptable,
                "script.conditions"
            ),
            p!(
                "target",
                "Target",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Scriptable,
                "script.conditions"
            ),
            p!(
                "input",
                "Input",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::InputBindable,
                "input.selection"
            ),
            p!(
                "source",
                "Source",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Scriptable,
                "script.conditions"
            ),
            p!(
                "emitter",
                "Emitter",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::EventSource,
                "events.source"
            ),
            p!(ro "enabled_when", "Enabled When", MetadataTraitKind::Scriptable, "script.conditions"),
            p!(ro "phases", "Phases", MetadataTraitKind::Scriptable, "script.conditions"),
            p!(
                "up_action",
                "Up Action",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::InputBindable,
                "input.selection"
            ),
            p!(
                "down_action",
                "Down Action",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::InputBindable,
                "input.selection"
            ),
            p!(
                "confirm_action",
                "Confirm Action",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::InputBindable,
                "input.selection"
            ),
            p!(ro "confirm_events", "Confirm Events", MetadataTraitKind::EventSource, "events.confirmation"),
            p!(
                "index_state",
                "Index State",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::InputBindable,
                "input.selection"
            ),
            p!(
                "item_count",
                "Item Count",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::InputBindable,
                "input.selection"
            ),
            p!(
                "cooldown",
                "Cooldown",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Scriptable,
                "script.timing"
            ),
            p!(
                "cooldown_id",
                "Cooldown ID",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Scriptable,
                "script.timing"
            ),
            p!(
                "max_hold_seconds",
                "Max Hold Seconds",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Scriptable,
                "script.timing"
            ),
            p!(
                "selected_color_prefix",
                "Selected Color Prefix",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::UiEditable,
                "ui.selection"
            ),
            p!(
                "selected_color",
                "Selected Color",
                EditorPropertyValueKind::Color,
                EditorPropertyEditorKind::Color,
                MetadataTraitKind::UiEditable,
                "ui.selection"
            ),
            p!(
                "unselected_color",
                "Unselected Color",
                EditorPropertyValueKind::Color,
                EditorPropertyEditorKind::Color,
                MetadataTraitKind::UiEditable,
                "ui.selection"
            ),
            p!(
                "confirm_audio",
                "Confirm Audio",
                EditorPropertyValueKind::AssetRef,
                EditorPropertyEditorKind::AssetPicker,
                MetadataTraitKind::HasAssetRefs,
                "assetRefs.optional"
            ),
            p!(
                "move_audio",
                "Move Audio",
                EditorPropertyValueKind::AssetRef,
                EditorPropertyEditorKind::AssetPicker,
                MetadataTraitKind::HasAssetRefs,
                "assetRefs.optional"
            ),
            p!(
                "audio",
                "Audio",
                EditorPropertyValueKind::AssetRef,
                EditorPropertyEditorKind::AssetPicker,
                MetadataTraitKind::HasAssetRefs,
                "assetRefs.optional"
            ),
        ],
        &[],
        TransformPolicy::UsesEntityTransform2,
        BoundsPolicy::EntityTransformPoint,
        &[EditorControlKind::InspectorOnly],
        &[],
    )
}
pub fn event_pipeline_descriptor() -> ComponentTypeDescriptor {
    let mut descriptor = generic_data_descriptor(
        "EventPipeline",
        "EventPipeline",
        "Event Pipeline",
        &[
            MetadataTraitKind::EventSource,
            MetadataTraitKind::EventListener,
            MetadataTraitKind::SceneTransitionSource,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "id",
                "ID",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::EventSource,
                "script.conditions"
            ),
            p!(
                "topic",
                "Topic",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::EventListener,
                "script.conditions"
            ),
            p!(ro "steps", "Steps", MetadataTraitKind::SceneTransitionSource, "script.conditions"),
        ],
    );
    descriptor.owner_scopes = SCENE_OWNER_SCOPES;
    descriptor.default_yaml = Some(
        "type: EventPipeline
id: pipeline
topic: scene.transition",
    );
    descriptor
}
pub fn ui_document_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "UiDocument",
        "UiDocument",
        "UI Document",
        &[ComponentDomain::UI],
        &[
            MetadataTraitKind::UiEditable,
            MetadataTraitKind::HasUiTree,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "target",
                "Target",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::UiEditable,
                "ui.identity"
            ),
            p!(ro "root", "Root", MetadataTraitKind::HasUiTree, "ui.tree"),
        ],
        &[],
        TransformPolicy::None,
        BoundsPolicy::None,
        &[EditorControlKind::InspectorOnly],
        &[],
    )
}
pub fn ui_model_bindings_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "UiModelBindings",
        "UiModelBindings",
        "UI Model Bindings",
        &[ComponentDomain::UI],
        &[
            MetadataTraitKind::UiEditable,
            MetadataTraitKind::DataBindable,
            MetadataTraitKind::GenericEditable,
        ],
        &[p!(ro "bindings", "Bindings", MetadataTraitKind::DataBindable, "ui.bindings")],
        &[],
        TransformPolicy::None,
        BoundsPolicy::None,
        &[EditorControlKind::InspectorOnly],
        &[],
    )
}
pub fn ui_theme_set_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "UiThemeSet",
        "UiThemeSet",
        "UI Theme Set",
        &[ComponentDomain::UI],
        &[
            MetadataTraitKind::UiEditable,
            MetadataTraitKind::HasAssetRefs,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "active",
                "Active Theme",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::UiEditable,
                "ui.theme"
            ),
            p!(ro "themes", "Themes", MetadataTraitKind::HasAssetRefs, "assetRefs.optional"),
        ],
        &[],
        TransformPolicy::None,
        BoundsPolicy::None,
        &[EditorControlKind::InspectorOnly],
        &[],
    )
}
pub fn velocity_2d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "Velocity2D",
        "Velocity2D",
        "Velocity 2D",
        &[ComponentDomain::Motion2D],
        &[
            MetadataTraitKind::Motion2D,
            MetadataTraitKind::Simulatable,
            MetadataTraitKind::GenericEditable,
        ],
        &[p!(
            "velocity",
            "Velocity",
            EditorPropertyValueKind::Vec2,
            EditorPropertyEditorKind::Vec2,
            MetadataTraitKind::Motion2D,
            "motion2.velocity"
        )],
        &[],
        TransformPolicy::None,
        BoundsPolicy::None,
        &[EditorControlKind::InspectorOnly],
        &[],
    )
}
pub fn lifetime_descriptor() -> ComponentTypeDescriptor {
    generic_data_descriptor(
        "Lifetime",
        "Lifetime",
        "Lifetime",
        &[
            MetadataTraitKind::LifetimeLimited,
            MetadataTraitKind::Poolable,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "seconds",
                "Seconds",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::LifetimeLimited,
                "lifetime.properties"
            ),
            p!(
                "outcome",
                "Outcome",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::LifetimeLimited,
                "lifetime.outcome"
            ),
            p!(
                "pool",
                "Pool",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Poolable,
                "pool.properties"
            ),
        ],
    )
}
pub fn tile_map_marker_2d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "TileMapMarker2D",
        "TileMapMarker2D",
        "Tile Map Marker 2D",
        &[ComponentDomain::EditorOnly],
        &[
            MetadataTraitKind::Selectable,
            MetadataTraitKind::UsesTransform2D,
            MetadataTraitKind::HasBounds2D,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "tilemap_entity",
                "Tilemap Entity",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::GenericEditable,
                "tilemap.binding"
            ),
            p!(
                "symbol",
                "Symbol",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::GenericEditable,
                "tilemap.marker"
            ),
            p!(
                "index",
                "Index",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::GenericEditable,
                "tilemap.marker"
            ),
            p!(
                "offset",
                "Offset",
                EditorPropertyValueKind::Vec2,
                EditorPropertyEditorKind::Vec2,
                MetadataTraitKind::HasBounds2D,
                "bounds2.offset"
            ),
        ],
        &[],
        TransformPolicy::UsesEntityTransform2,
        BoundsPolicy::ComponentBounds2D { field: "offset" },
        &[EditorControlKind::InspectorOnly],
        &[],
    )
}
pub fn camera_follow_2d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "CameraFollow2D",
        "CameraFollow2D",
        "Camera Follow 2D",
        &[ComponentDomain::Camera],
        &[
            MetadataTraitKind::Camera,
            MetadataTraitKind::UsesTransform2D,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "target",
                "Target",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Camera,
                "camera.properties"
            ),
            p!(
                "offset",
                "Offset",
                EditorPropertyValueKind::Vec2,
                EditorPropertyEditorKind::Vec2,
                MetadataTraitKind::Camera,
                "camera.properties"
            ),
            p!(
                "lerp",
                "Lerp",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Camera,
                "camera.properties"
            ),
            p!(
                "sway_amount",
                "Sway Amount",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Camera,
                "camera.properties"
            ),
            p!(
                "sway_frequency",
                "Sway Frequency",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Camera,
                "camera.properties"
            ),
            p!(
                "lookahead_max_distance",
                "Lookahead Max Distance",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Camera,
                "camera.properties"
            ),
            p!(
                "lookahead_velocity_scale",
                "Lookahead Velocity Scale",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Camera,
                "camera.properties"
            ),
        ],
        &[],
        TransformPolicy::None,
        BoundsPolicy::None,
        &[EditorControlKind::InspectorOnly],
        &[],
    )
}
pub fn parallax_2d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "Parallax2D",
        "Parallax2D",
        "Parallax 2D",
        &[ComponentDomain::Render2D, ComponentDomain::Camera],
        &[
            MetadataTraitKind::Renderable2D,
            MetadataTraitKind::Camera,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "camera",
                "Camera",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Camera,
                "camera.properties"
            ),
            p!(
                "factor",
                "Factor",
                EditorPropertyValueKind::Vec2,
                EditorPropertyEditorKind::Vec2,
                MetadataTraitKind::Renderable2D,
                "render2d.content"
            ),
        ],
        &[],
        TransformPolicy::None,
        BoundsPolicy::None,
        &[EditorControlKind::InspectorOnly],
        &[],
    )
}
pub fn freeflight_motion_2d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "FreeflightMotion2D",
        "FreeflightMotion2D",
        "Freeflight Motion 2D",
        &[ComponentDomain::Motion2D],
        &[
            MetadataTraitKind::Motion2D,
            MetadataTraitKind::InputBindable,
            MetadataTraitKind::Simulatable,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "thrust_acceleration",
                "Thrust Acceleration",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning"
            ),
            p!(
                "turn_acceleration",
                "Turn Acceleration",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning"
            ),
            p!(
                "max_speed",
                "Max Speed",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning"
            ),
            p!(
                "reverse_acceleration",
                "Reverse Acceleration",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning"
            ),
            p!(
                "strafe_acceleration",
                "Strafe Acceleration",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning"
            ),
            p!(
                "linear_damping",
                "Linear Damping",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning"
            ),
            p!(
                "max_angular_speed",
                "Max Angular Speed",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning"
            ),
            p!(
                "turn_damping",
                "Turn Damping",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning"
            ),
            p!(ro "thrust_response_curve", "Thrust Response Curve", MetadataTraitKind::Motion2D, "motion2.tuning"),
            p!(ro "reverse_response_curve", "Reverse Response Curve", MetadataTraitKind::Motion2D, "motion2.tuning"),
            p!(ro "strafe_response_curve", "Strafe Response Curve", MetadataTraitKind::Motion2D, "motion2.tuning"),
            p!(ro "turn_response_curve", "Turn Response Curve", MetadataTraitKind::Motion2D, "motion2.tuning"),
        ],
        &[],
        TransformPolicy::None,
        BoundsPolicy::None,
        &[EditorControlKind::InspectorOnly],
        &[],
    )
}
pub fn kinematic_body_2d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "KinematicBody2D",
        "KinematicBody2D",
        "Kinematic Body 2D",
        &[ComponentDomain::Motion2D, ComponentDomain::Physics2D],
        &[
            MetadataTraitKind::Motion2D,
            MetadataTraitKind::Simulatable,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "velocity",
                "Velocity",
                EditorPropertyValueKind::Vec2,
                EditorPropertyEditorKind::Vec2,
                MetadataTraitKind::Motion2D,
                "motion2.velocity"
            ),
            p!(
                "gravity_scale",
                "Gravity Scale",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning"
            ),
            p!(
                "terminal_velocity",
                "Terminal Velocity",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning"
            ),
        ],
        &[],
        TransformPolicy::None,
        BoundsPolicy::None,
        &[EditorControlKind::InspectorOnly],
        &[],
    )
}
pub fn motion_controller_2d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "MotionController2D",
        "MotionController2D",
        "Motion Controller 2D",
        &[ComponentDomain::Motion2D],
        &[
            MetadataTraitKind::Motion2D,
            MetadataTraitKind::InputBindable,
            MetadataTraitKind::Simulatable,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "max_speed",
                "Max Speed",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning"
            ),
            p!(
                "acceleration",
                "Acceleration",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning"
            ),
            p!(
                "air_acceleration",
                "Air Acceleration",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning"
            ),
            p!(
                "deceleration",
                "Deceleration",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning"
            ),
            p!(
                "gravity",
                "Gravity",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning"
            ),
            p!(
                "jump_velocity",
                "Jump Velocity",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning"
            ),
            p!(
                "terminal_velocity",
                "Terminal Velocity",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning"
            ),
        ],
        &[],
        TransformPolicy::None,
        BoundsPolicy::None,
        &[EditorControlKind::InspectorOnly],
        &[],
    )
}
pub fn projectile_emitter_2d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "ProjectileEmitter2D",
        "ProjectileEmitter2D",
        "Projectile Emitter 2D",
        &[ComponentDomain::Motion2D, ComponentDomain::Data],
        &[
            MetadataTraitKind::EventSource,
            MetadataTraitKind::Motion2D,
            MetadataTraitKind::Poolable,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "pool",
                "Pool",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Poolable,
                "pool.properties"
            ),
            p!(
                "speed",
                "Speed",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning"
            ),
            p!(
                "spawn_offset",
                "Spawn Offset",
                EditorPropertyValueKind::Vec2,
                EditorPropertyEditorKind::Vec2,
                MetadataTraitKind::Motion2D,
                "motion2.tuning"
            ),
            p!(
                "inherit_velocity_scale",
                "Inherit Velocity Scale",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning"
            ),
        ],
        &[],
        TransformPolicy::UsesEntityTransform2,
        BoundsPolicy::EntityTransformPoint,
        &[EditorControlKind::InspectorOnly],
        &[],
    )
}
pub fn entity_pool_descriptor() -> ComponentTypeDescriptor {
    generic_data_descriptor(
        "EntityPool",
        "EntityPool",
        "Entity Pool",
        &[
            MetadataTraitKind::Poolable,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "pool",
                "Pool",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Poolable,
                "pool.properties"
            ),
            p!(ro "members", "Members", MetadataTraitKind::Poolable, "pool.members"),
        ],
    )
}
pub fn bounds_2d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "Bounds2D",
        "Bounds2D",
        "Bounds 2D",
        &[ComponentDomain::EditorOnly],
        &[
            MetadataTraitKind::HasBounds2D,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "size",
                "Size",
                EditorPropertyValueKind::Vec2,
                EditorPropertyEditorKind::Vec2,
                MetadataTraitKind::HasBounds2D,
                "bounds2.size"
            ),
            p!(
                "offset",
                "Offset",
                EditorPropertyValueKind::Vec2,
                EditorPropertyEditorKind::Vec2,
                MetadataTraitKind::HasBounds2D,
                "bounds2.offset"
            ),
        ],
        &[],
        TransformPolicy::None,
        BoundsPolicy::ComponentBounds2D { field: "size" },
        &[EditorControlKind::Rect2D],
        &[],
    )
}

pub fn camera_3d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "Camera3D",
        "Camera3D",
        "Camera 3D",
        &[ComponentDomain::Camera],
        &[
            MetadataTraitKind::Camera,
            MetadataTraitKind::RenderableViewportSource,
            MetadataTraitKind::UsesTransform3D,
            MetadataTraitKind::Selectable,
            MetadataTraitKind::GenericEditable,
        ],
        &[],
        &[],
        TransformPolicy::UsesEntityTransform3,
        BoundsPolicy::None,
        &[EditorControlKind::Transform3D],
        &[EditorPatchOpKind::SetTransform3],
    )
}

pub fn light_3d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "Light3D",
        "Light3D",
        "Light 3D",
        &[ComponentDomain::Render3D],
        &[
            MetadataTraitKind::Renderable3D,
            MetadataTraitKind::UsesTransform3D,
            MetadataTraitKind::GenericEditable,
        ],
        &[p!(
            "kind",
            "Light Kind",
            EditorPropertyValueKind::String,
            EditorPropertyEditorKind::Text,
            MetadataTraitKind::Renderable3D,
            "render3d.content"
        )],
        &[],
        TransformPolicy::UsesEntityTransform3,
        BoundsPolicy::None,
        &[EditorControlKind::Transform3D],
        &[EditorPatchOpKind::SetTransform3],
    )
}

pub fn mesh_3d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "Mesh3D",
        "Mesh3D",
        "Mesh 3D",
        &[ComponentDomain::Render3D],
        &[
            MetadataTraitKind::Renderable3D,
            MetadataTraitKind::UsesTransform3D,
            MetadataTraitKind::HasAssetRefs,
            MetadataTraitKind::GenericEditable,
        ],
        &[EditorPropertyDescriptor {
            path: "mesh",
            label: "Mesh",
            value_kind: EditorPropertyValueKind::AssetRef,
            access: EditorPropertyAccess::Editable,
            editor: EditorPropertyEditorKind::AssetPicker,
            asset_domain: Some(AssetDomain::Mesh),
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
        }],
        &[ComponentAssetRefDescriptor {
            field_path: "mesh",
            domain: AssetDomain::Mesh,
            required: true,
            trait_kind: MetadataTraitKind::HasAssetRefs,
            group: "assetRefs.primary",
        }],
        TransformPolicy::UsesEntityTransform3,
        BoundsPolicy::None,
        &[EditorControlKind::Transform3D],
        &[EditorPatchOpKind::SetTransform3],
    )
}

pub fn material_3d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "Material3D",
        "Material3D",
        "Material 3D",
        &[ComponentDomain::Render3D],
        &[
            MetadataTraitKind::Renderable3D,
            MetadataTraitKind::HasAssetRefs,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "label",
                "Label",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Renderable3D,
                "render3d.content"
            ),
            EditorPropertyDescriptor {
                path: "source",
                label: "Source",
                value_kind: EditorPropertyValueKind::AssetRef,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::AssetPicker,
                asset_domain: Some(AssetDomain::Material),
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
            p!(
                "albedo",
                "Albedo",
                EditorPropertyValueKind::Color,
                EditorPropertyEditorKind::Color,
                MetadataTraitKind::Renderable3D,
                "render3d.color"
            ),
        ],
        &[ComponentAssetRefDescriptor {
            field_path: "source",
            domain: AssetDomain::Material,
            required: true,
            trait_kind: MetadataTraitKind::HasAssetRefs,
            group: "assetRefs.primary",
        }],
        TransformPolicy::None,
        BoundsPolicy::None,
        &[EditorControlKind::InspectorOnly],
        &[],
    )
}

pub fn text_3d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "Text3D",
        "Text3D",
        "Text 3D",
        &[ComponentDomain::Render3D],
        &[
            MetadataTraitKind::Renderable3D,
            MetadataTraitKind::UsesTransform3D,
            MetadataTraitKind::HasAssetRefs,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "content",
                "Content",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Renderable3D,
                "render3d.content"
            ),
            EditorPropertyDescriptor {
                path: "font",
                label: "Font",
                value_kind: EditorPropertyValueKind::AssetRef,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::AssetPicker,
                asset_domain: Some(AssetDomain::Font),
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
            p!(
                "size",
                "Size",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Renderable3D,
                "render3d.content"
            ),
        ],
        &[ComponentAssetRefDescriptor {
            field_path: "font",
            domain: AssetDomain::Font,
            required: true,
            trait_kind: MetadataTraitKind::HasAssetRefs,
            group: "assetRefs.primary",
        }],
        TransformPolicy::UsesEntityTransform3,
        BoundsPolicy::None,
        &[EditorControlKind::Transform3D],
        &[EditorPatchOpKind::SetTransform3],
    )
}

pub fn static_collider_2d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        "StaticCollider2D",
        "StaticCollider2D",
        "Static Collider 2D",
        &[ComponentDomain::Physics2D],
        &[
            MetadataTraitKind::Collidable2D,
            MetadataTraitKind::UsesTransform2D,
            MetadataTraitKind::Selectable,
            MetadataTraitKind::HasBounds2D,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "size",
                "Size",
                EditorPropertyValueKind::Vec2,
                EditorPropertyEditorKind::Vec2,
                MetadataTraitKind::HasBounds2D,
                "bounds2.size"
            ),
            p!(
                "offset",
                "Offset",
                EditorPropertyValueKind::Vec2,
                EditorPropertyEditorKind::Vec2,
                MetadataTraitKind::HasBounds2D,
                "bounds2.offset"
            ),
            p!(
                "layer",
                "Layer",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Collidable2D,
                "collision.layer"
            ),
            p!(ro "mask", "Mask", MetadataTraitKind::Collidable2D, "collision.mask"),
        ],
        &[],
        TransformPolicy::UsesEntityTransform2,
        BoundsPolicy::DerivedFromCollider2D,
        &[EditorControlKind::Collider2D],
        &[EditorPatchOpKind::SetColliderShape],
    )
}

/// Built-in registry for engine-owned descriptors.
///
/// New plugin-owned component descriptors must be registered through
/// ComponentMetadataProvider instead of being added here.
pub fn default_component_registry() -> ComponentRegistry {
    ComponentRegistry::new([
        camera_3d_descriptor(),
        light_3d_descriptor(),
        text_3d_descriptor(),
        lightmap_2d_source_descriptor(),
        trigger_2d_descriptor(),
        script_component_descriptor(),
        behavior_descriptor(),
        event_pipeline_descriptor(),
        input_action_map_descriptor(),
        ui_document_descriptor(),
        ui_model_bindings_descriptor(),
        ui_theme_set_descriptor(),
        aabb_collider_2d_descriptor(),
        static_collider_2d_descriptor(),
        circle_collider_2d_descriptor(),
        velocity_2d_descriptor(),
        lifetime_descriptor(),
        tile_map_marker_2d_descriptor(),
        camera_follow_2d_descriptor(),
        parallax_2d_descriptor(),
        freeflight_motion_2d_descriptor(),
        kinematic_body_2d_descriptor(),
        motion_controller_2d_descriptor(),
        projectile_emitter_2d_descriptor(),
        entity_pool_descriptor(),
        bounds_2d_descriptor(),
        mesh_3d_descriptor(),
        material_3d_descriptor(),
    ])
}

pub fn component_registry_with_providers(
    providers: Option<&crate::ComponentMetadataProviderRegistry>,
) -> ComponentRegistry {
    let mut registry = default_component_registry();
    if let Some(providers) = providers {
        providers.apply_all(&mut registry);
    }
    registry
}

pub fn component_registry_for_runtime(runtime: &amigo_runtime::Runtime) -> ComponentRegistry {
    let providers = runtime.resolve::<crate::ComponentMetadataProviderRegistry>();
    component_registry_with_providers(providers.as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_level_descriptors_use_scene_owner_scope() {
        assert_eq!(
            input_action_map_descriptor().owner_scopes,
            SCENE_OWNER_SCOPES
        );
        assert_eq!(event_pipeline_descriptor().owner_scopes, SCENE_OWNER_SCOPES);
    }

    #[test]
    fn default_registry_excludes_plugin_owned_scene_components() {
        let registry = default_component_registry();
        for component in [
            "Camera2D",
            "DepthMap2D",
            "DepthAuxMap2D",
            "GlobalLight2D",
            "BeaconLight2D",
        ] {
            assert!(
                registry.descriptor(component).is_none(),
                "{component} metadata must be registered by its plugin"
            );
        }
    }
}
