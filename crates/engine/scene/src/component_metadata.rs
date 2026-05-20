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
    LayeredImage2D,
    DepthMap2D,
    DepthAuxMap2D,
    GlobalLight2D,
    LightMap2DSource,
    TileMap2D,
    Text2D,
    VectorShape2D,
    BeaconLight2D,
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ComponentKindId(String);

impl ComponentKindId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<ComponentKind> for ComponentKindId {
    fn from(kind: ComponentKind) -> Self {
        Self::new(kind.type_name())
    }
}

impl ComponentKind {
    pub fn type_name(self) -> &'static str {
        match self {
            ComponentKind::Camera2D => "Camera2D",
            ComponentKind::Camera3D => "Camera3D",
            ComponentKind::Light3D => "Light3D",
            ComponentKind::Sprite2D => "Sprite2D",
            ComponentKind::LayeredImage2D => "LayeredImage2D",
            ComponentKind::DepthMap2D => "DepthMap2D",
            ComponentKind::DepthAuxMap2D => "DepthAuxMap2D",
            ComponentKind::GlobalLight2D => "GlobalLight2D",
            ComponentKind::LightMap2DSource => "LightMap2DSource",
            ComponentKind::TileMap2D => "TileMap2D",
            ComponentKind::Text2D => "Text2D",
            ComponentKind::VectorShape2D => "VectorShape2D",
            ComponentKind::BeaconLight2D => "BeaconLight2D",
            ComponentKind::EntityPool => "EntityPool",
            ComponentKind::Lifetime => "Lifetime",
            ComponentKind::ProjectileEmitter2D => "ProjectileEmitter2D",
            ComponentKind::InputActionMap => "InputActionMap",
            ComponentKind::Behavior => "Behavior",
            ComponentKind::EventPipeline => "EventPipeline",
            ComponentKind::UiModelBindings => "UiModelBindings",
            ComponentKind::ScriptComponent => "ScriptComponent",
            ComponentKind::ParticleEmitter2D => "ParticleEmitter2D",
            ComponentKind::Velocity2D => "Velocity2D",
            ComponentKind::Bounds2D => "Bounds2D",
            ComponentKind::FreeflightMotion2D => "FreeflightMotion2D",
            ComponentKind::KinematicBody2D => "KinematicBody2D",
            ComponentKind::AabbCollider2D => "AabbCollider2D",
            ComponentKind::StaticCollider2D => "StaticCollider2D",
            ComponentKind::CircleCollider2D => "CircleCollider2D",
            ComponentKind::Trigger2D => "Trigger2D",
            ComponentKind::MotionController2D => "MotionController2D",
            ComponentKind::CameraFollow2D => "CameraFollow2D",
            ComponentKind::Parallax2D => "Parallax2D",
            ComponentKind::TileMapMarker2D => "TileMapMarker2D",
            ComponentKind::Mesh3D => "Mesh3D",
            ComponentKind::Material3D => "Material3D",
            ComponentKind::Text3D => "Text3D",
            ComponentKind::UiDocument => "UiDocument",
            ComponentKind::UiThemeSet => "UiThemeSet",
        }
    }

    pub fn all() -> &'static [ComponentKind] {
        &[
            ComponentKind::Camera2D,
            ComponentKind::Camera3D,
            ComponentKind::Light3D,
            ComponentKind::Sprite2D,
            ComponentKind::LayeredImage2D,
            ComponentKind::DepthMap2D,
            ComponentKind::DepthAuxMap2D,
            ComponentKind::GlobalLight2D,
            ComponentKind::LightMap2DSource,
            ComponentKind::TileMap2D,
            ComponentKind::Text2D,
            ComponentKind::VectorShape2D,
            ComponentKind::BeaconLight2D,
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
pub enum AssetDomain {
    Image,
    Sprite,
    Spritesheet,
    LayeredImage,
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
    ComponentBounds2D {
        field: &'static str,
    },
    SpawnArea2D {
        field: &'static str,
        size_field: &'static str,
        fallback_width: u32,
        fallback_height: u32,
    },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EditorPropertyVisibility {
    Primary,
    Advanced,
    Debug,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EditorRuntimeBindingTemplate {
    None,
    RenderLayerField,
    ComponentRuntimeField,
    LayeredImageBaseOpacity,
    LayeredImagePartField,
    ParticleEmitterField,
    SceneCommandPatch,
    MockOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorNumberConstraints {
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub step: Option<f32>,
    pub clamp: bool,
    pub unit: Option<&'static str>,
    pub display_scale: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorPropertyOption {
    pub id: &'static str,
    pub label: &'static str,
}

pub const EDITOR_NUMBER_OPACITY: EditorNumberConstraints = EditorNumberConstraints {
    min: Some(0.0),
    max: Some(1.0),
    step: Some(0.01),
    clamp: true,
    unit: None,
    display_scale: 1.0,
};

pub const EDITOR_NUMBER_Z_DEPTH: EditorNumberConstraints = EditorNumberConstraints {
    min: Some(0.0),
    max: Some(1.0),
    step: Some(0.01),
    clamp: true,
    unit: None,
    display_scale: 1.0,
};

pub const EDITOR_NUMBER_ORDER: EditorNumberConstraints = EditorNumberConstraints {
    min: Some(-1000.0),
    max: Some(1000.0),
    step: Some(1.0),
    clamp: true,
    unit: None,
    display_scale: 1.0,
};

pub const EDITOR_NUMBER_PARTICLE_RATE: EditorNumberConstraints = EditorNumberConstraints {
    min: Some(0.0),
    max: Some(1000.0),
    step: Some(1.0),
    clamp: true,
    unit: None,
    display_scale: 1.0,
};

pub const EDITOR_NUMBER_PARTICLE_COUNT: EditorNumberConstraints = EditorNumberConstraints {
    min: Some(0.0),
    max: Some(10000.0),
    step: Some(1.0),
    clamp: true,
    unit: None,
    display_scale: 1.0,
};

pub const EDITOR_NUMBER_PARTICLE_SECONDS: EditorNumberConstraints = EditorNumberConstraints {
    min: Some(0.0),
    max: Some(60.0),
    step: Some(0.01),
    clamp: true,
    unit: Some("s"),
    display_scale: 1.0,
};

pub const EDITOR_NUMBER_PARTICLE_SPEED: EditorNumberConstraints = EditorNumberConstraints {
    min: Some(0.0),
    max: Some(2000.0),
    step: Some(1.0),
    clamp: true,
    unit: None,
    display_scale: 1.0,
};

pub const EDITOR_NUMBER_PARTICLE_DEGREES: EditorNumberConstraints = EditorNumberConstraints {
    min: Some(-360.0),
    max: Some(360.0),
    step: Some(1.0),
    clamp: true,
    unit: Some("deg"),
    display_scale: 1.0,
};

pub const EDITOR_NUMBER_PARTICLE_SIZE: EditorNumberConstraints = EditorNumberConstraints {
    min: Some(0.0),
    max: Some(512.0),
    step: Some(0.1),
    clamp: true,
    unit: Some("px"),
    display_scale: 1.0,
};

pub const EDITOR_NUMBER_PARTICLE_VELOCITY_SCALE: EditorNumberConstraints =
    EditorNumberConstraints {
        min: Some(0.0),
        max: Some(1.0),
        step: Some(0.01),
        clamp: true,
        unit: None,
        display_scale: 1.0,
    };

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
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
    pub number_constraints: Option<EditorNumberConstraints>,
    pub options: &'static [EditorPropertyOption],
    pub visibility: EditorPropertyVisibility,
    pub order: i32,
    pub tags: &'static [&'static str],
    pub readonly_reason: Option<&'static str>,
    pub binding_template: Option<EditorRuntimeBindingTemplate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ComponentOwnerScope {
    Scene,
    Entity,
    UiNode,
    Asset,
}

pub const ENTITY_OWNER_SCOPES: &[ComponentOwnerScope] = &[ComponentOwnerScope::Entity];
pub const SCENE_OWNER_SCOPES: &[ComponentOwnerScope] = &[ComponentOwnerScope::Scene];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentTypeDescriptor {
    pub kind: ComponentKind,
    pub type_name: &'static str,
    pub label: &'static str,
    pub domains: &'static [ComponentDomain],
    pub owner_scopes: &'static [ComponentOwnerScope],
    pub default_yaml: Option<&'static str>,
    pub metadata_traits: &'static [MetadataTraitKind],
    pub asset_refs: &'static [ComponentAssetRefDescriptor],
    pub properties: &'static [EditorPropertyDescriptor],
    pub transform_policy: TransformPolicy,
    pub bounds_policy: BoundsPolicy,
    pub editor_controls: &'static [EditorControlKind],
    pub patch_ops: &'static [EditorPatchOpKind],
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

impl ComponentTypeDescriptor {
    pub fn has_trait(&self, trait_kind: MetadataTraitKind) -> bool {
        self.metadata_traits.contains(&trait_kind)
    }

    pub fn default_yaml(mut self, yaml: &'static str) -> Self {
        self.default_yaml = Some(yaml);
        self
    }
}

#[derive(Debug, Clone)]
pub struct ComponentRegistry {
    descriptors: BTreeMap<ComponentKindId, ComponentTypeDescriptor>,
}

impl ComponentRegistry {
    pub fn new(descriptors: impl IntoIterator<Item = ComponentTypeDescriptor>) -> Self {
        Self {
            descriptors: descriptors
                .into_iter()
                .map(|descriptor| (ComponentKindId::from(descriptor.kind), descriptor))
                .collect(),
        }
    }

    pub fn descriptor(&self, kind: ComponentKind) -> Option<&ComponentTypeDescriptor> {
        self.descriptors.get(&ComponentKindId::from(kind))
    }

    pub fn descriptor_by_kind_id(
        &self,
        kind_id: &ComponentKindId,
    ) -> Option<&ComponentTypeDescriptor> {
        self.descriptors.get(kind_id)
    }

    pub fn descriptor_by_type_name(&self, type_name: &str) -> Option<&ComponentTypeDescriptor> {
        self.descriptors
            .values()
            .find(|descriptor| descriptor.type_name.eq_ignore_ascii_case(type_name))
    }

    pub fn insert(
        &mut self,
        descriptor: ComponentTypeDescriptor,
    ) -> Option<ComponentTypeDescriptor> {
        self.descriptors
            .insert(ComponentKindId::from(descriptor.kind), descriptor)
    }

    pub fn extend(&mut self, descriptors: impl IntoIterator<Item = ComponentTypeDescriptor>) {
        for descriptor in descriptors {
            self.insert(descriptor);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &ComponentTypeDescriptor> {
        self.descriptors.values()
    }
}

pub fn camera_2d_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor {
        kind: ComponentKind::Camera2D,
        type_name: "Camera2D",
        label: "Camera 2D",
        domains: &[ComponentDomain::Camera],
        owner_scopes: ENTITY_OWNER_SCOPES,
        default_yaml: None,
        metadata_traits: &[
            MetadataTraitKind::Camera,
            MetadataTraitKind::RenderableViewportSource,
            MetadataTraitKind::UsesTransform2D,
            MetadataTraitKind::Selectable,
            MetadataTraitKind::HasAssetRefs,
            MetadataTraitKind::HasEditorControls,
            MetadataTraitKind::GenericEditable,
        ],
        asset_refs: &[
            ComponentAssetRefDescriptor {
                field_path: "lens.profile",
                domain: AssetDomain::Raw,
                required: false,
                trait_kind: MetadataTraitKind::HasAssetRefs,
                group: "assetRefs.camera",
            },
            ComponentAssetRefDescriptor {
                field_path: "film.profile",
                domain: AssetDomain::Raw,
                required: false,
                trait_kind: MetadataTraitKind::HasAssetRefs,
                group: "assetRefs.camera",
            },
            ComponentAssetRefDescriptor {
                field_path: "look.profile",
                domain: AssetDomain::Raw,
                required: false,
                trait_kind: MetadataTraitKind::HasAssetRefs,
                group: "assetRefs.camera",
            },
            ComponentAssetRefDescriptor {
                field_path: "lens_surface.rain_profile",
                domain: AssetDomain::Raw,
                required: false,
                trait_kind: MetadataTraitKind::HasAssetRefs,
                group: "assetRefs.camera",
            },
        ],
        properties: &[
            EditorPropertyDescriptor {
                path: "exposure.iso",
                label: "Exposure ISO",
                value_kind: EditorPropertyValueKind::Number,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Number,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::RuntimeControllable),
                group: "camera.exposure",
                patch_op: None,
                number_constraints: Some(EditorNumberConstraints {
                    min: Some(25.0),
                    max: Some(12800.0),
                    step: Some(1.0),
                    clamp: true,
                    unit: Some("iso"),
                    display_scale: 1.0,
                }),
                options: &[],
                visibility: EditorPropertyVisibility::Primary,
                order: 0,
                tags: &[],
                readonly_reason: None,
                binding_template: Some(EditorRuntimeBindingTemplate::MockOnly),
            },
            EditorPropertyDescriptor {
                path: "exposure.shutter_speed",
                label: "Shutter Speed",
                value_kind: EditorPropertyValueKind::Number,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Number,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::RuntimeControllable),
                group: "camera.exposure",
                patch_op: None,
                number_constraints: Some(EditorNumberConstraints {
                    min: Some(0.0001),
                    max: Some(2.0),
                    step: Some(0.0001),
                    clamp: true,
                    unit: Some("s"),
                    display_scale: 1.0,
                }),
                options: &[],
                visibility: EditorPropertyVisibility::Primary,
                order: 1,
                tags: &[],
                readonly_reason: None,
                binding_template: Some(EditorRuntimeBindingTemplate::MockOnly),
            },
            EditorPropertyDescriptor {
                path: "aperture.focus_distance_m",
                label: "Focus Distance",
                value_kind: EditorPropertyValueKind::Number,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Number,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::RuntimeControllable),
                group: "camera.aperture",
                patch_op: None,
                number_constraints: Some(EditorNumberConstraints {
                    min: Some(0.2),
                    max: Some(1000.0),
                    step: Some(0.1),
                    clamp: true,
                    unit: Some("m"),
                    display_scale: 1.0,
                }),
                options: &[],
                visibility: EditorPropertyVisibility::Primary,
                order: 2,
                tags: &[],
                readonly_reason: None,
                binding_template: Some(EditorRuntimeBindingTemplate::MockOnly),
            },
            EditorPropertyDescriptor {
                path: "aperture.focus_depth",
                label: "Focus Depth Override",
                value_kind: EditorPropertyValueKind::Number,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Number,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::RuntimeControllable),
                group: "camera.aperture",
                patch_op: None,
                number_constraints: Some(EditorNumberConstraints {
                    min: Some(0.0),
                    max: Some(1.0),
                    step: Some(0.01),
                    clamp: true,
                    unit: None,
                    display_scale: 1.0,
                }),
                options: &[],
                visibility: EditorPropertyVisibility::Debug,
                order: 3,
                tags: &[],
                readonly_reason: Some("Low-level debug focus override"),
                binding_template: Some(EditorRuntimeBindingTemplate::MockOnly),
            },
            EditorPropertyDescriptor {
                path: "look.contrast",
                label: "Contrast",
                value_kind: EditorPropertyValueKind::Number,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Number,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::RuntimeControllable),
                group: "camera.look",
                patch_op: None,
                number_constraints: None,
                options: &[],
                visibility: EditorPropertyVisibility::Primary,
                order: 4,
                tags: &[],
                readonly_reason: None,
                binding_template: Some(EditorRuntimeBindingTemplate::MockOnly),
            },
            EditorPropertyDescriptor {
                path: "look.saturation",
                label: "Saturation",
                value_kind: EditorPropertyValueKind::Number,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Number,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::RuntimeControllable),
                group: "camera.look",
                patch_op: None,
                number_constraints: None,
                options: &[],
                visibility: EditorPropertyVisibility::Primary,
                order: 5,
                tags: &[],
                readonly_reason: None,
                binding_template: Some(EditorRuntimeBindingTemplate::MockOnly),
            },
            EditorPropertyDescriptor {
                path: "lens.profile",
                label: "Lens Profile",
                value_kind: EditorPropertyValueKind::AssetRef,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::AssetPicker,
                asset_domain: Some(AssetDomain::Raw),
                trait_kind: Some(MetadataTraitKind::HasAssetRefs),
                group: "assetRefs.camera",
                patch_op: None,
                number_constraints: None,
                options: &[],
                visibility: EditorPropertyVisibility::Advanced,
                order: 6,
                tags: &[],
                readonly_reason: None,
                binding_template: Some(EditorRuntimeBindingTemplate::MockOnly),
            },
            EditorPropertyDescriptor {
                path: "film.profile",
                label: "Film Profile",
                value_kind: EditorPropertyValueKind::AssetRef,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::AssetPicker,
                asset_domain: Some(AssetDomain::Raw),
                trait_kind: Some(MetadataTraitKind::HasAssetRefs),
                group: "assetRefs.camera",
                patch_op: None,
                number_constraints: None,
                options: &[],
                visibility: EditorPropertyVisibility::Advanced,
                order: 7,
                tags: &[],
                readonly_reason: None,
                binding_template: Some(EditorRuntimeBindingTemplate::MockOnly),
            },
            EditorPropertyDescriptor {
                path: "look.profile",
                label: "Look Profile",
                value_kind: EditorPropertyValueKind::AssetRef,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::AssetPicker,
                asset_domain: Some(AssetDomain::Raw),
                trait_kind: Some(MetadataTraitKind::HasAssetRefs),
                group: "assetRefs.camera",
                patch_op: None,
                number_constraints: None,
                options: &[],
                visibility: EditorPropertyVisibility::Advanced,
                order: 8,
                tags: &[],
                readonly_reason: None,
                binding_template: Some(EditorRuntimeBindingTemplate::MockOnly),
            },
            EditorPropertyDescriptor {
                path: "lens_surface.rain_profile",
                label: "Rain Profile",
                value_kind: EditorPropertyValueKind::AssetRef,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::AssetPicker,
                asset_domain: Some(AssetDomain::Raw),
                trait_kind: Some(MetadataTraitKind::HasAssetRefs),
                group: "assetRefs.camera",
                patch_op: None,
                number_constraints: None,
                options: &[],
                visibility: EditorPropertyVisibility::Advanced,
                order: 9,
                tags: &[],
                readonly_reason: None,
                binding_template: Some(EditorRuntimeBindingTemplate::MockOnly),
            },
            EditorPropertyDescriptor {
                path: "quality",
                label: "Quality Profile",
                value_kind: EditorPropertyValueKind::String,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Text,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::RuntimeControllable),
                group: "camera.runtime",
                patch_op: None,
                number_constraints: None,
                options: &[],
                visibility: EditorPropertyVisibility::Advanced,
                order: 10,
                tags: &[],
                readonly_reason: None,
                binding_template: Some(EditorRuntimeBindingTemplate::MockOnly),
            },
            EditorPropertyDescriptor {
                path: "debug.view",
                label: "Debug View",
                value_kind: EditorPropertyValueKind::String,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Text,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::RuntimeControllable),
                group: "camera.runtime",
                patch_op: None,
                number_constraints: None,
                options: &[],
                visibility: EditorPropertyVisibility::Debug,
                order: 11,
                tags: &[],
                readonly_reason: None,
                binding_template: Some(EditorRuntimeBindingTemplate::MockOnly),
            },
            EditorPropertyDescriptor {
                path: "lens_surface.lens_rain.enabled",
                label: "Lens Rain Enabled",
                value_kind: EditorPropertyValueKind::Bool,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Checkbox,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::RuntimeControllable),
                group: "camera.lens_surface",
                patch_op: None,
                number_constraints: None,
                options: &[],
                visibility: EditorPropertyVisibility::Primary,
                order: 12,
                tags: &[],
                readonly_reason: None,
                binding_template: Some(EditorRuntimeBindingTemplate::MockOnly),
            },
            EditorPropertyDescriptor {
                path: "lens_surface.lens_rain.spawn_rate",
                label: "Lens Rain Spawn Rate",
                value_kind: EditorPropertyValueKind::Number,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Number,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::RuntimeControllable),
                group: "camera.lens_surface",
                patch_op: None,
                number_constraints: Some(EDITOR_NUMBER_PARTICLE_RATE),
                options: &[],
                visibility: EditorPropertyVisibility::Primary,
                order: 13,
                tags: &[],
                readonly_reason: None,
                binding_template: Some(EditorRuntimeBindingTemplate::MockOnly),
            },
            EditorPropertyDescriptor {
                path: "lens_surface.lens_rain.distortion",
                label: "Lens Rain Distortion",
                value_kind: EditorPropertyValueKind::Number,
                access: EditorPropertyAccess::Editable,
                editor: EditorPropertyEditorKind::Number,
                asset_domain: None,
                trait_kind: Some(MetadataTraitKind::RuntimeControllable),
                group: "camera.lens_surface",
                patch_op: None,
                number_constraints: None,
                options: &[],
                visibility: EditorPropertyVisibility::Primary,
                order: 8,
                tags: &[],
                readonly_reason: None,
                binding_template: Some(EditorRuntimeBindingTemplate::MockOnly),
            },
        ],
        transform_policy: TransformPolicy::UsesEntityTransform2,
        bounds_policy: BoundsPolicy::CameraViewport2D,
        editor_controls: &[EditorControlKind::Camera2D, EditorControlKind::Transform2D],
        patch_ops: &[
            EditorPatchOpKind::SetTransform2,
            EditorPatchOpKind::SetCamera2D,
        ],
    }
}

