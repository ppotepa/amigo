use std::collections::BTreeMap;

use amigo_core::{AmigoError, AmigoResult};
use serde::{Deserialize, Serialize};

use crate::MetadataTraitKind;

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
        default_width: u32,
        default_height: u32,
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
    pub kind_id: &'static str,
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

impl ComponentTypeDescriptor {
    pub fn component_kind_id(&self) -> ComponentKindId {
        ComponentKindId::new(self.kind_id)
    }

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
                .map(|descriptor| (descriptor.component_kind_id(), descriptor))
                .collect(),
        }
    }

    pub fn descriptor(&self, kind_id: &str) -> Option<&ComponentTypeDescriptor> {
        self.descriptors.get(&ComponentKindId::new(kind_id))
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
            .find(|descriptor| component_type_name_matches(descriptor.type_name, type_name))
    }

    pub fn insert(
        &mut self,
        descriptor: ComponentTypeDescriptor,
    ) -> Option<ComponentTypeDescriptor> {
        self.descriptors
            .insert(descriptor.component_kind_id(), descriptor)
    }

    pub fn try_insert(&mut self, descriptor: ComponentTypeDescriptor) -> AmigoResult<()> {
        let kind_id = descriptor.component_kind_id();
        if self.descriptors.contains_key(&kind_id) {
            return Err(AmigoError::Message(format!(
                "duplicate component metadata provider for `{}`",
                kind_id.as_str()
            )));
        }

        self.descriptors.insert(kind_id, descriptor);
        Ok(())
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

fn component_type_name_matches(descriptor_type_name: &str, requested_type_name: &str) -> bool {
    descriptor_type_name.eq_ignore_ascii_case(requested_type_name)
        || requested_type_name
            .rsplit('.')
            .next()
            .is_some_and(|short_name| descriptor_type_name.eq_ignore_ascii_case(short_name))
}
