use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::MetadataTraitKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ComponentKind {
    Camera2D,
    Camera3D,
    Light3D,
    Sprite2D,
    TileMap2D,
    Text2D,
    VectorShape2D,
    EntityPool,
    Lifetime,
    ProjectileEmitter2D,
    InputActionMap,
    Behavior,
    EventPipeline,
    UiModelBindings,
    ScriptComponent,
    ParticleEmitter2D,
    Velocity2D,
    Bounds2D,
    FreeflightMotion2D,
    KinematicBody2D,
    AabbCollider2D,
    StaticCollider2D,
    CircleCollider2D,
    Trigger2D,
    MotionController2D,
    CameraFollow2D,
    Parallax2D,
    TileMapMarker2D,
    Mesh3D,
    Material3D,
    Text3D,
    UiDocument,
    UiThemeSet,
}

impl ComponentKind {
    pub fn all() -> &'static [ComponentKind] {
        &[
            ComponentKind::Camera2D,
            ComponentKind::Camera3D,
            ComponentKind::Light3D,
            ComponentKind::Sprite2D,
            ComponentKind::TileMap2D,
            ComponentKind::Text2D,
            ComponentKind::VectorShape2D,
            ComponentKind::EntityPool,
            ComponentKind::Lifetime,
            ComponentKind::ProjectileEmitter2D,
            ComponentKind::InputActionMap,
            ComponentKind::Behavior,
            ComponentKind::EventPipeline,
            ComponentKind::UiModelBindings,
            ComponentKind::ScriptComponent,
            ComponentKind::ParticleEmitter2D,
            ComponentKind::Velocity2D,
            ComponentKind::Bounds2D,
            ComponentKind::FreeflightMotion2D,
            ComponentKind::KinematicBody2D,
            ComponentKind::AabbCollider2D,
            ComponentKind::StaticCollider2D,
            ComponentKind::CircleCollider2D,
            ComponentKind::Trigger2D,
            ComponentKind::MotionController2D,
            ComponentKind::CameraFollow2D,
            ComponentKind::Parallax2D,
            ComponentKind::TileMapMarker2D,
            ComponentKind::Mesh3D,
            ComponentKind::Material3D,
            ComponentKind::Text3D,
            ComponentKind::UiDocument,
            ComponentKind::UiThemeSet,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ComponentDomain {
    Render2D,
    Render3D,
    Physics2D,
    Motion2D,
    Scripting,
    Audio,
    UI,
    Camera,
    Particles,
    Data,
    EditorOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ComponentCapability {
    Renderable2D,
    Renderable3D,
    Transformable2D,
    Transformable3D,
    Selectable,
    HasBounds2D,
    HasBounds3D,
    HasAssetRefs,
    HasEditorControl,
    Simulatable,
    Collidable2D,
    Trigger2D,
    Scriptable,
    UiEditable,
    Instantiable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AssetDomain {
    Image,
    Sprite,
    Spritesheet,
    TileSet,
    TileRuleSet,
    TileMap,
    Audio,
    Font,
    Scene,
    Prefab,
    Script,
    Material,
    Mesh,
    ParticlePreset,
    CursorPack,
    UiTheme,
    Raw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EditorControlKind {
    Transform2D,
    Transform3D,
    Rect2D,
    TextBounds2D,
    VectorVertex2D,
    TileMapBrush2D,
    Collider2D,
    Trigger2D,
    Camera2D,
    AudioRadius2D,
    InspectorOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EditorPatchOpKind {
    SetTransform2,
    SetTransform3,
    SetTextContent,
    SetTextBounds,
    SetVectorPoints,
    SetTileCell,
    ResizeTileMap,
    SetColliderShape,
    SetCamera2D,
    SetPrefabOverride,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformPolicy {
    None,
    UsesEntityTransform2,
    UsesEntityTransform3,
    ComponentLocal2D,
    ComponentLocal3D,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BoundsPolicy {
    None,
    EntityTransformPoint,
    ComponentBounds2D { field: &'static str },
    DerivedFromGeometry2D,
    DerivedFromTileMap,
    DerivedFromCollider2D,
    CameraViewport2D,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentAssetRefDescriptor {
    pub field_path: &'static str,
    pub domain: AssetDomain,
    pub required: bool,
    pub trait_kind: MetadataTraitKind,
    pub group: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EditorPropertyValueKind {
    String,
    Number,
    Bool,
    Vec2,
    Vec3,
    Color,
    AssetRef,
    Enum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EditorPropertyAccess {
    ReadOnly,
    Editable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EditorPropertyEditorKind {
    Text,
    Number,
    Checkbox,
    Vec2,
    Vec3,
    Color,
    AssetPicker,
    EnumSelect,
    ReadOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorPropertyDescriptor {
    pub path: &'static str,
    pub label: &'static str,
    pub value_kind: EditorPropertyValueKind,
    pub access: EditorPropertyAccess,
    pub editor: EditorPropertyEditorKind,
    pub asset_domain: Option<AssetDomain>,
    pub trait_kind: Option<MetadataTraitKind>,
    pub group: &'static str,
    pub patch_op: Option<EditorPatchOpKind>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentTypeDescriptor {
    pub kind: ComponentKind,
    pub type_name: &'static str,
    pub label: &'static str,
    pub domains: &'static [ComponentDomain],
    pub capabilities: &'static [ComponentCapability],
    pub metadata_traits: &'static [MetadataTraitKind],
    pub asset_refs: &'static [ComponentAssetRefDescriptor],
    pub properties: &'static [EditorPropertyDescriptor],
    pub transform_policy: TransformPolicy,
    pub bounds_policy: BoundsPolicy,
    pub editor_controls: &'static [EditorControlKind],
    pub patch_ops: &'static [EditorPatchOpKind],
}

impl ComponentTypeDescriptor {
    pub fn has(&self, capability: ComponentCapability) -> bool {
        self.capabilities.contains(&capability)
    }

    pub fn has_trait(&self, trait_kind: MetadataTraitKind) -> bool {
        self.metadata_traits.contains(&trait_kind)
    }
}

#[derive(Debug, Clone)]
pub struct ComponentRegistry {
    descriptors: BTreeMap<ComponentKind, ComponentTypeDescriptor>,
}

impl ComponentRegistry {
    pub fn new(descriptors: impl IntoIterator<Item = ComponentTypeDescriptor>) -> Self {
        Self {
            descriptors: descriptors
                .into_iter()
                .map(|descriptor| (descriptor.kind, descriptor))
                .collect(),
        }
    }

    pub fn descriptor(&self, kind: ComponentKind) -> Option<&ComponentTypeDescriptor> {
        self.descriptors.get(&kind)
    }

    pub fn descriptor_by_type_name(&self, type_name: &str) -> Option<&ComponentTypeDescriptor> {
        self.descriptors
            .values()
            .find(|descriptor| descriptor.type_name.eq_ignore_ascii_case(type_name))
    }

    pub fn has_capability(&self, kind: ComponentKind, capability: ComponentCapability) -> bool {
        self.descriptor(kind)
            .map(|descriptor| descriptor.has(capability))
            .unwrap_or(false)
    }
}

pub fn camera_2d_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor {
        kind: ComponentKind::Camera2D,
        type_name: "Camera2D",
        label: "Camera 2D",
        domains: &[ComponentDomain::Camera],
        capabilities: &[
            ComponentCapability::Transformable2D,
            ComponentCapability::Selectable,
            ComponentCapability::HasEditorControl,
        ],
        metadata_traits: &[
            MetadataTraitKind::Camera,
            MetadataTraitKind::RenderableViewportSource,
            MetadataTraitKind::UsesTransform2D,
            MetadataTraitKind::Selectable,
            MetadataTraitKind::HasEditorControls,
            MetadataTraitKind::GenericEditable,
        ],
        asset_refs: &[],
        properties: &[],
        transform_policy: TransformPolicy::UsesEntityTransform2,
        bounds_policy: BoundsPolicy::CameraViewport2D,
        editor_controls: &[EditorControlKind::Camera2D, EditorControlKind::Transform2D],
        patch_ops: &[
            EditorPatchOpKind::SetTransform2,
            EditorPatchOpKind::SetCamera2D,
        ],
    }
}

pub fn text_2d_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor {
        kind: ComponentKind::Text2D,
        type_name: "Text2D",
        label: "Text 2D",
        domains: &[ComponentDomain::Render2D],
        capabilities: &[
            ComponentCapability::Renderable2D,
            ComponentCapability::Transformable2D,
            ComponentCapability::Selectable,
            ComponentCapability::HasBounds2D,
            ComponentCapability::HasAssetRefs,
            ComponentCapability::HasEditorControl,
        ],
        metadata_traits: &[
            MetadataTraitKind::Renderable2D,
            MetadataTraitKind::UsesTransform2D,
            MetadataTraitKind::Selectable,
            MetadataTraitKind::HasBounds2D,
            MetadataTraitKind::HasAssetRefs,
            MetadataTraitKind::HasEditorControls,
            MetadataTraitKind::GenericEditable,
        ],
        asset_refs: &[ComponentAssetRefDescriptor {
            field_path: "font",
            domain: AssetDomain::Font,
            required: true,
            trait_kind: MetadataTraitKind::HasAssetRefs,
            group: "assetRefs.primary",
        }],
        properties: &[
            EditorPropertyDescriptor {
                path: "content",
                label: "Content",
                value_kind: EditorPropertyValueKind::String,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Text,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::Renderable2D),
                group: "render2d.content",
                patch_op: Some(EditorPatchOpKind::SetTextContent),
            },
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
            },
            EditorPropertyDescriptor {
                path: "bounds",
                label: "Bounds",
                value_kind: EditorPropertyValueKind::Vec2,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Vec2,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::HasBounds2D),
                group: "bounds2.size",
                patch_op: Some(EditorPatchOpKind::SetTextBounds),
            },
        ],
        transform_policy: TransformPolicy::UsesEntityTransform2,
        bounds_policy: BoundsPolicy::ComponentBounds2D { field: "bounds" },
        editor_controls: &[
            EditorControlKind::Transform2D,
            EditorControlKind::TextBounds2D,
        ],
        patch_ops: &[
            EditorPatchOpKind::SetTransform2,
            EditorPatchOpKind::SetTextContent,
            EditorPatchOpKind::SetTextBounds,
        ],
    }
}

pub fn vector_shape_2d_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor {
        kind: ComponentKind::VectorShape2D,
        type_name: "VectorShape2D",
        label: "Vector Shape 2D",
        domains: &[ComponentDomain::Render2D],
        capabilities: &[
            ComponentCapability::Renderable2D,
            ComponentCapability::Transformable2D,
            ComponentCapability::Selectable,
            ComponentCapability::HasBounds2D,
            ComponentCapability::HasEditorControl,
        ],
        metadata_traits: &[
            MetadataTraitKind::Renderable2D,
            MetadataTraitKind::UsesTransform2D,
            MetadataTraitKind::Selectable,
            MetadataTraitKind::HasBounds2D,
            MetadataTraitKind::HasEditorControls,
            MetadataTraitKind::GenericEditable,
        ],
        asset_refs: &[],
        properties: &[
            EditorPropertyDescriptor {
                path: "kind",
                label: "Shape Kind",
                value_kind: EditorPropertyValueKind::Enum,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::EnumSelect,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::Renderable2D),
                group: "render2d.content",
                patch_op: None,
            },
            EditorPropertyDescriptor {
                path: "points",
                label: "Points",
                value_kind: EditorPropertyValueKind::String,
                access: EditorPropertyAccess::ReadOnly,
                editor: EditorPropertyEditorKind::ReadOnly,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::Renderable2D),
                group: "render2d.content",
                patch_op: Some(EditorPatchOpKind::SetVectorPoints),
            },
            EditorPropertyDescriptor {
                path: "radius",
                label: "Radius",
                value_kind: EditorPropertyValueKind::Number,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Number,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::HasBounds2D),
                group: "bounds2.radius",
                patch_op: None,
            },
            EditorPropertyDescriptor {
                path: "fill_color",
                label: "Fill Color",
                value_kind: EditorPropertyValueKind::Color,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Color,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::Renderable2D),
                group: "render2d.color",
                patch_op: None,
            },
            EditorPropertyDescriptor {
                path: "stroke_color",
                label: "Stroke Color",
                value_kind: EditorPropertyValueKind::Color,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Color,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::Renderable2D),
                group: "render2d.color",
                patch_op: None,
            },
            EditorPropertyDescriptor {
                path: "stroke_width",
                label: "Stroke Width",
                value_kind: EditorPropertyValueKind::Number,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Number,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::Renderable2D),
                group: "render2d.color",
                patch_op: None,
            },
            EditorPropertyDescriptor {
                path: "z_index",
                label: "Z Index",
                value_kind: EditorPropertyValueKind::Number,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Number,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::Renderable2D),
                group: "render2d.order",
                patch_op: None,
            },
        ],
        transform_policy: TransformPolicy::UsesEntityTransform2,
        bounds_policy: BoundsPolicy::DerivedFromGeometry2D,
        editor_controls: &[
            EditorControlKind::Transform2D,
            EditorControlKind::VectorVertex2D,
        ],
        patch_ops: &[
            EditorPatchOpKind::SetTransform2,
            EditorPatchOpKind::SetVectorPoints,
        ],
    }
}

pub fn sprite_2d_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor {
        kind: ComponentKind::Sprite2D,
        type_name: "Sprite2D",
        label: "Sprite 2D",
        domains: &[ComponentDomain::Render2D],
        capabilities: &[
            ComponentCapability::Renderable2D,
            ComponentCapability::Transformable2D,
            ComponentCapability::Selectable,
            ComponentCapability::HasBounds2D,
            ComponentCapability::HasAssetRefs,
            ComponentCapability::HasEditorControl,
        ],
        metadata_traits: &[
            MetadataTraitKind::Renderable2D,
            MetadataTraitKind::UsesTransform2D,
            MetadataTraitKind::Selectable,
            MetadataTraitKind::HasBounds2D,
            MetadataTraitKind::HasAssetRefs,
            MetadataTraitKind::HasEditorControls,
            MetadataTraitKind::GenericEditable,
        ],
        asset_refs: &[ComponentAssetRefDescriptor {
            field_path: "texture",
            domain: AssetDomain::Image,
            required: true,
            trait_kind: MetadataTraitKind::HasAssetRefs,
            group: "assetRefs.primary",
        }],
        properties: &[
            EditorPropertyDescriptor {
                path: "texture",
                label: "Texture",
                value_kind: EditorPropertyValueKind::AssetRef,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::AssetPicker,
                asset_domain: Some(AssetDomain::Image),
                trait_kind: Some(MetadataTraitKind::HasAssetRefs),
                group: "assetRefs.primary",
                patch_op: None,
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
            },
            EditorPropertyDescriptor {
                path: "z_index",
                label: "Z Index",
                value_kind: EditorPropertyValueKind::Number,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Number,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::Renderable2D),
                group: "render2d.order",
                patch_op: None,
            },
            EditorPropertyDescriptor {
                path: "animation",
                label: "Animation",
                value_kind: EditorPropertyValueKind::String,
                access: EditorPropertyAccess::ReadOnly,
                editor: EditorPropertyEditorKind::ReadOnly,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::Renderable2D),
                group: "render2d.content",
                patch_op: None,
            },
            EditorPropertyDescriptor {
                path: "sheet",
                label: "Sheet",
                value_kind: EditorPropertyValueKind::String,
                access: EditorPropertyAccess::ReadOnly,
                editor: EditorPropertyEditorKind::ReadOnly,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::Renderable2D),
                group: "render2d.content",
                patch_op: None,
            },
        ],
        transform_policy: TransformPolicy::UsesEntityTransform2,
        bounds_policy: BoundsPolicy::ComponentBounds2D { field: "size" },
        editor_controls: &[EditorControlKind::Transform2D, EditorControlKind::Rect2D],
        patch_ops: &[EditorPatchOpKind::SetTransform2],
    }
}