pub fn global_light_2d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        ComponentKind::GlobalLight2D,
        "GlobalLight2D",
        "Global Light 2D",
        &[ComponentDomain::Render2D],
        &[
            MetadataTraitKind::Renderable2D,
            MetadataTraitKind::RuntimeControllable,
            MetadataTraitKind::GenericEditable,
        ],
        &[
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
        &[],
        TransformPolicy::None,
        BoundsPolicy::None,
        &[EditorControlKind::InspectorOnly],
        &[],
    )
}

pub fn beacon_light_2d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        ComponentKind::BeaconLight2D,
        "BeaconLight2D",
        "Beacon Light 2D",
        &[ComponentDomain::Render2D],
        &[
            MetadataTraitKind::Renderable2D,
            MetadataTraitKind::RenderLayered2D,
            MetadataTraitKind::UsesTransform2D,
            MetadataTraitKind::Selectable,
            MetadataTraitKind::HasBounds2D,
            MetadataTraitKind::RuntimeControllable,
            MetadataTraitKind::GenericEditable,
        ],
        &[
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
        &[],
        TransformPolicy::UsesEntityTransform2,
        BoundsPolicy::EntityTransformPoint,
        &[
            EditorControlKind::Transform2D,
            EditorControlKind::InspectorOnly,
        ],
        &[EditorPatchOpKind::SetTransform2],
    )
}

