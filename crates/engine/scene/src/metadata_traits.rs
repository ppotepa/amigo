use serde::{Deserialize, Serialize};

use crate::{EditorControlKind, EditorPatchOpKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MetadataTraitKind {
    SceneDocument,
    HasEntities,
    HasScripts,
    HasDiagnostics,
    HasAssetUsages,
    HasUiDocuments,
    HasIdentity,
    HasVisibility,
    HasComponents,
    Transformable2D,
    UsesTransform2D,
    HasBounds2D,
    Selectable,
    Renderable2D,
    RenderLayered2D,
    LightReceiver2D,
    HasAssetRefs,
    Collidable2D,
    Trigger2D,
    Scriptable,
    EventSource,
    EventListener,
    SceneTransitionSource,
    InputBindable,
    UiEditable,
    HasUiTree,
    DataBindable,
    GenericEditable,
    RuntimeControllable,
    Patchable,
    HasEditorControls,
    DiagnosticSource,
    Camera,
    RenderableViewportSource,
    Motion2D,
    Simulatable,
    Poolable,
    LifetimeLimited,
    Transformable3D,
    UsesTransform3D,
    HasBounds3D,
    Renderable3D,
}

impl MetadataTraitKind {
    pub const fn id(self) -> &'static str {
        match self {
            Self::SceneDocument => "SceneDocument",
            Self::HasEntities => "HasEntities",
            Self::HasScripts => "HasScripts",
            Self::HasDiagnostics => "HasDiagnostics",
            Self::HasAssetUsages => "HasAssetUsages",
            Self::HasUiDocuments => "HasUiDocuments",
            Self::HasIdentity => "HasIdentity",
            Self::HasVisibility => "HasVisibility",
            Self::HasComponents => "HasComponents",
            Self::Transformable2D => "Transformable2D",
            Self::UsesTransform2D => "UsesTransform2D",
            Self::HasBounds2D => "HasBounds2D",
            Self::Selectable => "Selectable",
            Self::Renderable2D => "Renderable2D",
            Self::RenderLayered2D => "RenderLayered2D",
            Self::LightReceiver2D => "LightReceiver2D",
            Self::HasAssetRefs => "HasAssetRefs",
            Self::Collidable2D => "Collidable2D",
            Self::Trigger2D => "Trigger2D",
            Self::Scriptable => "Scriptable",
            Self::EventSource => "EventSource",
            Self::EventListener => "EventListener",
            Self::SceneTransitionSource => "SceneTransitionSource",
            Self::InputBindable => "InputBindable",
            Self::UiEditable => "UiEditable",
            Self::HasUiTree => "HasUiTree",
            Self::DataBindable => "DataBindable",
            Self::GenericEditable => "GenericEditable",
            Self::RuntimeControllable => "RuntimeControllable",
            Self::Patchable => "Patchable",
            Self::HasEditorControls => "HasEditorControls",
            Self::DiagnosticSource => "DiagnosticSource",
            Self::Camera => "Camera",
            Self::RenderableViewportSource => "RenderableViewportSource",
            Self::Motion2D => "Motion2D",
            Self::Simulatable => "Simulatable",
            Self::Poolable => "Poolable",
            Self::LifetimeLimited => "LifetimeLimited",
            Self::Transformable3D => "Transformable3D",
            Self::UsesTransform3D => "UsesTransform3D",
            Self::HasBounds3D => "HasBounds3D",
            Self::Renderable3D => "Renderable3D",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MetadataTargetScope {
    Scene,
    Entity,
    Component,
    ComponentProperty,
    Asset,
    AssetRef,
    Script,
    UiDocument,
    UiNode,
    Diagnostic,
}

impl MetadataTargetScope {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Scene => "Scene",
            Self::Entity => "Entity",
            Self::Component => "Component",
            Self::ComponentProperty => "ComponentProperty",
            Self::Asset => "Asset",
            Self::AssetRef => "AssetRef",
            Self::Script => "Script",
            Self::UiDocument => "UiDocument",
            Self::UiNode => "UiNode",
            Self::Diagnostic => "Diagnostic",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataTraitPropertyGroupDescriptor {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataTraitDiagnosticDescriptor {
    pub code: &'static str,
    pub label: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataTraitDescriptor {
    pub kind: MetadataTraitKind,
    pub label: &'static str,
    pub description: &'static str,
    pub applies_to: Vec<MetadataTargetScope>,
    pub expected_yaml_shapes: Vec<&'static str>,
    pub property_groups: Vec<MetadataTraitPropertyGroupDescriptor>,
    pub controls: Vec<EditorControlKind>,
    pub patch_ops: Vec<EditorPatchOpKind>,
    pub diagnostics: Vec<MetadataTraitDiagnosticDescriptor>,
}

const fn group(
    id: &'static str,
    label: &'static str,
    description: &'static str,
) -> MetadataTraitPropertyGroupDescriptor {
    MetadataTraitPropertyGroupDescriptor {
        id,
        label,
        description,
    }
}

const fn diagnostic(
    code: &'static str,
    label: &'static str,
    description: &'static str,
) -> MetadataTraitDiagnosticDescriptor {
    MetadataTraitDiagnosticDescriptor {
        code,
        label,
        description,
    }
}

pub fn default_metadata_trait_descriptors() -> Vec<MetadataTraitDescriptor> {
    vec![
        scene_document_trait(),
        simple_trait(
            MetadataTraitKind::HasEntities,
            "Has Entities",
            "Scene owns entity instances.",
            vec![MetadataTargetScope::Scene],
        ),
        simple_trait(
            MetadataTraitKind::HasScripts,
            "Has Scripts",
            "Scene references scripts.",
            vec![MetadataTargetScope::Scene],
        ),
        simple_trait(
            MetadataTraitKind::HasDiagnostics,
            "Has Diagnostics",
            "Target can report diagnostics.",
            vec![MetadataTargetScope::Scene, MetadataTargetScope::Entity],
        ),
        simple_trait(
            MetadataTraitKind::HasAssetUsages,
            "Has Asset Usages",
            "Target references asset usages.",
            vec![MetadataTargetScope::Scene],
        ),
        simple_trait(
            MetadataTraitKind::HasUiDocuments,
            "Has UI Documents",
            "Scene contains UI documents.",
            vec![MetadataTargetScope::Scene],
        ),
        simple_trait(
            MetadataTraitKind::HasIdentity,
            "Has Identity",
            "Entity has stable identity fields.",
            vec![MetadataTargetScope::Entity],
        ),
        simple_trait(
            MetadataTraitKind::HasVisibility,
            "Has Visibility",
            "Entity has visibility state.",
            vec![MetadataTargetScope::Entity],
        ),
        simple_trait(
            MetadataTraitKind::HasComponents,
            "Has Components",
            "Entity owns component instances.",
            vec![MetadataTargetScope::Entity],
        ),
        transformable_2d_trait(),
        uses_transform_2d_trait(),
        renderable_2d_trait(),
        simple_trait(
            MetadataTraitKind::RenderLayered2D,
            "Render Layered 2D",
            "2D renderable assigned to a named render layer.",
            vec![MetadataTargetScope::Component],
        ),
        simple_trait(
            MetadataTraitKind::LightReceiver2D,
            "Light Receiver 2D",
            "2D renderable can receive sampled or dynamic light.",
            vec![MetadataTargetScope::Component],
        ),
        has_bounds_2d_trait(),
        has_asset_refs_trait(),
        simple_trait(
            MetadataTraitKind::Selectable,
            "Selectable",
            "Target can be selected in editor.",
            vec![MetadataTargetScope::Entity, MetadataTargetScope::Component],
        ),
        collision_trait(),
        simple_trait(
            MetadataTraitKind::Trigger2D,
            "Trigger 2D",
            "2D trigger volume that can emit events.",
            vec![MetadataTargetScope::Component],
        ),
        scriptable_trait(),
        simple_trait(
            MetadataTraitKind::EventSource,
            "Event Source",
            "Target can emit events.",
            vec![MetadataTargetScope::Component],
        ),
        simple_trait(
            MetadataTraitKind::EventListener,
            "Event Listener",
            "Target can listen to events.",
            vec![MetadataTargetScope::Component],
        ),
        simple_trait(
            MetadataTraitKind::SceneTransitionSource,
            "Scene Transition Source",
            "Target can request scene transitions.",
            vec![MetadataTargetScope::Component],
        ),
        input_bindable_trait(),
        ui_editable_trait(),
        simple_trait(
            MetadataTraitKind::HasUiTree,
            "Has UI Tree",
            "Target owns UI node tree data.",
            vec![
                MetadataTargetScope::Component,
                MetadataTargetScope::UiDocument,
            ],
        ),
        simple_trait(
            MetadataTraitKind::DataBindable,
            "Data Bindable",
            "Target binds UI/data values.",
            vec![MetadataTargetScope::Component],
        ),
        generic_editable_trait(),
        simple_trait(
            MetadataTraitKind::RuntimeControllable,
            "Runtime Controllable",
            "Target exposes runtime/script controls.",
            vec![MetadataTargetScope::Component],
        ),
        simple_trait(
            MetadataTraitKind::Patchable,
            "Patchable",
            "Target supports metadata patch operations.",
            vec![MetadataTargetScope::Component],
        ),
        simple_trait(
            MetadataTraitKind::HasEditorControls,
            "Has Editor Controls",
            "Target exposes editor viewport controls.",
            vec![MetadataTargetScope::Component],
        ),
        simple_trait(
            MetadataTraitKind::DiagnosticSource,
            "Diagnostic Source",
            "Target can own diagnostics.",
            vec![MetadataTargetScope::Entity, MetadataTargetScope::Component],
        ),
        camera_trait(),
        simple_trait(
            MetadataTraitKind::RenderableViewportSource,
            "Viewport Source",
            "Target contributes viewport camera/source data.",
            vec![MetadataTargetScope::Component],
        ),
        motion_2d_trait(),
        simple_trait(
            MetadataTraitKind::Simulatable,
            "Simulatable",
            "Target participates in simulation.",
            vec![MetadataTargetScope::Component],
        ),
        poolable_trait(),
        lifetime_limited_trait(),
        transformable_3d_trait(),
        uses_transform_3d_trait(),
        has_bounds_3d_trait(),
        renderable_3d_trait(),
    ]
}

pub fn metadata_trait_descriptor(kind: MetadataTraitKind) -> Option<MetadataTraitDescriptor> {
    default_metadata_trait_descriptors()
        .into_iter()
        .find(|descriptor| descriptor.kind == kind)
}

fn simple_trait(
    kind: MetadataTraitKind,
    label: &'static str,
    description: &'static str,
    applies_to: Vec<MetadataTargetScope>,
) -> MetadataTraitDescriptor {
    MetadataTraitDescriptor {
        kind,
        label,
        description,
        applies_to,
        expected_yaml_shapes: vec![],
        property_groups: Vec::new(),
        controls: Vec::new(),
        patch_ops: Vec::new(),
        diagnostics: Vec::new(),
    }
}

fn scene_document_trait() -> MetadataTraitDescriptor {
    MetadataTraitDescriptor {
        kind: MetadataTraitKind::SceneDocument,
        label: "Scene Document",
        description: "Root scene document metadata.",
        applies_to: vec![MetadataTargetScope::Scene],
        expected_yaml_shapes: vec!["entities", "scripts"],
        property_groups: Vec::new(),
        controls: Vec::new(),
        patch_ops: Vec::new(),
        diagnostics: Vec::new(),
    }
}

pub fn transformable_2d_trait() -> MetadataTraitDescriptor {
    MetadataTraitDescriptor {
        kind: MetadataTraitKind::Transformable2D,
        label: "Transformable 2D",
        description: "Target owns editable 2D transform fields.",
        applies_to: vec![MetadataTargetScope::Entity, MetadataTargetScope::Component],
        expected_yaml_shapes: vec![
            "transform2.translation.x",
            "transform2.translation.y",
            "transform2.rotation_radians",
            "transform2.scale.x",
            "transform2.scale.y",
        ],
        property_groups: vec![
            group(
                "transform2.translation",
                "Translation",
                "2D translation fields.",
            ),
            group("transform2.rotation", "Rotation", "2D rotation field."),
            group("transform2.scale", "Scale", "2D scale fields."),
        ],
        controls: vec![EditorControlKind::Transform2D],
        patch_ops: vec![EditorPatchOpKind::SetTransform2],
        diagnostics: vec![diagnostic(
            "transform2.invalid",
            "Invalid Transform2",
            "Transform2 fields are missing or malformed.",
        )],
    }
}

pub fn uses_transform_2d_trait() -> MetadataTraitDescriptor {
    MetadataTraitDescriptor {
        kind: MetadataTraitKind::UsesTransform2D,
        label: "Uses Transform 2D",
        description: "Component uses its owner entity 2D transform.",
        applies_to: vec![MetadataTargetScope::Component],
        expected_yaml_shapes: vec![],
        property_groups: Vec::new(),
        controls: Vec::new(),
        patch_ops: Vec::new(),
        diagnostics: Vec::new(),
    }
}

pub fn renderable_2d_trait() -> MetadataTraitDescriptor {
    MetadataTraitDescriptor {
        kind: MetadataTraitKind::Renderable2D,
        label: "Renderable 2D",
        description: "Target contributes visible 2D render content.",
        applies_to: vec![MetadataTargetScope::Entity, MetadataTargetScope::Component],
        expected_yaml_shapes: vec![
            "render_layer",
            "z_index",
            "tint",
            "color",
            "fill_color",
            "stroke_color",
            "opacity",
            "content",
        ],
        property_groups: vec![
            group(
                "render2d.order",
                "Render Order",
                "Layer and z-index fields.",
            ),
            group(
                "render2d.color",
                "Color",
                "Tint, fill, stroke and opacity fields.",
            ),
            group(
                "render2d.content",
                "Content",
                "Text, sprite, shape or visual content fields.",
            ),
        ],
        controls: Vec::new(),
        patch_ops: Vec::new(),
        diagnostics: Vec::new(),
    }
}

pub fn has_bounds_2d_trait() -> MetadataTraitDescriptor {
    MetadataTraitDescriptor {
        kind: MetadataTraitKind::HasBounds2D,
        label: "Has Bounds 2D",
        description: "Target exposes 2D bounds for selection, collision or layout.",
        applies_to: vec![MetadataTargetScope::Entity, MetadataTargetScope::Component],
        expected_yaml_shapes: vec![
            "size.x", "size.y", "bounds.x", "bounds.y", "offset.x", "offset.y", "radius",
        ],
        property_groups: vec![
            group("bounds2.size", "Size", "2D size fields."),
            group("bounds2.offset", "Offset", "2D bounds offset fields."),
            group("bounds2.radius", "Radius", "Circular bounds radius field."),
        ],
        controls: vec![EditorControlKind::Rect2D],
        patch_ops: Vec::new(),
        diagnostics: vec![diagnostic(
            "bounds2.invalid",
            "Invalid Bounds2D",
            "Bounds fields are missing or malformed.",
        )],
    }
}

pub fn has_asset_refs_trait() -> MetadataTraitDescriptor {
    MetadataTraitDescriptor {
        kind: MetadataTraitKind::HasAssetRefs,
        label: "Has Asset References",
        description: "Target references one or more assets.",
        applies_to: vec![
            MetadataTargetScope::Scene,
            MetadataTargetScope::Entity,
            MetadataTargetScope::Component,
            MetadataTargetScope::UiDocument,
        ],
        expected_yaml_shapes: vec![
            "texture", "font", "script", "tileset", "ruleset", "material", "mesh", "theme",
        ],
        property_groups: vec![
            group(
                "assetRefs.primary",
                "Primary Asset References",
                "Main asset references.",
            ),
            group(
                "assetRefs.optional",
                "Optional Asset References",
                "Optional asset references.",
            ),
        ],
        controls: Vec::new(),
        patch_ops: Vec::new(),
        diagnostics: vec![diagnostic(
            "assetRef.missingRequired",
            "Missing Required Asset",
            "Required asset reference is empty or unresolved.",
        )],
    }
}

fn collision_trait() -> MetadataTraitDescriptor {
    MetadataTraitDescriptor {
        kind: MetadataTraitKind::Collidable2D,
        label: "Collidable 2D",
        description: "Target contributes 2D collision data.",
        applies_to: vec![MetadataTargetScope::Component],
        expected_yaml_shapes: vec!["size", "offset", "radius", "layer", "mask"],
        property_groups: vec![
            group(
                "collision.shape",
                "Collision Shape",
                "Collider shape fields.",
            ),
            group(
                "collision.layer",
                "Collision Layer",
                "Collision layer field.",
            ),
            group("collision.mask", "Collision Mask", "Collision mask field."),
        ],
        controls: vec![EditorControlKind::Collider2D],
        patch_ops: vec![EditorPatchOpKind::SetColliderShape],
        diagnostics: Vec::new(),
    }
}

fn scriptable_trait() -> MetadataTraitDescriptor {
    MetadataTraitDescriptor {
        kind: MetadataTraitKind::Scriptable,
        label: "Scriptable",
        description: "Target references or executes scripts/behavior.",
        applies_to: vec![MetadataTargetScope::Component],
        expected_yaml_shapes: vec!["script", "kind", "action"],
        property_groups: vec![group(
            "script.primary",
            "Script",
            "Script or behavior identity fields.",
        )],
        controls: Vec::new(),
        patch_ops: Vec::new(),
        diagnostics: Vec::new(),
    }
}

fn input_bindable_trait() -> MetadataTraitDescriptor {
    MetadataTraitDescriptor {
        kind: MetadataTraitKind::InputBindable,
        label: "Input Bindable",
        description: "Target binds input actions.",
        applies_to: vec![MetadataTargetScope::Component],
        expected_yaml_shapes: vec!["actions", "input", "active"],
        property_groups: vec![
            group(
                "input.identity",
                "Input Identity",
                "Input map identity fields.",
            ),
            group("input.state", "Input State", "Input activation fields."),
            group("input.actions", "Actions", "Input action list."),
        ],
        controls: Vec::new(),
        patch_ops: Vec::new(),
        diagnostics: Vec::new(),
    }
}

fn ui_editable_trait() -> MetadataTraitDescriptor {
    MetadataTraitDescriptor {
        kind: MetadataTraitKind::UiEditable,
        label: "UI Editable",
        description: "Target owns editable UI metadata.",
        applies_to: vec![
            MetadataTargetScope::Component,
            MetadataTargetScope::UiDocument,
        ],
        expected_yaml_shapes: vec!["target", "root", "bindings", "themes"],
        property_groups: vec![
            group("ui.identity", "UI Identity", "UI target identity fields."),
            group("ui.tree", "UI Tree", "UI node tree fields."),
            group("ui.bindings", "Bindings", "UI model bindings."),
            group("ui.theme", "Theme", "UI theme fields."),
        ],
        controls: Vec::new(),
        patch_ops: Vec::new(),
        diagnostics: Vec::new(),
    }
}

fn generic_editable_trait() -> MetadataTraitDescriptor {
    MetadataTraitDescriptor {
        kind: MetadataTraitKind::GenericEditable,
        label: "Generic Editable",
        description: "Target can be inspected by generic metadata-driven editor UI.",
        applies_to: vec![MetadataTargetScope::Component],
        expected_yaml_shapes: vec![],
        property_groups: vec![group(
            "genericEditable.properties",
            "Properties",
            "Generic component properties.",
        )],
        controls: Vec::new(),
        patch_ops: Vec::new(),
        diagnostics: Vec::new(),
    }
}

fn camera_trait() -> MetadataTraitDescriptor {
    MetadataTraitDescriptor {
        kind: MetadataTraitKind::Camera,
        label: "Camera",
        description: "Target defines camera behavior.",
        applies_to: vec![MetadataTargetScope::Component],
        expected_yaml_shapes: vec!["zoom", "active", "viewport", "target", "offset", "lerp"],
        property_groups: vec![group("camera.properties", "Camera", "Camera fields.")],
        controls: vec![EditorControlKind::Camera2D],
        patch_ops: vec![EditorPatchOpKind::SetCamera2D],
        diagnostics: Vec::new(),
    }
}

fn motion_2d_trait() -> MetadataTraitDescriptor {
    MetadataTraitDescriptor {
        kind: MetadataTraitKind::Motion2D,
        label: "Motion 2D",
        description: "Target contributes 2D motion or simulation fields.",
        applies_to: vec![MetadataTargetScope::Component],
        expected_yaml_shapes: vec![
            "velocity",
            "max_speed",
            "acceleration",
            "gravity",
            "jump_velocity",
        ],
        property_groups: vec![
            group("motion2.velocity", "Velocity", "Velocity fields."),
            group("motion2.tuning", "Motion Tuning", "Motion tuning fields."),
        ],
        controls: Vec::new(),
        patch_ops: Vec::new(),
        diagnostics: Vec::new(),
    }
}

fn poolable_trait() -> MetadataTraitDescriptor {
    MetadataTraitDescriptor {
        kind: MetadataTraitKind::Poolable,
        label: "Poolable",
        description: "Target participates in entity pooling.",
        applies_to: vec![MetadataTargetScope::Component],
        expected_yaml_shapes: vec!["pool", "pool_id", "members"],
        property_groups: vec![
            group("pool.properties", "Pool", "Pool configuration fields."),
            group("pool.members", "Pool Members", "Pool membership fields."),
        ],
        controls: Vec::new(),
        patch_ops: Vec::new(),
        diagnostics: Vec::new(),
    }
}

fn lifetime_limited_trait() -> MetadataTraitDescriptor {
    MetadataTraitDescriptor {
        kind: MetadataTraitKind::LifetimeLimited,
        label: "Lifetime Limited",
        description: "Target has finite lifetime behavior.",
        applies_to: vec![MetadataTargetScope::Component],
        expected_yaml_shapes: vec!["seconds", "despawn", "on_expire"],
        property_groups: vec![
            group("lifetime.properties", "Lifetime", "Lifetime timing fields."),
            group(
                "lifetime.outcome",
                "Outcome",
                "Lifetime expiration behavior.",
            ),
        ],
        controls: Vec::new(),
        patch_ops: Vec::new(),
        diagnostics: Vec::new(),
    }
}
fn transformable_3d_trait() -> MetadataTraitDescriptor {
    MetadataTraitDescriptor {
        kind: MetadataTraitKind::Transformable3D,
        label: "Transformable 3D",
        description: "Target owns editable 3D transform fields.",
        applies_to: vec![MetadataTargetScope::Entity, MetadataTargetScope::Component],
        expected_yaml_shapes: vec![
            "transform3.translation.x",
            "transform3.translation.y",
            "transform3.translation.z",
            "transform3.rotation",
            "transform3.scale.x",
            "transform3.scale.y",
            "transform3.scale.z",
        ],
        property_groups: vec![
            group(
                "transform3.translation",
                "Translation",
                "3D translation fields.",
            ),
            group("transform3.rotation", "Rotation", "3D rotation fields."),
            group("transform3.scale", "Scale", "3D scale fields."),
        ],
        controls: vec![EditorControlKind::Transform3D],
        patch_ops: vec![EditorPatchOpKind::SetTransform3],
        diagnostics: Vec::new(),
    }
}

fn uses_transform_3d_trait() -> MetadataTraitDescriptor {
    MetadataTraitDescriptor {
        kind: MetadataTraitKind::UsesTransform3D,
        label: "Uses Transform 3D",
        description: "Component uses its owner entity 3D transform.",
        applies_to: vec![MetadataTargetScope::Component],
        expected_yaml_shapes: vec![],
        property_groups: Vec::new(),
        controls: Vec::new(),
        patch_ops: Vec::new(),
        diagnostics: Vec::new(),
    }
}

fn has_bounds_3d_trait() -> MetadataTraitDescriptor {
    MetadataTraitDescriptor {
        kind: MetadataTraitKind::HasBounds3D,
        label: "Has Bounds 3D",
        description: "Target exposes 3D bounds for selection, collision or layout.",
        applies_to: vec![MetadataTargetScope::Entity, MetadataTargetScope::Component],
        expected_yaml_shapes: vec!["size.x", "size.y", "size.z", "bounds", "offset", "radius"],
        property_groups: vec![
            group("bounds3.size", "Size", "3D size fields."),
            group("bounds3.offset", "Offset", "3D bounds offset fields."),
        ],
        controls: Vec::new(),
        patch_ops: Vec::new(),
        diagnostics: Vec::new(),
    }
}

fn renderable_3d_trait() -> MetadataTraitDescriptor {
    MetadataTraitDescriptor {
        kind: MetadataTraitKind::Renderable3D,
        label: "Renderable 3D",
        description: "Target contributes visible 3D render content.",
        applies_to: vec![MetadataTargetScope::Entity, MetadataTargetScope::Component],
        expected_yaml_shapes: vec!["mesh", "material", "albedo", "content", "font", "size"],
        property_groups: vec![
            group(
                "render3d.content",
                "Content",
                "3D mesh, text or light content fields.",
            ),
            group(
                "render3d.color",
                "Color",
                "3D material or light color fields.",
            ),
        ],
        controls: Vec::new(),
        patch_ops: Vec::new(),
        diagnostics: Vec::new(),
    }
}