pub fn tile_map_2d_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor {
        kind: ComponentKind::TileMap2D,
        type_name: "TileMap2D",
        label: "Tile Map 2D",
        domains: &[ComponentDomain::Render2D],
        capabilities: &[
            ComponentCapability::Renderable2D,
            ComponentCapability::Transformable2D,
            ComponentCapability::Selectable,
            ComponentCapability::HasBounds2D,
            ComponentCapability::HasAssetRefs,
            ComponentCapability::HasEditorControl,
        ],
        metadata_traits: &[
            MetadataTraitKind::Renderable2D,
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

pub fn trigger_2d_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor {
        kind: ComponentKind::Trigger2D,
        type_name: "Trigger2D",
        label: "Trigger 2D",
        domains: &[ComponentDomain::Physics2D],
        capabilities: &[
            ComponentCapability::Transformable2D,
            ComponentCapability::Selectable,
            ComponentCapability::HasBounds2D,
            ComponentCapability::Trigger2D,
            ComponentCapability::HasEditorControl,
        ],
        metadata_traits: &[
            MetadataTraitKind::Collidable2D,
            MetadataTraitKind::Trigger2D,
            MetadataTraitKind::EventSource,
            MetadataTraitKind::UsesTransform2D,
            MetadataTraitKind::Selectable,
            MetadataTraitKind::HasBounds2D,
            MetadataTraitKind::HasEditorControls,
            MetadataTraitKind::GenericEditable,
        ],
        asset_refs: &[],
        properties: &[],
        transform_policy: TransformPolicy::UsesEntityTransform2,
        bounds_policy: BoundsPolicy::DerivedFromCollider2D,
        editor_controls: &[EditorControlKind::Transform2D, EditorControlKind::Trigger2D],
        patch_ops: &[
            EditorPatchOpKind::SetTransform2,
            EditorPatchOpKind::SetColliderShape,
        ],
    }
}

pub fn script_component_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor {
        kind: ComponentKind::ScriptComponent,
        type_name: "ScriptComponent",
        label: "Script Component",
        domains: &[ComponentDomain::Scripting],
        capabilities: &[
            ComponentCapability::Scriptable,
            ComponentCapability::HasAssetRefs,
        ],
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
        properties: &[],
        transform_policy: TransformPolicy::None,
        bounds_policy: BoundsPolicy::None,
        editor_controls: &[EditorControlKind::InspectorOnly],
        patch_ops: &[],
    }
}