pub fn lightmap_2d_source_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        ComponentKind::LightMap2DSource,
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

pub fn tile_map_2d_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor {
        kind: ComponentKind::TileMap2D,
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

pub fn trigger_2d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        ComponentKind::Trigger2D,
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
        kind: ComponentKind::ScriptComponent,
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
    kind: ComponentKind,
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
        kind,
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
        ComponentKind::AabbCollider2D,
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
        ComponentKind::CircleCollider2D,
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
        ComponentKind::InputActionMap,
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
        ComponentKind::Behavior,
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
        ComponentKind::UiDocument,
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
        ComponentKind::UiModelBindings,
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
        ComponentKind::UiThemeSet,
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
        ComponentKind::Velocity2D,
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
pub fn particle_emitter_2d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        ComponentKind::ParticleEmitter2D,
        "ParticleEmitter2D",
        "Particle Emitter 2D",
        &[ComponentDomain::Particles, ComponentDomain::Render2D],
        &[
            MetadataTraitKind::Renderable2D,
            MetadataTraitKind::RenderLayered2D,
            MetadataTraitKind::LightReceiver2D,
            MetadataTraitKind::UsesTransform2D,
            MetadataTraitKind::Selectable,
            MetadataTraitKind::HasBounds2D,
            MetadataTraitKind::Motion2D,
            MetadataTraitKind::HasEditorControls,
            MetadataTraitKind::GenericEditable,
        ],
        &[
            p!(live
                "active",
                "Active",
                EditorPropertyValueKind::Bool,
                EditorPropertyEditorKind::Checkbox,
                MetadataTraitKind::Renderable2D,
                "render2d.content",
                EditorRuntimeBindingTemplate::ParticleEmitterField
            ),
            p!(live num
                "spawn_rate",
                "Spawn Rate",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Renderable2D,
                "render2d.content",
                EDITOR_NUMBER_PARTICLE_RATE,
                EditorRuntimeBindingTemplate::ParticleEmitterField
            ),
            p!(live num
                "max_particles",
                "Max Particles",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Renderable2D,
                "render2d.content",
                EDITOR_NUMBER_PARTICLE_COUNT,
                EditorRuntimeBindingTemplate::ParticleEmitterField
            ),
            p!(live num
                "particle_lifetime",
                "Particle Lifetime",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Renderable2D,
                "render2d.content",
                EDITOR_NUMBER_PARTICLE_SECONDS,
                EditorRuntimeBindingTemplate::ParticleEmitterField
            ),
            p!(live num
                "initial_speed",
                "Initial Speed",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning",
                EDITOR_NUMBER_PARTICLE_SPEED,
                EditorRuntimeBindingTemplate::ParticleEmitterField
            ),
            p!(live num
                "initial_size",
                "Initial Size",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::HasBounds2D,
                "bounds2.size",
                EDITOR_NUMBER_PARTICLE_SIZE,
                EditorRuntimeBindingTemplate::ParticleEmitterField
            ),
            p!(live num
                "final_size",
                "Final Size",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::HasBounds2D,
                "bounds2.size",
                EDITOR_NUMBER_PARTICLE_SIZE,
                EditorRuntimeBindingTemplate::ParticleEmitterField
            ),
            p!(
                "render_layer",
                "Render Layer",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::RenderLayered2D,
                "render2d.order"
            ),
            p!(live num
                "z_index",
                "Z Index",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Renderable2D,
                "render2d.order",
                EDITOR_NUMBER_ORDER,
                EditorRuntimeBindingTemplate::ParticleEmitterField
            ),
            p!(
                "color",
                "Color",
                EditorPropertyValueKind::Color,
                EditorPropertyEditorKind::Color,
                MetadataTraitKind::Renderable2D,
                "render2d.color"
            ),
            p!(ro "color_ramp", "Color Ramp", MetadataTraitKind::Renderable2D, "render2d.color"),
            p!(ro "alpha_curve", "Alpha Curve", MetadataTraitKind::Renderable2D, "render2d.color"),
            p!(ro "size_curve", "Size Curve", MetadataTraitKind::HasBounds2D, "bounds2.size"),
            p!(ro "speed_curve", "Speed Curve", MetadataTraitKind::Motion2D, "motion2.tuning"),
            p!(ro "emission_rate_curve", "Emission Rate Curve", MetadataTraitKind::Renderable2D, "render2d.content"),
            p!(ro "shape", "Shape", MetadataTraitKind::Renderable2D, "render2d.content"),
            p!(ro "spawn_area", "Spawn Area", MetadataTraitKind::HasBounds2D, "bounds2.size"),
            p!(live num
                "spread_degrees",
                "Spread Degrees",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Renderable2D,
                "render2d.content",
                EDITOR_NUMBER_PARTICLE_DEGREES,
                EditorRuntimeBindingTemplate::ParticleEmitterField
            ),
            p!(live num
                "local_direction_degrees",
                "Local Direction Degrees",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Renderable2D,
                "render2d.content",
                EDITOR_NUMBER_PARTICLE_DEGREES,
                EditorRuntimeBindingTemplate::ParticleEmitterField
            ),
            p!(live num
                "lifetime_jitter",
                "Lifetime Jitter",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Renderable2D,
                "render2d.content",
                EDITOR_NUMBER_PARTICLE_SECONDS,
                EditorRuntimeBindingTemplate::ParticleEmitterField
            ),
            p!(live num
                "speed_jitter",
                "Speed Jitter",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning",
                EDITOR_NUMBER_PARTICLE_SPEED,
                EditorRuntimeBindingTemplate::ParticleEmitterField
            ),
            p!(ro "forces", "Forces", MetadataTraitKind::Motion2D, "motion2.tuning"),
            p!(
                "material",
                "Material",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::LightReceiver2D,
                "render2d.lighting"
            ),
            p!(
                "attached_to",
                "Attached To",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Motion2D,
                "motion2.attachment"
            ),
            p!(
                "local_offset",
                "Local Offset",
                EditorPropertyValueKind::Vec2,
                EditorPropertyEditorKind::Vec2,
                MetadataTraitKind::HasBounds2D,
                "bounds2.offset"
            ),
            p!(
                "blend_mode",
                "Blend Mode",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Renderable2D,
                "render2d.content"
            ),
            p!(live num
                "inherit_parent_velocity",
                "Inherit Parent Velocity",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning",
                EDITOR_NUMBER_PARTICLE_VELOCITY_SCALE,
                EditorRuntimeBindingTemplate::ParticleEmitterField
            ),
            p!(
                "align",
                "Align",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Renderable2D,
                "render2d.content"
            ),
            p!(
                "motion_stretch",
                "Motion Stretch",
                EditorPropertyValueKind::Number,
                EditorPropertyEditorKind::Number,
                MetadataTraitKind::Motion2D,
                "motion2.tuning"
            ),
            p!(
                "simulation_space",
                "Simulation Space",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Renderable2D,
                "render2d.content"
            ),
            p!(
                "light",
                "Light",
                EditorPropertyValueKind::Bool,
                EditorPropertyEditorKind::Checkbox,
                MetadataTraitKind::Renderable2D,
                "render2d.content"
            ),
            p!(
                "line_anchor",
                "Line Anchor",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Renderable2D,
                "render2d.content"
            ),
            p!(
                "velocity_mode",
                "Velocity Mode",
                EditorPropertyValueKind::String,
                EditorPropertyEditorKind::Text,
                MetadataTraitKind::Motion2D,
                "motion2.tuning"
            ),
        ],
        &[],
        TransformPolicy::UsesEntityTransform2,
        BoundsPolicy::SpawnArea2D {
            field: "spawn_area",
            size_field: "size",
            fallback_width: 128,
            fallback_height: 128,
        },
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
        ComponentKind::CameraFollow2D,
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
        ComponentKind::Parallax2D,
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
        ComponentKind::FreeflightMotion2D,
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
        ComponentKind::KinematicBody2D,
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
        ComponentKind::MotionController2D,
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
        ComponentKind::ProjectileEmitter2D,
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
                "pool.properties"
            ),
            p!(ro "members", "Members", MetadataTraitKind::Poolable, "pool.members"),
        ],
    )
}
pub fn bounds_2d_descriptor() -> ComponentTypeDescriptor {
    generic_component_descriptor(
        ComponentKind::Bounds2D,
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
        ComponentKind::Camera3D,
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
        ComponentKind::Light3D,
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
        ComponentKind::Mesh3D,
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
        ComponentKind::Material3D,
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
        ComponentKind::Text3D,
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
        ComponentKind::StaticCollider2D,
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

pub fn depth_map_2d_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor {
        kind: ComponentKind::DepthMap2D,
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
        kind: ComponentKind::DepthAuxMap2D,
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
                readonly_reason: Some("DepthAux channels are semantic labels for renderer/light pipeline consumers."),
                binding_template: None,
            },
        ],
        transform_policy: TransformPolicy::UsesEntityTransform2,
        bounds_policy: BoundsPolicy::ComponentBounds2D { field: "size" },
        editor_controls: &[EditorControlKind::Transform2D, EditorControlKind::Rect2D],
        patch_ops: &[EditorPatchOpKind::SetTransform2],
    }
}

/// Built-in legacy registry.
///
/// New plugin-owned component descriptors must be registered through
/// ComponentMetadataProvider instead of being added here.
pub fn default_component_registry() -> ComponentRegistry {
    ComponentRegistry::new([
        camera_2d_descriptor(),
        camera_3d_descriptor(),
        light_3d_descriptor(),
        text_3d_descriptor(),
        depth_map_2d_descriptor(),
        depth_aux_map_2d_descriptor(),
        global_light_2d_descriptor(),
        beacon_light_2d_descriptor(),
        lightmap_2d_source_descriptor(),
        tile_map_2d_descriptor(),
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
        particle_emitter_2d_descriptor(),
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
}
