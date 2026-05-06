use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformPolicy {
    None,
    UsesEntityTransform2,
    UsesEntityTransform3,
    ComponentLocal2D,
    ComponentLocal3D,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundsPolicy {
    None,
    EntityTransformPoint,
    ComponentBounds2D { field: &'static str },
    DerivedFromGeometry2D,
    DerivedFromTileMap,
    DerivedFromCollider2D,
    CameraViewport2D,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentAssetRefDescriptor {
    pub field_path: &'static str,
    pub domain: AssetDomain,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct ComponentTypeDescriptor {
    pub kind: ComponentKind,
    pub type_name: &'static str,
    pub label: &'static str,
    pub domains: &'static [ComponentDomain],
    pub capabilities: &'static [ComponentCapability],
    pub asset_refs: &'static [ComponentAssetRefDescriptor],
    pub transform_policy: TransformPolicy,
    pub bounds_policy: BoundsPolicy,
    pub editor_controls: &'static [EditorControlKind],
    pub patch_ops: &'static [EditorPatchOpKind],
}

impl ComponentTypeDescriptor {
    pub fn has(&self, capability: ComponentCapability) -> bool {
        self.capabilities.contains(&capability)
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
        asset_refs: &[],
        transform_policy: TransformPolicy::UsesEntityTransform2,
        bounds_policy: BoundsPolicy::CameraViewport2D,
        editor_controls: &[EditorControlKind::Camera2D, EditorControlKind::Transform2D],
        patch_ops: &[EditorPatchOpKind::SetTransform2, EditorPatchOpKind::SetCamera2D],
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
        asset_refs: &[ComponentAssetRefDescriptor {
            field_path: "font",
            domain: AssetDomain::Font,
            required: true,
        }],
        transform_policy: TransformPolicy::UsesEntityTransform2,
        bounds_policy: BoundsPolicy::ComponentBounds2D { field: "bounds" },
        editor_controls: &[EditorControlKind::Transform2D, EditorControlKind::TextBounds2D],
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
        asset_refs: &[],
        transform_policy: TransformPolicy::UsesEntityTransform2,
        bounds_policy: BoundsPolicy::DerivedFromGeometry2D,
        editor_controls: &[EditorControlKind::Transform2D, EditorControlKind::VectorVertex2D],
        patch_ops: &[EditorPatchOpKind::SetTransform2, EditorPatchOpKind::SetVectorPoints],
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
        asset_refs: &[ComponentAssetRefDescriptor {
            field_path: "texture",
            domain: AssetDomain::Image,
            required: true,
        }],
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
        asset_refs: &[
            ComponentAssetRefDescriptor {
                field_path: "tileset",
                domain: AssetDomain::TileSet,
                required: true,
            },
            ComponentAssetRefDescriptor {
                field_path: "ruleset",
                domain: AssetDomain::TileRuleSet,
                required: false,
            },
        ],
        transform_policy: TransformPolicy::UsesEntityTransform2,
        bounds_policy: BoundsPolicy::DerivedFromTileMap,
        editor_controls: &[EditorControlKind::Transform2D, EditorControlKind::TileMapBrush2D],
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
        asset_refs: &[],
        transform_policy: TransformPolicy::UsesEntityTransform2,
        bounds_policy: BoundsPolicy::DerivedFromCollider2D,
        editor_controls: &[EditorControlKind::Transform2D, EditorControlKind::Trigger2D],
        patch_ops: &[EditorPatchOpKind::SetTransform2, EditorPatchOpKind::SetColliderShape],
    }
}

pub fn script_component_descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor {
        kind: ComponentKind::ScriptComponent,
        type_name: "ScriptComponent",
        label: "Script Component",
        domains: &[ComponentDomain::Scripting],
        capabilities: &[ComponentCapability::Scriptable, ComponentCapability::HasAssetRefs],
        asset_refs: &[ComponentAssetRefDescriptor {
            field_path: "script",
            domain: AssetDomain::Script,
            required: true,
        }],
        transform_policy: TransformPolicy::None,
        bounds_policy: BoundsPolicy::None,
        editor_controls: &[EditorControlKind::InspectorOnly],
        patch_ops: &[],
    }
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