fn generic_component_descriptor(
    kind: ComponentKind,
    type_name: &'static str,
    label: &'static str,
    domains: &'static [ComponentDomain],
    capabilities: &'static [ComponentCapability],
    metadata_traits: &'static [MetadataTraitKind],
    properties: &'static [EditorPropertyDescriptor],
    asset_refs: &'static [ComponentAssetRefDescriptor],
    transform_policy: TransformPolicy,
    bounds_policy: BoundsPolicy,
    editor_controls: &'static [EditorControlKind],
    patch_ops: &'static [EditorPatchOpKind],
) -> ComponentTypeDescriptor {
    ComponentTypeDescriptor {
        kind,
        type_name,
        label,
        domains,
        capabilities,
        metadata_traits,
        asset_refs,
        properties,
        transform_policy,
        bounds_policy,
        editor_controls,
        patch_ops,
    }
}

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
        }
    };
}

pub fn aabb_collider_2d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        ComponentKind::AabbCollider2D,
        "AabbCollider2D",
        "AABB Collider 2D",
        &[ComponentDomain::Physics2D],
        &[
            ComponentCapability::Collidable2D,
            ComponentCapability::Selectable,
            ComponentCapability::HasBounds2D,
            ComponentCapability::HasEditorControl,
        ],
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
        ComponentKind::CircleCollider2D,
        "CircleCollider2D",
        "Circle Collider 2D",
        &[ComponentDomain::Physics2D],
        &[
            ComponentCapability::Collidable2D,
            ComponentCapability::Selectable,
            ComponentCapability::HasBounds2D,
            ComponentCapability::HasEditorControl,
        ],
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
    generic_component_descriptor(
        ComponentKind::InputActionMap,
        "InputActionMap",
        "Input Action Map",
        &[ComponentDomain::Data],
        &[],
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
            p!(ro "actions", "Actions", MetadataTraitKind::InputBindable, "input.actions"),
        ],
        &[],
        TransformPolicy::None,
        BoundsPolicy::None,
        &[EditorControlKind::InspectorOnly],
        &[],
    )
}

fn generic_data_descriptor(
    kind: ComponentKind,
    type_name: &'static str,
    label: &'static str,
    traits: &'static [MetadataTraitKind],
    properties: &'static [EditorPropertyDescriptor],
) -> ComponentTypeDescriptor {
    generic_component_descriptor(
        kind,
        type_name,
        label,
        &[ComponentDomain::Data],
        &[],
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
    generic_data_descriptor(
        ComponentKind::Behavior,
        "Behavior",
        "Behavior",
        &[
            MetadataTraitKind::Scriptable,
            MetadataTraitKind::EventSource,
            MetadataTraitKind::EventListener,
            MetadataTraitKind::InputBindable,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "kind",
                "Kind",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Scriptable,
                "script.primary"
            ),
            p!(
                "action",
                "Action",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Scriptable,
                "script.primary"
            ),
        ],
    )
}
pub fn event_pipeline_descriptor() -> ComponentTypeDescriptor {
    generic_data_descriptor(
        ComponentKind::EventPipeline,
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
                "script.primary"
            ),
            p!(
                "topic",
                "Topic",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::EventListener,
                "script.primary"
            ),
            p!(ro "steps", "Steps", MetadataTraitKind::SceneTransitionSource, "script.primary"),
        ],
    )
}
pub fn ui_document_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        ComponentKind::UiDocument,
        "UiDocument",
        "UI Document",
        &[ComponentDomain::UI],
        &[ComponentCapability::UiEditable],
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
        ComponentKind::UiModelBindings,
        "UiModelBindings",
        "UI Model Bindings",
        &[ComponentDomain::UI],
        &[ComponentCapability::UiEditable],
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
        ComponentKind::UiThemeSet,
        "UiThemeSet",
        "UI Theme Set",
        &[ComponentDomain::UI],
        &[
            ComponentCapability::UiEditable,
            ComponentCapability::HasAssetRefs,
        ],
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
        ComponentKind::Velocity2D,
        "Velocity2D",
        "Velocity 2D",
        &[ComponentDomain::Motion2D],
        &[ComponentCapability::Simulatable],
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
        ComponentKind::Lifetime,
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
                "generic.properties"
            ),
            p!(
                "outcome",
                "Outcome",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::LifetimeLimited,
                "generic.properties"
            ),
            p!(
                "pool",
                "Pool",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Poolable,
                "generic.properties"
            ),
        ],
    )
}
pub fn particle_emitter_2d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        ComponentKind::ParticleEmitter2D,
        "ParticleEmitter2D",
        "Particle Emitter 2D",
        &[ComponentDomain::Particles, ComponentDomain::Render2D],
        &[
            ComponentCapability::Renderable2D,
            ComponentCapability::HasBounds2D,
            ComponentCapability::HasEditorControl,
        ],
        &[
            MetadataTraitKind::Renderable2D,
            MetadataTraitKind::UsesTransform2D,
            MetadataTraitKind::HasBounds2D,
            MetadataTraitKind::HasEditorControls,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(
                "active",
                "Active",
                EditorPropertyValueKind::Bool,
                EditorPropertyEditorKind::Checkbox,
                MetadataTraitKind::Renderable2D,
                "render2d.content"
            ),
            p!(
                "spawn_rate",
                "Spawn Rate",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Renderable2D,
                "render2d.content"
            ),
            p!(
                "max_particles",
                "Max Particles",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Renderable2D,
                "render2d.content"
            ),
        ],
        &[],
        TransformPolicy::UsesEntityTransform2,
        BoundsPolicy::DerivedFromGeometry2D,
        &[EditorControlKind::Transform2D, EditorControlKind::Rect2D],
        &[EditorPatchOpKind::SetTransform2],
    )
}
pub fn tile_map_marker_2d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        ComponentKind::TileMapMarker2D,
        "TileMapMarker2D",
        "Tile Map Marker 2D",
        &[ComponentDomain::EditorOnly],
        &[
            ComponentCapability::Selectable,
            ComponentCapability::HasBounds2D,
        ],
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
                "generic.properties"
            ),
            p!(
                "symbol",
                "Symbol",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::GenericEditable,
                "generic.properties"
            ),
            p!(
                "index",
                "Index",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::GenericEditable,
                "generic.properties"
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
        ComponentKind::CameraFollow2D,
        "CameraFollow2D",
        "Camera Follow 2D",
        &[ComponentDomain::Camera],
        &[],
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
        ComponentKind::Parallax2D,
        "Parallax2D",
        "Parallax 2D",
        &[ComponentDomain::Render2D, ComponentDomain::Camera],
        &[],
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
        ComponentKind::FreeflightMotion2D,
        "FreeflightMotion2D",
        "Freeflight Motion 2D",
        &[ComponentDomain::Motion2D],
        &[ComponentCapability::Simulatable],
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
        ComponentKind::KinematicBody2D,
        "KinematicBody2D",
        "Kinematic Body 2D",
        &[ComponentDomain::Motion2D, ComponentDomain::Physics2D],
        &[ComponentCapability::Simulatable],
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
        ComponentKind::MotionController2D,
        "MotionController2D",
        "Motion Controller 2D",
        &[ComponentDomain::Motion2D],
        &[ComponentCapability::Simulatable],
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
                "jump_velocity",
                "Jump Velocity",
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
    generic_data_descriptor(
        ComponentKind::ProjectileEmitter2D,
        "ProjectileEmitter2D",
        "Projectile Emitter 2D",
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
                "generic.properties"
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
        ],
    )
}
pub fn entity_pool_descriptor() -> ComponentTypeDescriptor {
    generic_data_descriptor(
        ComponentKind::EntityPool,
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
                "generic.properties"
            ),
            p!(ro "members", "Members", MetadataTraitKind::Poolable, "generic.properties"),
        ],
    )
}
pub fn bounds_2d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        ComponentKind::Bounds2D,
        "Bounds2D",
        "Bounds 2D",
        &[ComponentDomain::EditorOnly],
        &[ComponentCapability::HasBounds2D],
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
pub fn default_component_registry() -> ComponentRegistry {
    ComponentRegistry::new([
        camera_2d_descriptor(),
        text_2d_descriptor(),
        vector_shape_2d_descriptor(),
        sprite_2d_descriptor(),
        tile_map_2d_descriptor(),
        trigger_2d_descriptor(),
        script_component_descriptor(),
    ])
}
