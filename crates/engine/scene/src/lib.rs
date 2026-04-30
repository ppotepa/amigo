mod document;
mod error;
mod hydration;
mod transition;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Mutex;

use amigo_assets::AssetKey;
use amigo_core::TypedId;
use amigo_math::{ColorRgba, Curve1d, Transform2, Transform3, Vec2, Vec3};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

pub use document::*;
pub use error::*;
pub use hydration::*;
pub use transition::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SceneEntityTag;
pub type SceneEntityId = TypedId<SceneEntityTag>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceneKey(String);

impl SceneKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EntitySelector {
    Entity(String),
    Tag(String),
    Group(String),
    Pool(String),
}

#[derive(Debug, Clone)]
pub struct SceneEntity {
    pub id: SceneEntityId,
    pub name: String,
    pub transform: Transform3,
    pub lifecycle: SceneEntityLifecycle,
    pub tags: Vec<String>,
    pub groups: Vec<String>,
    pub properties: BTreeMap<String, ScenePropertyValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneEntityLifecycle {
    pub visible: bool,
    pub simulation_enabled: bool,
    pub collision_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityPoolSceneCommand {
    pub source_mod: String,
    pub pool: String,
    pub members: Vec<String>,
}

impl EntityPoolSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        pool: impl Into<String>,
        members: Vec<String>,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            pool: pool.into(),
            members,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifetimeExpirationOutcome {
    Hide,
    Disable,
    Despawn,
    ReturnToPool { pool: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LifetimeSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub seconds: f32,
    pub outcome: LifetimeExpirationOutcome,
}

impl LifetimeSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        seconds: f32,
        outcome: LifetimeExpirationOutcome,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            seconds,
            outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectileEmitter2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub pool: String,
    pub speed: f32,
    pub spawn_offset: Vec2,
    pub inherit_velocity_scale: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParticleShape2dSceneCommand {
    Circle { segments: u32 },
    Quad,
    Line { length: f32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleEmitter2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub attached_to: Option<String>,
    pub local_offset: Vec2,
    pub local_direction_radians: f32,
    pub active: bool,
    pub spawn_rate: f32,
    pub max_particles: usize,
    pub particle_lifetime: f32,
    pub lifetime_jitter: f32,
    pub initial_speed: f32,
    pub speed_jitter: f32,
    pub spread_radians: f32,
    pub inherit_parent_velocity: f32,
    pub initial_size: f32,
    pub final_size: f32,
    pub color: ColorRgba,
    pub z_index: f32,
    pub shape: ParticleShape2dSceneCommand,
    pub emission_rate_curve: Curve1d,
    pub size_curve: Curve1d,
    pub alpha_curve: Curve1d,
    pub speed_curve: Curve1d,
}

impl ProjectileEmitter2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        pool: impl Into<String>,
        speed: f32,
        spawn_offset: Vec2,
        inherit_velocity_scale: f32,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            pool: pool.into(),
            speed,
            spawn_offset,
            inherit_velocity_scale,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Velocity2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub velocity: Vec2,
}

impl Velocity2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        velocity: Vec2,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            velocity,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundsBehavior2dSceneCommand {
    Bounce { restitution: f32 },
    Wrap,
    Hide,
    Despawn,
    Clamp,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bounds2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub min: Vec2,
    pub max: Vec2,
    pub behavior: BoundsBehavior2dSceneCommand,
}

impl Bounds2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        min: Vec2,
        max: Vec2,
        behavior: BoundsBehavior2dSceneCommand,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            min,
            max,
            behavior,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreeflightMotion2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub thrust_acceleration: f32,
    pub reverse_acceleration: f32,
    pub strafe_acceleration: f32,
    pub turn_acceleration: f32,
    pub linear_damping: f32,
    pub turn_damping: f32,
    pub max_speed: f32,
    pub max_angular_speed: f32,
    pub initial_velocity: Vec2,
    pub initial_angular_velocity: f32,
    pub thrust_response_curve: Curve1d,
    pub reverse_response_curve: Curve1d,
    pub strafe_response_curve: Curve1d,
    pub turn_response_curve: Curve1d,
}

impl FreeflightMotion2dSceneCommand {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        thrust_acceleration: f32,
        reverse_acceleration: f32,
        strafe_acceleration: f32,
        turn_acceleration: f32,
        linear_damping: f32,
        turn_damping: f32,
        max_speed: f32,
        max_angular_speed: f32,
        initial_velocity: Vec2,
        initial_angular_velocity: f32,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            thrust_acceleration,
            reverse_acceleration,
            strafe_acceleration,
            turn_acceleration,
            linear_damping,
            turn_damping,
            max_speed,
            max_angular_speed,
            initial_velocity,
            initial_angular_velocity,
            thrust_response_curve: Curve1d::Linear,
            reverse_response_curve: Curve1d::Linear,
            strafe_response_curve: Curve1d::Linear,
            turn_response_curve: Curve1d::Linear,
        }
    }

    pub fn with_response_curves(
        mut self,
        thrust: Curve1d,
        reverse: Curve1d,
        strafe: Curve1d,
        turn: Curve1d,
    ) -> Self {
        self.thrust_response_curve = thrust;
        self.reverse_response_curve = reverse;
        self.strafe_response_curve = strafe;
        self.turn_response_curve = turn;
        self
    }
}

impl Default for SceneEntityLifecycle {
    fn default() -> Self {
        Self {
            visible: true,
            simulation_enabled: true,
            collision_enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScenePropertyValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HydratedSceneSnapshot {
    pub source_mod: Option<String>,
    pub scene_id: Option<String>,
    pub relative_document_path: Option<PathBuf>,
    pub entity_names: Vec<String>,
    pub component_kinds: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpriteSheet2dSceneCommand {
    pub columns: u32,
    pub rows: u32,
    pub frame_count: u32,
    pub frame_size: Vec2,
    pub fps: f32,
    pub looping: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SpriteAnimation2dSceneOverride {
    pub fps: Option<f32>,
    pub looping: Option<bool>,
    pub start_frame: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sprite2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub texture: AssetKey,
    pub size: Vec2,
    pub sheet: Option<SpriteSheet2dSceneCommand>,
    pub animation: Option<SpriteAnimation2dSceneOverride>,
    pub z_index: f32,
    pub transform: Transform2,
}

impl Sprite2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        texture: AssetKey,
        size: Vec2,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            texture,
            size,
            sheet: None,
            animation: None,
            z_index: 0.0,
            transform: Transform2::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileMap2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub tileset: AssetKey,
    pub ruleset: Option<AssetKey>,
    pub tile_size: Vec2,
    pub grid: Vec<String>,
    pub depth_fill_rows: usize,
    pub z_index: f32,
}

impl TileMap2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        tileset: AssetKey,
        tile_size: Vec2,
        grid: Vec<String>,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            tileset,
            ruleset: None,
            tile_size,
            grid,
            depth_fill_rows: 0,
            z_index: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Text2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub content: String,
    pub font: AssetKey,
    pub bounds: Vec2,
    pub transform: Transform2,
}

impl Text2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        content: impl Into<String>,
        font: AssetKey,
        bounds: Vec2,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            content: content.into(),
            font,
            bounds,
            transform: Transform2::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VectorShapeKind2dSceneCommand {
    Polyline { points: Vec<Vec2>, closed: bool },
    Polygon { points: Vec<Vec2> },
    Circle { radius: f32, segments: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VectorStyle2dSceneCommand {
    pub stroke_color: ColorRgba,
    pub stroke_width: f32,
    pub fill_color: Option<ColorRgba>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VectorShape2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub kind: VectorShapeKind2dSceneCommand,
    pub style: VectorStyle2dSceneCommand,
    pub z_index: f32,
    pub transform: Transform2,
}

impl VectorShape2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        kind: VectorShapeKind2dSceneCommand,
        style: VectorStyle2dSceneCommand,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            kind,
            style,
            z_index: 0.0,
            transform: Transform2::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KinematicBody2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub velocity: Vec2,
    pub gravity_scale: f32,
    pub terminal_velocity: f32,
}

impl KinematicBody2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        velocity: Vec2,
        gravity_scale: f32,
        terminal_velocity: f32,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            velocity,
            gravity_scale,
            terminal_velocity,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AabbCollider2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub size: Vec2,
    pub offset: Vec2,
    pub layer: String,
    pub mask: Vec<String>,
}

impl AabbCollider2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        size: Vec2,
        offset: Vec2,
        layer: impl Into<String>,
        mask: Vec<String>,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            size,
            offset,
            layer: layer.into(),
            mask,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CircleCollider2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub radius: f32,
    pub offset: Vec2,
}

impl CircleCollider2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        radius: f32,
        offset: Vec2,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            radius,
            offset,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Trigger2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub size: Vec2,
    pub offset: Vec2,
    pub layer: String,
    pub mask: Vec<String>,
    pub event: Option<String>,
}

impl Trigger2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        size: Vec2,
        offset: Vec2,
        layer: impl Into<String>,
        mask: Vec<String>,
        event: Option<String>,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            size,
            offset,
            layer: layer.into(),
            mask,
            event,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionEventRule2dSceneCommand {
    pub source_mod: String,
    pub id: String,
    pub source: EntitySelector,
    pub target: EntitySelector,
    pub event: String,
    pub once_per_overlap: bool,
}

impl CollisionEventRule2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        id: impl Into<String>,
        source: EntitySelector,
        target: EntitySelector,
        event: impl Into<String>,
        once_per_overlap: bool,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            id: id.into(),
            source,
            target,
            event: event.into(),
            once_per_overlap,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MotionController2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub max_speed: f32,
    pub acceleration: f32,
    pub deceleration: f32,
    pub air_acceleration: f32,
    pub gravity: f32,
    pub jump_velocity: f32,
    pub terminal_velocity: f32,
}

impl MotionController2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        max_speed: f32,
        acceleration: f32,
        deceleration: f32,
        air_acceleration: f32,
        gravity: f32,
        jump_velocity: f32,
        terminal_velocity: f32,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            max_speed,
            acceleration,
            deceleration,
            air_acceleration,
            gravity,
            jump_velocity,
            terminal_velocity,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraFollow2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub target: String,
    pub offset: Vec2,
    pub lerp: f32,
}

impl CameraFollow2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        target: impl Into<String>,
        offset: Vec2,
        lerp: f32,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            target: target.into(),
            offset,
            lerp,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parallax2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub camera: String,
    pub factor: Vec2,
    pub anchor: Vec2,
    pub camera_origin: Option<Vec2>,
}

impl Parallax2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        camera: impl Into<String>,
        factor: Vec2,
        anchor: Vec2,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            camera: camera.into(),
            factor,
            anchor,
            camera_origin: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileMapMarker2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub tilemap_entity: Option<String>,
    pub symbol: String,
    pub index: usize,
    pub offset: Vec2,
}

impl TileMapMarker2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        tilemap_entity: Option<String>,
        symbol: impl Into<String>,
        index: usize,
        offset: Vec2,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            tilemap_entity,
            symbol: symbol.into(),
            index,
            offset,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mesh3dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub mesh_asset: AssetKey,
    pub transform: Transform3,
}

impl Mesh3dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        mesh_asset: AssetKey,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            mesh_asset,
            transform: Transform3::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Material3dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub label: String,
    pub albedo: ColorRgba,
    pub source: Option<AssetKey>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Text3dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub content: String,
    pub font: AssetKey,
    pub size: f32,
    pub transform: Transform3,
}

impl Text3dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        content: impl Into<String>,
        font: AssetKey,
        size: f32,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            content: content.into(),
            font,
            size,
            transform: Transform3::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SceneUiTarget {
    ScreenSpace {
        layer: SceneUiLayer,
        viewport: Option<SceneUiViewport>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneUiViewport {
    pub width: f32,
    pub height: f32,
    pub scaling: SceneUiViewportScaling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneUiViewportScaling {
    Expand,
    Fixed,
    Fit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SceneUiLayer {
    Background,
    Hud,
    Menu,
    Debug,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneUiDocument {
    pub target: SceneUiTarget,
    pub root: SceneUiNode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneUiNode {
    pub id: Option<String>,
    pub kind: SceneUiNodeKind,
    pub style_class: Option<String>,
    pub style: SceneUiStyle,
    pub binds: SceneUiBinds,
    pub on_click: Option<SceneUiEventBinding>,
    pub on_change: Option<SceneUiEventBinding>,
    pub children: Vec<SceneUiNode>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SceneUiBinds {
    pub text: Option<String>,
    pub visible: Option<String>,
    pub enabled: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SceneUiNodeKind {
    Panel,
    Row,
    Column,
    Stack,
    Text {
        content: String,
        font: Option<AssetKey>,
    },
    Button {
        text: String,
        font: Option<AssetKey>,
    },
    ProgressBar {
        value: f32,
    },
    Slider {
        value: f32,
        min: f32,
        max: f32,
        step: f32,
    },
    Toggle {
        checked: bool,
        text: String,
        font: Option<AssetKey>,
    },
    OptionSet {
        selected: String,
        options: Vec<String>,
        font: Option<AssetKey>,
    },
    Dropdown {
        selected: String,
        options: Vec<String>,
        font: Option<AssetKey>,
    },
    Spacer,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneUiStyle {
    pub left: Option<f32>,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub padding: f32,
    pub gap: f32,
    pub background: Option<ColorRgba>,
    pub color: Option<ColorRgba>,
    pub border_color: Option<ColorRgba>,
    pub border_width: f32,
    pub border_radius: f32,
    pub font_size: f32,
    pub word_wrap: bool,
    pub fit_to_width: bool,
    pub align: SceneUiTextAlign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneUiTextAlign {
    Start,
    Center,
}

impl Default for SceneUiStyle {
    fn default() -> Self {
        Self {
            left: None,
            top: None,
            right: None,
            bottom: None,
            width: None,
            height: None,
            padding: 0.0,
            gap: 0.0,
            background: None,
            color: None,
            border_color: None,
            border_width: 0.0,
            border_radius: 0.0,
            font_size: 16.0,
            word_wrap: false,
            fit_to_width: false,
            align: SceneUiTextAlign::Start,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneUiEventBinding {
    pub event: String,
    pub payload: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub document: SceneUiDocument,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiThemeSetSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub active: Option<String>,
    pub themes: Vec<SceneUiTheme>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneUiTheme {
    pub id: String,
    pub palette: SceneUiThemePalette,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneUiThemePalette {
    pub background: ColorRgba,
    pub surface: ColorRgba,
    pub surface_alt: ColorRgba,
    pub text: ColorRgba,
    pub text_muted: ColorRgba,
    pub border: ColorRgba,
    pub accent: ColorRgba,
    pub accent_text: ColorRgba,
    pub danger: ColorRgba,
    pub warning: ColorRgba,
    pub success: ColorRgba,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioCueSceneCommand {
    pub source_mod: String,
    pub name: String,
    pub clip: AssetKey,
    pub min_interval: Option<f32>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SceneEntityLifecycleOverride {
    pub visible: Option<bool>,
    pub simulation_enabled: Option<bool>,
    pub collision_enabled: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActivationEntrySceneCommand {
    pub target: EntitySelector,
    pub lifecycle: SceneEntityLifecycleOverride,
    pub transform: Option<Transform3>,
    pub velocity: Option<Vec2>,
    pub angular_velocity: Option<f32>,
    pub properties: BTreeMap<String, ScenePropertyValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActivationSetSceneCommand {
    pub source_mod: String,
    pub id: String,
    pub entries: Vec<ActivationEntrySceneCommand>,
}

#[derive(Debug, Default)]
pub struct CameraFollow2dSceneService {
    commands: Mutex<Vec<CameraFollow2dSceneCommand>>,
}

impl CameraFollow2dSceneService {
    pub fn queue(&self, command: CameraFollow2dSceneCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("camera follow scene service mutex should not be poisoned");
        commands.retain(|existing| existing.entity_name != command.entity_name);
        commands.push(command);
    }

    pub fn clear(&self) {
        let mut commands = self
            .commands
            .lock()
            .expect("camera follow scene service mutex should not be poisoned");
        commands.clear();
    }

    pub fn commands(&self) -> Vec<CameraFollow2dSceneCommand> {
        let commands = self
            .commands
            .lock()
            .expect("camera follow scene service mutex should not be poisoned");
        commands.clone()
    }

    pub fn follow(&self, entity_name: &str) -> Option<CameraFollow2dSceneCommand> {
        let commands = self
            .commands
            .lock()
            .expect("camera follow scene service mutex should not be poisoned");
        commands
            .iter()
            .find(|command| command.entity_name == entity_name)
            .cloned()
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.commands()
            .into_iter()
            .map(|command| command.entity_name)
            .collect()
    }
}

#[derive(Debug, Default)]
pub struct Parallax2dSceneService {
    commands: Mutex<Vec<Parallax2dSceneCommand>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityPoolSnapshot {
    pub pool: String,
    pub members: Vec<String>,
    pub active_members: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct EntityPoolState {
    members: BTreeMap<String, Vec<String>>,
    active_members: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Debug, Default)]
pub struct EntityPoolSceneService {
    state: Mutex<EntityPoolState>,
}

impl EntityPoolSceneService {
    pub fn queue(&self, command: EntityPoolSceneCommand) {
        let mut state = self
            .state
            .lock()
            .expect("entity pool scene service mutex should not be poisoned");
        let active_members = state
            .active_members
            .remove(&command.pool)
            .unwrap_or_default()
            .into_iter()
            .filter(|member| command.members.iter().any(|candidate| candidate == member))
            .collect();
        state.members.insert(command.pool.clone(), command.members);
        state.active_members.insert(command.pool, active_members);
    }

    pub fn clear(&self) {
        let mut state = self
            .state
            .lock()
            .expect("entity pool scene service mutex should not be poisoned");
        state.members.clear();
        state.active_members.clear();
    }

    pub fn members(&self, pool: &str) -> Vec<String> {
        self.state
            .lock()
            .expect("entity pool scene service mutex should not be poisoned")
            .members
            .get(pool)
            .cloned()
            .unwrap_or_default()
    }

    pub fn active_members(&self, pool: &str) -> Vec<String> {
        self.state
            .lock()
            .expect("entity pool scene service mutex should not be poisoned")
            .active_members
            .get(pool)
            .map(|members| members.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn active_count(&self, pool: &str) -> usize {
        self.state
            .lock()
            .expect("entity pool scene service mutex should not be poisoned")
            .active_members
            .get(pool)
            .map(BTreeSet::len)
            .unwrap_or(0)
    }

    pub fn snapshot(&self, pool: &str) -> Option<EntityPoolSnapshot> {
        let state = self
            .state
            .lock()
            .expect("entity pool scene service mutex should not be poisoned");
        state.members.get(pool).map(|members| EntityPoolSnapshot {
            pool: pool.to_owned(),
            members: members.clone(),
            active_members: state
                .active_members
                .get(pool)
                .map(|members| members.iter().cloned().collect())
                .unwrap_or_default(),
        })
    }

    pub fn acquire(&self, scene_service: &SceneService, pool: &str) -> Option<String> {
        let mut state = self
            .state
            .lock()
            .expect("entity pool scene service mutex should not be poisoned");
        let members = state.members.get(pool)?;
        let free_member = members
            .iter()
            .find(|member| {
                !state
                    .active_members
                    .get(pool)
                    .map(|active| active.contains(*member))
                    .unwrap_or(false)
            })
            .cloned()?;
        state
            .active_members
            .entry(pool.to_owned())
            .or_default()
            .insert(free_member.clone());
        drop(state);
        let _ = scene_service.set_visible(&free_member, true);
        let _ = scene_service.set_simulation_enabled(&free_member, true);
        let _ = scene_service.set_collision_enabled(&free_member, true);
        Some(free_member)
    }

    pub fn release(&self, scene_service: &SceneService, pool: &str, entity_name: &str) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("entity pool scene service mutex should not be poisoned");
        let Some(members) = state.members.get(pool) else {
            return false;
        };
        if !members.iter().any(|member| member == entity_name) {
            return false;
        }
        let was_active = state
            .active_members
            .entry(pool.to_owned())
            .or_default()
            .remove(entity_name);
        drop(state);
        let _ = scene_service.set_visible(entity_name, false);
        let _ = scene_service.set_simulation_enabled(entity_name, false);
        let _ = scene_service.set_collision_enabled(entity_name, false);
        was_active
    }

    pub fn release_all(&self, scene_service: &SceneService, pool: &str) -> usize {
        let mut state = self
            .state
            .lock()
            .expect("entity pool scene service mutex should not be poisoned");
        let Some(active_members) = state.active_members.get_mut(pool) else {
            return 0;
        };
        let released: Vec<String> = active_members.iter().cloned().collect();
        active_members.clear();
        drop(state);

        for entity_name in &released {
            let _ = scene_service.set_visible(entity_name, false);
            let _ = scene_service.set_simulation_enabled(entity_name, false);
            let _ = scene_service.set_collision_enabled(entity_name, false);
        }

        released.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LifetimeState {
    pub entity_name: String,
    pub duration_seconds: f32,
    pub remaining_seconds: f32,
    pub outcome: LifetimeExpirationOutcome,
}

#[derive(Debug, Default)]
pub struct LifetimeSceneService {
    definitions: Mutex<BTreeMap<String, LifetimeState>>,
    lifetimes: Mutex<BTreeMap<String, LifetimeState>>,
}

impl LifetimeSceneService {
    pub fn queue(&self, command: LifetimeSceneCommand) {
        let lifetime = LifetimeState {
            entity_name: command.entity_name,
            duration_seconds: command.seconds.max(0.0),
            remaining_seconds: command.seconds.max(0.0),
            outcome: command.outcome,
        };
        self.definitions
            .lock()
            .expect("lifetime scene service mutex should not be poisoned")
            .insert(lifetime.entity_name.clone(), lifetime.clone());
        self.lifetimes
            .lock()
            .expect("lifetime scene service mutex should not be poisoned")
            .insert(lifetime.entity_name.clone(), lifetime);
    }

    pub fn clear(&self) {
        self.definitions
            .lock()
            .expect("lifetime scene service mutex should not be poisoned")
            .clear();
        self.lifetimes
            .lock()
            .expect("lifetime scene service mutex should not be poisoned")
            .clear();
    }

    pub fn lifetime(&self, entity_name: &str) -> Option<LifetimeState> {
        self.lifetimes
            .lock()
            .expect("lifetime scene service mutex should not be poisoned")
            .get(entity_name)
            .cloned()
    }

    pub fn reset_lifetime(&self, entity_name: &str) -> bool {
        let Some(mut lifetime) = self
            .definitions
            .lock()
            .expect("lifetime scene service mutex should not be poisoned")
            .get(entity_name)
            .cloned()
        else {
            return false;
        };
        lifetime.remaining_seconds = lifetime.duration_seconds;
        self.lifetimes
            .lock()
            .expect("lifetime scene service mutex should not be poisoned")
            .insert(entity_name.to_owned(), lifetime);
        true
    }

    pub fn tick(&self, delta_seconds: f32) -> Vec<LifetimeState> {
        let mut lifetimes = self
            .lifetimes
            .lock()
            .expect("lifetime scene service mutex should not be poisoned");
        let mut expired = Vec::new();
        let mut expired_names = Vec::new();
        for (entity_name, lifetime) in lifetimes.iter_mut() {
            lifetime.remaining_seconds -= delta_seconds.max(0.0);
            if lifetime.remaining_seconds <= 0.0 {
                expired.push(lifetime.clone());
                expired_names.push(entity_name.clone());
            }
        }
        for entity_name in expired_names {
            lifetimes.remove(&entity_name);
        }
        expired
    }
}

impl Parallax2dSceneService {
    pub fn queue(&self, command: Parallax2dSceneCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("parallax scene service mutex should not be poisoned");
        commands.retain(|existing| existing.entity_name != command.entity_name);
        commands.push(command);
    }

    pub fn clear(&self) {
        let mut commands = self
            .commands
            .lock()
            .expect("parallax scene service mutex should not be poisoned");
        commands.clear();
    }

    pub fn commands(&self) -> Vec<Parallax2dSceneCommand> {
        let commands = self
            .commands
            .lock()
            .expect("parallax scene service mutex should not be poisoned");
        commands.clone()
    }

    pub fn set_camera_origin(&self, entity_name: &str, camera_origin: Vec2) -> bool {
        let mut commands = self
            .commands
            .lock()
            .expect("parallax scene service mutex should not be poisoned");
        let Some(command) = commands
            .iter_mut()
            .find(|command| command.entity_name == entity_name)
        else {
            return false;
        };
        command.camera_origin = Some(camera_origin);
        true
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.commands()
            .into_iter()
            .map(|command| command.entity_name)
            .collect()
    }
}

impl Material3dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        label: impl Into<String>,
        source: Option<AssetKey>,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            label: label.into(),
            albedo: ColorRgba::WHITE,
            source,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SceneCommand {
    SpawnNamedEntity {
        name: String,
        transform: Option<Transform3>,
    },
    ConfigureEntity {
        entity_name: String,
        lifecycle: SceneEntityLifecycle,
        tags: Vec<String>,
        groups: Vec<String>,
        properties: BTreeMap<String, ScenePropertyValue>,
    },
    SelectScene {
        scene: SceneKey,
    },
    ReloadActiveScene,
    ClearEntities,
    QueueSprite2d {
        command: Sprite2dSceneCommand,
    },
    QueueTileMap2d {
        command: TileMap2dSceneCommand,
    },
    QueueText2d {
        command: Text2dSceneCommand,
    },
    QueueVectorShape2d {
        command: VectorShape2dSceneCommand,
    },
    QueueEntityPool {
        command: EntityPoolSceneCommand,
    },
    QueueLifetime {
        command: LifetimeSceneCommand,
    },
    QueueProjectileEmitter2d {
        command: ProjectileEmitter2dSceneCommand,
    },
    QueueParticleEmitter2d {
        command: ParticleEmitter2dSceneCommand,
    },
    QueueVelocity2d {
        command: Velocity2dSceneCommand,
    },
    QueueBounds2d {
        command: Bounds2dSceneCommand,
    },
    QueueFreeflightMotion2d {
        command: FreeflightMotion2dSceneCommand,
    },
    QueueKinematicBody2d {
        command: KinematicBody2dSceneCommand,
    },
    QueueAabbCollider2d {
        command: AabbCollider2dSceneCommand,
    },
    QueueCircleCollider2d {
        command: CircleCollider2dSceneCommand,
    },
    QueueTrigger2d {
        command: Trigger2dSceneCommand,
    },
    QueueCollisionEventRule2d {
        command: CollisionEventRule2dSceneCommand,
    },
    QueueMotionController2d {
        command: MotionController2dSceneCommand,
    },
    QueueCameraFollow2d {
        command: CameraFollow2dSceneCommand,
    },
    QueueParallax2d {
        command: Parallax2dSceneCommand,
    },
    QueueTileMapMarker2d {
        command: TileMapMarker2dSceneCommand,
    },
    QueueMesh3d {
        command: Mesh3dSceneCommand,
    },
    QueueMaterial3d {
        command: Material3dSceneCommand,
    },
    QueueText3d {
        command: Text3dSceneCommand,
    },
    QueueUi {
        command: UiSceneCommand,
    },
    QueueUiThemeSet {
        command: UiThemeSetSceneCommand,
    },
    QueueAudioCue {
        command: AudioCueSceneCommand,
    },
    QueueActivationSet {
        command: ActivationSetSceneCommand,
    },
    ActivateSet {
        id: String,
    },
}

impl SceneCommand {
    pub fn queue_motion_controller(command: MotionController2dSceneCommand) -> Self {
        Self::QueueMotionController2d { command }
    }

    pub fn motion_controller_command(&self) -> Option<&MotionController2dSceneCommand> {
        match self {
            Self::QueueMotionController2d { command } => Some(command),
            _ => None,
        }
    }

    pub fn is_motion_controller_command(&self) -> bool {
        self.motion_controller_command().is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneEvent {
    EntitySpawned {
        entity_id: u64,
        name: String,
    },
    SceneSelected {
        scene: SceneKey,
    },
    SceneReloadRequested {
        scene: SceneKey,
    },
    EntitiesCleared,
    SpriteQueued {
        entity_id: u64,
        entity_name: String,
        texture: AssetKey,
    },
    TileMapQueued {
        entity_id: u64,
        entity_name: String,
        tileset: AssetKey,
    },
    TextQueued {
        entity_id: u64,
        entity_name: String,
        font: AssetKey,
    },
    VectorQueued {
        entity_id: u64,
        entity_name: String,
    },
    EntityPoolQueued {
        pool: String,
    },
    LifetimeQueued {
        entity_id: u64,
        entity_name: String,
    },
    ProjectileEmitterQueued {
        entity_id: u64,
        entity_name: String,
        pool: String,
    },
    ParticleEmitterQueued {
        entity_id: u64,
        entity_name: String,
    },
    Velocity2dQueued {
        entity_id: u64,
        entity_name: String,
    },
    Bounds2dQueued {
        entity_id: u64,
        entity_name: String,
    },
    FreeflightMotion2dQueued {
        entity_id: u64,
        entity_name: String,
    },
    KinematicBodyQueued {
        entity_id: u64,
        entity_name: String,
    },
    AabbColliderQueued {
        entity_id: u64,
        entity_name: String,
    },
    CircleColliderQueued {
        entity_id: u64,
        entity_name: String,
    },
    TriggerQueued {
        entity_id: u64,
        entity_name: String,
        topic: Option<String>,
    },
    CollisionEventRuleQueued {
        rule_id: String,
        topic: String,
    },
    MotionControllerQueued {
        entity_id: u64,
        entity_name: String,
    },
    CameraFollowQueued {
        entity_id: u64,
        entity_name: String,
        target: String,
    },
    ParallaxQueued {
        entity_id: u64,
        entity_name: String,
        camera: String,
    },
    TileMapMarkerQueued {
        entity_id: u64,
        entity_name: String,
        symbol: String,
    },
    MeshQueued {
        entity_id: u64,
        entity_name: String,
        mesh_asset: AssetKey,
    },
    MaterialQueued {
        entity_id: u64,
        entity_name: String,
        material_label: String,
    },
    Text3dQueued {
        entity_id: u64,
        entity_name: String,
        font: AssetKey,
    },
    UiQueued {
        entity_id: u64,
        entity_name: String,
    },
    UiThemeSetQueued {
        entity_id: u64,
        entity_name: String,
    },
}

impl SceneEvent {
    pub fn motion_controller_queued(entity_id: u64, entity_name: impl Into<String>) -> Self {
        Self::MotionControllerQueued {
            entity_id,
            entity_name: entity_name.into(),
        }
    }

    pub fn motion_controller_entity_name(&self) -> Option<&str> {
        match self {
            Self::MotionControllerQueued { entity_name, .. } => Some(entity_name.as_str()),
            _ => None,
        }
    }
}

pub fn format_scene_command(command: &SceneCommand) -> String {
    match command {
        SceneCommand::SpawnNamedEntity { name, .. } => format!("scene.spawn({name})"),
        SceneCommand::ConfigureEntity { entity_name, .. } => {
            format!("scene.configure({entity_name})")
        }
        SceneCommand::SelectScene { scene } => format!("scene.select({})", scene.as_str()),
        SceneCommand::ReloadActiveScene => "scene.reload_active".to_owned(),
        SceneCommand::ClearEntities => "scene.clear".to_owned(),
        SceneCommand::QueueSprite2d { command } => format!(
            "scene.2d.sprite({}, {}, {}x{})",
            command.entity_name,
            command.texture.as_str(),
            command.size.x,
            command.size.y
        ),
        SceneCommand::QueueTileMap2d { command } => format!(
            "scene.2d.tilemap({}, {}, {} rows)",
            command.entity_name,
            command.tileset.as_str(),
            command.grid.len()
        ),
        SceneCommand::QueueText2d { command } => format!(
            "scene.2d.text({}, {}, {}x{})",
            command.entity_name,
            command.font.as_str(),
            command.bounds.x,
            command.bounds.y
        ),
        SceneCommand::QueueVectorShape2d { command } => format!(
            "scene.2d.vector({}, {:?})",
            command.entity_name, command.kind
        ),
        SceneCommand::QueueEntityPool { command } => {
            format!(
                "scene.pool({}, {} members)",
                command.pool,
                command.members.len()
            )
        }
        SceneCommand::QueueLifetime { command } => {
            format!(
                "scene.lifetime({}, {}s)",
                command.entity_name, command.seconds
            )
        }
        SceneCommand::QueueProjectileEmitter2d { command } => format!(
            "scene.2d.projectile_emitter({}, pool={}, speed={})",
            command.entity_name, command.pool, command.speed
        ),
        SceneCommand::QueueParticleEmitter2d { command } => format!(
            "scene.2d.particle_emitter({}, spawn_rate={}, lifetime={})",
            command.entity_name, command.spawn_rate, command.particle_lifetime
        ),
        SceneCommand::QueueVelocity2d { command } => format!(
            "scene.2d.velocity({}, {}, {})",
            command.entity_name, command.velocity.x, command.velocity.y
        ),
        SceneCommand::QueueBounds2d { command } => format!(
            "scene.2d.bounds({}, {:?})",
            command.entity_name, command.behavior
        ),
        SceneCommand::QueueFreeflightMotion2d { command } => format!(
            "scene.2d.freeflight({}, max_speed={}, max_angular_speed={})",
            command.entity_name, command.max_speed, command.max_angular_speed
        ),
        SceneCommand::QueueKinematicBody2d { command } => format!(
            "scene.2d.physics.body({}, {}, {}, {})",
            command.entity_name, command.velocity.x, command.velocity.y, command.gravity_scale
        ),
        SceneCommand::QueueAabbCollider2d { command } => format!(
            "scene.2d.physics.collider({}, {}x{}, {})",
            command.entity_name, command.size.x, command.size.y, command.layer
        ),
        SceneCommand::QueueCircleCollider2d { command } => format!(
            "scene.2d.physics.circle({}, r={}, {}, {})",
            command.entity_name, command.radius, command.offset.x, command.offset.y
        ),
        SceneCommand::QueueTrigger2d { command } => format!(
            "scene.2d.physics.trigger({}, {}x{}, {})",
            command.entity_name,
            command.size.x,
            command.size.y,
            command.event.as_deref().unwrap_or("none")
        ),
        SceneCommand::QueueCollisionEventRule2d { command } => format!(
            "scene.2d.physics.collision_event({}, {})",
            command.id, command.event
        ),
        SceneCommand::QueueMotionController2d { command } => format!(
            "scene.2d.motion({}, max_speed={}, jump_velocity={})",
            command.entity_name, command.max_speed, command.jump_velocity
        ),
        SceneCommand::QueueCameraFollow2d { command } => format!(
            "scene.2d.camera_follow({}, {}, {}, {})",
            command.entity_name, command.target, command.offset.x, command.offset.y
        ),
        SceneCommand::QueueParallax2d { command } => format!(
            "scene.2d.parallax({}, {}, {}, {})",
            command.entity_name, command.camera, command.factor.x, command.factor.y
        ),
        SceneCommand::QueueTileMapMarker2d { command } => format!(
            "scene.2d.tilemap_marker({}, {}, #{})",
            command.entity_name, command.symbol, command.index
        ),
        SceneCommand::QueueMesh3d { command } => format!(
            "scene.3d.mesh({}, {})",
            command.entity_name,
            command.mesh_asset.as_str()
        ),
        SceneCommand::QueueMaterial3d { command } => format!(
            "scene.3d.material({}, {}, {})",
            command.entity_name,
            command.label,
            command
                .source
                .as_ref()
                .map(|asset| asset.as_str().to_owned())
                .unwrap_or_else(|| "generated".to_owned())
        ),
        SceneCommand::QueueText3d { command } => format!(
            "scene.3d.text({}, {}, {})",
            command.entity_name,
            command.font.as_str(),
            command.size
        ),
        SceneCommand::QueueUi { command } => {
            format!("scene.ui({}, screen-space)", command.entity_name)
        }
        SceneCommand::QueueUiThemeSet { command } => format!(
            "scene.ui.theme_set({}, {} themes)",
            command.entity_name,
            command.themes.len()
        ),
        SceneCommand::QueueAudioCue { command } => {
            format!(
                "scene.audio.cue({}, {})",
                command.name,
                command.clip.as_str()
            )
        }
        SceneCommand::QueueActivationSet { command } => {
            format!(
                "scene.activation_set({}, {} entries)",
                command.id,
                command.entries.len()
            )
        }
        SceneCommand::ActivateSet { id } => format!("scene.activate_set({id})"),
    }
}

#[derive(Debug, Default)]
pub struct ActivationSetSceneService {
    sets: Mutex<BTreeMap<String, ActivationSetSceneCommand>>,
}

impl ActivationSetSceneService {
    pub fn queue(&self, command: ActivationSetSceneCommand) {
        self.sets
            .lock()
            .expect("activation set scene service mutex should not be poisoned")
            .insert(command.id.clone(), command);
    }

    pub fn clear(&self) {
        self.sets
            .lock()
            .expect("activation set scene service mutex should not be poisoned")
            .clear();
    }

    pub fn activation_set(&self, id: &str) -> Option<ActivationSetSceneCommand> {
        self.sets
            .lock()
            .expect("activation set scene service mutex should not be poisoned")
            .get(id)
            .cloned()
    }

    pub fn sets(&self) -> Vec<ActivationSetSceneCommand> {
        self.sets
            .lock()
            .expect("activation set scene service mutex should not be poisoned")
            .values()
            .cloned()
            .collect()
    }
}

#[derive(Debug, Default)]
struct SceneState {
    next_id: u64,
    entities: BTreeMap<u64, SceneEntity>,
    selected_scene: Option<SceneKey>,
}

impl SceneState {
    pub fn spawn_with_transform(
        &mut self,
        name: impl Into<String>,
        transform: Transform3,
    ) -> SceneEntityId {
        let id = SceneEntityId::new(self.next_id);
        self.next_id += 1;

        let entity = SceneEntity {
            id,
            name: name.into(),
            transform,
            lifecycle: SceneEntityLifecycle::default(),
            tags: Vec::new(),
            groups: Vec::new(),
            properties: BTreeMap::new(),
        };

        self.entities.insert(id.raw(), entity);
        id
    }

    pub fn entities(&self) -> impl Iterator<Item = &SceneEntity> {
        self.entities.values()
    }

    pub fn entity_by_name(&self, name: &str) -> Option<&SceneEntity> {
        self.entities.values().find(|entity| entity.name == name)
    }

    pub fn entity_by_name_mut(&mut self, name: &str) -> Option<&mut SceneEntity> {
        self.entities
            .values_mut()
            .find(|entity| entity.name == name)
    }
}

#[derive(Debug, Default)]
pub struct SceneService {
    state: Mutex<SceneState>,
}

impl SceneService {
    pub fn spawn(&self, name: impl Into<String>) -> SceneEntityId {
        self.spawn_with_transform(name, Transform3::default())
    }

    pub fn spawn_with_transform(
        &self,
        name: impl Into<String>,
        transform: Transform3,
    ) -> SceneEntityId {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        state.spawn_with_transform(name, transform)
    }

    pub fn clear_entities(&self) {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        state.entities.clear();
    }

    pub fn remove_entities_by_name(&self, entity_names: &[String]) -> usize {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        let before = state.entities.len();

        state
            .entities
            .retain(|_, entity| !entity_names.iter().any(|name| name == &entity.name));

        before.saturating_sub(state.entities.len())
    }

    pub fn select_scene(&self, scene: SceneKey) {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        state.selected_scene = Some(scene);
    }

    pub fn selected_scene(&self) -> Option<SceneKey> {
        let state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        state.selected_scene.clone()
    }

    pub fn entity_count(&self) -> usize {
        let state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        state.entities.len()
    }

    pub fn entities(&self) -> Vec<SceneEntity> {
        let state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        state.entities().cloned().collect()
    }

    pub fn entity_by_name(&self, name: &str) -> Option<SceneEntity> {
        let state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        state.entity_by_name(name).cloned()
    }

    pub fn find_or_spawn_named_entity(&self, name: impl Into<String>) -> SceneEntityId {
        let name = name.into();
        self.entity_by_name(&name)
            .map(|entity| entity.id)
            .unwrap_or_else(|| self.spawn(name))
    }

    pub fn transform_of(&self, entity_name: &str) -> Option<Transform3> {
        self.entity_by_name(entity_name)
            .map(|entity| entity.transform)
    }

    pub fn set_transform(&self, entity_name: &str, transform: Transform3) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        let Some(entity) = state.entity_by_name_mut(entity_name) else {
            return false;
        };
        entity.transform = transform;
        true
    }

    pub fn lifecycle_of(&self, entity_name: &str) -> Option<SceneEntityLifecycle> {
        self.entity_by_name(entity_name)
            .map(|entity| entity.lifecycle)
    }

    pub fn set_lifecycle(&self, entity_name: &str, lifecycle: SceneEntityLifecycle) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        let Some(entity) = state.entity_by_name_mut(entity_name) else {
            return false;
        };
        entity.lifecycle = lifecycle;
        true
    }

    pub fn set_visible(&self, entity_name: &str, visible: bool) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        let Some(entity) = state.entity_by_name_mut(entity_name) else {
            return false;
        };
        entity.lifecycle.visible = visible;
        true
    }

    pub fn set_simulation_enabled(&self, entity_name: &str, enabled: bool) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        let Some(entity) = state.entity_by_name_mut(entity_name) else {
            return false;
        };
        entity.lifecycle.simulation_enabled = enabled;
        true
    }

    pub fn set_collision_enabled(&self, entity_name: &str, enabled: bool) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        let Some(entity) = state.entity_by_name_mut(entity_name) else {
            return false;
        };
        entity.lifecycle.collision_enabled = enabled;
        true
    }

    pub fn is_visible(&self, entity_name: &str) -> bool {
        self.lifecycle_of(entity_name)
            .map(|lifecycle| lifecycle.visible)
            .unwrap_or(false)
    }

    pub fn is_simulation_enabled(&self, entity_name: &str) -> bool {
        self.lifecycle_of(entity_name)
            .map(|lifecycle| lifecycle.simulation_enabled)
            .unwrap_or(false)
    }

    pub fn is_collision_enabled(&self, entity_name: &str) -> bool {
        self.lifecycle_of(entity_name)
            .map(|lifecycle| lifecycle.collision_enabled)
            .unwrap_or(false)
    }

    pub fn configure_entity_metadata(
        &self,
        entity_name: &str,
        lifecycle: SceneEntityLifecycle,
        tags: Vec<String>,
        groups: Vec<String>,
        properties: BTreeMap<String, ScenePropertyValue>,
    ) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        let Some(entity) = state.entity_by_name_mut(entity_name) else {
            return false;
        };
        entity.lifecycle = lifecycle;
        entity.tags = tags;
        entity.groups = groups;
        entity.properties = properties;
        true
    }

    pub fn tags_of(&self, entity_name: &str) -> Vec<String> {
        self.entity_by_name(entity_name)
            .map(|entity| entity.tags)
            .unwrap_or_default()
    }

    pub fn groups_of(&self, entity_name: &str) -> Vec<String> {
        self.entity_by_name(entity_name)
            .map(|entity| entity.groups)
            .unwrap_or_default()
    }

    pub fn has_tag(&self, entity_name: &str, tag: &str) -> bool {
        self.entity_by_name(entity_name)
            .map(|entity| entity.tags.iter().any(|value| value == tag))
            .unwrap_or(false)
    }

    pub fn has_group(&self, entity_name: &str, group: &str) -> bool {
        self.entity_by_name(entity_name)
            .map(|entity| entity.groups.iter().any(|value| value == group))
            .unwrap_or(false)
    }

    pub fn entities_by_tag(&self, tag: &str) -> Vec<String> {
        self.entities()
            .into_iter()
            .filter(|entity| entity.tags.iter().any(|value| value == tag))
            .map(|entity| entity.name)
            .collect()
    }

    pub fn entities_by_group(&self, group: &str) -> Vec<String> {
        self.entities()
            .into_iter()
            .filter(|entity| entity.groups.iter().any(|value| value == group))
            .map(|entity| entity.name)
            .collect()
    }

    pub fn active_entities_by_tag(&self, tag: &str) -> Vec<String> {
        self.entities()
            .into_iter()
            .filter(|entity| {
                entity.lifecycle.simulation_enabled && entity.tags.iter().any(|value| value == tag)
            })
            .map(|entity| entity.name)
            .collect()
    }

    pub fn property_of(&self, entity_name: &str, key: &str) -> Option<ScenePropertyValue> {
        self.entity_by_name(entity_name)
            .and_then(|entity| entity.properties.get(key).cloned())
    }

    pub fn set_property(
        &self,
        entity_name: &str,
        key: impl Into<String>,
        value: ScenePropertyValue,
    ) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        let Some(entity) = state.entity_by_name_mut(entity_name) else {
            return false;
        };
        entity.properties.insert(key.into(), value);
        true
    }

    pub fn rotate_entity_2d(&self, entity_name: &str, delta_radians: f32) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        let Some(entity) = state.entity_by_name_mut(entity_name) else {
            return false;
        };
        entity.transform.rotation_euler.z += delta_radians;
        true
    }

    pub fn set_entity_rotation_2d(&self, entity_name: &str, radians: f32) -> bool {
        if !radians.is_finite() {
            return false;
        }
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        let Some(entity) = state.entity_by_name_mut(entity_name) else {
            return false;
        };
        entity.transform.rotation_euler.z = radians;
        true
    }

    pub fn rotate_entity_3d(&self, entity_name: &str, delta_euler: Vec3) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        let Some(entity) = state.entity_by_name_mut(entity_name) else {
            return false;
        };
        entity.transform.rotation_euler.x += delta_euler.x;
        entity.transform.rotation_euler.y += delta_euler.y;
        entity.transform.rotation_euler.z += delta_euler.z;
        true
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.entities()
            .into_iter()
            .map(|entity| entity.name)
            .collect()
    }
}

#[derive(Debug, Default)]
pub struct HydratedSceneState {
    snapshot: Mutex<HydratedSceneSnapshot>,
}

impl HydratedSceneState {
    pub fn snapshot(&self) -> HydratedSceneSnapshot {
        self.snapshot
            .lock()
            .expect("hydrated scene state mutex should not be poisoned")
            .clone()
    }

    pub fn replace(&self, snapshot: HydratedSceneSnapshot) -> HydratedSceneSnapshot {
        let mut state = self
            .snapshot
            .lock()
            .expect("hydrated scene state mutex should not be poisoned");
        std::mem::replace(&mut *state, snapshot)
    }

    pub fn clear(&self) -> HydratedSceneSnapshot {
        self.replace(HydratedSceneSnapshot::default())
    }
}

#[derive(Debug, Default)]
pub struct SceneCommandQueue {
    commands: Mutex<Vec<SceneCommand>>,
}

impl SceneCommandQueue {
    pub fn submit(&self, command: SceneCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("scene command queue mutex should not be poisoned");
        commands.push(command);
    }

    pub fn pending(&self) -> Vec<SceneCommand> {
        let commands = self
            .commands
            .lock()
            .expect("scene command queue mutex should not be poisoned");
        commands.clone()
    }

    pub fn drain(&self) -> Vec<SceneCommand> {
        let mut commands = self
            .commands
            .lock()
            .expect("scene command queue mutex should not be poisoned");
        commands.drain(..).collect()
    }
}

#[derive(Debug, Default)]
pub struct SceneEventQueue {
    events: Mutex<Vec<SceneEvent>>,
}

impl SceneEventQueue {
    pub fn publish(&self, event: SceneEvent) {
        let mut events = self
            .events
            .lock()
            .expect("scene event queue mutex should not be poisoned");
        events.push(event);
    }

    pub fn pending(&self) -> Vec<SceneEvent> {
        let events = self
            .events
            .lock()
            .expect("scene event queue mutex should not be poisoned");
        events.clone()
    }

    pub fn drain(&self) -> Vec<SceneEvent> {
        let mut events = self
            .events
            .lock()
            .expect("scene event queue mutex should not be poisoned");
        events.drain(..).collect()
    }
}

pub struct ScenePlugin;

impl RuntimePlugin for ScenePlugin {
    fn name(&self) -> &'static str {
        "amigo-scene"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(SceneService::default())?;
        registry.register(HydratedSceneState::default())?;
        registry.register(SceneTransitionService::default())?;
        registry.register(EntityPoolSceneService::default())?;
        registry.register(LifetimeSceneService::default())?;
        registry.register(CameraFollow2dSceneService::default())?;
        registry.register(Parallax2dSceneService::default())?;
        registry.register(ActivationSetSceneService::default())?;
        registry.register(SceneCommandQueue::default())?;
        registry.register(SceneEventQueue::default())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use super::{
        AabbCollider2dSceneCommand, CameraFollow2dSceneCommand, CameraFollow2dSceneService,
        EntityPoolSceneCommand, EntityPoolSceneService, HydratedSceneSnapshot, HydratedSceneState,
        KinematicBody2dSceneCommand, LifetimeExpirationOutcome, LifetimeSceneCommand,
        LifetimeSceneService, Material3dSceneCommand, Mesh3dSceneCommand,
        MotionController2dSceneCommand, Parallax2dSceneCommand, Parallax2dSceneService,
        SceneCommand, SceneCommandQueue, SceneEntityLifecycle, SceneEvent, SceneEventQueue,
        SceneKey, ScenePropertyValue, SceneService, SceneTransitionService, Sprite2dSceneCommand,
        Text2dSceneCommand, TileMap2dSceneCommand, TileMapMarker2dSceneCommand,
        Trigger2dSceneCommand, format_scene_command,
    };
    use amigo_assets::AssetKey;
    use amigo_math::{Transform3, Vec2, Vec3};

    #[test]
    fn scene_service_spawns_entities_in_order() {
        let scene = SceneService::default();

        let first = scene.spawn("core-root");
        let second = scene.spawn("playground-2d-camera");

        assert_eq!(first.raw(), 0);
        assert_eq!(second.raw(), 1);
        assert_eq!(scene.entity_count(), 2);
        assert_eq!(
            scene.entity_names(),
            vec!["core-root".to_owned(), "playground-2d-camera".to_owned()]
        );
    }

    #[test]
    fn scene_service_tracks_selected_scene() {
        let scene = SceneService::default();

        scene.select_scene(SceneKey::new("dev-shell"));

        assert_eq!(
            scene
                .selected_scene()
                .expect("selected scene should exist")
                .as_str(),
            "dev-shell"
        );
    }

    #[test]
    fn scene_service_can_find_entity_by_name() {
        let scene = SceneService::default();

        scene.spawn("playground-3d-probe");

        assert_eq!(
            scene
                .entity_by_name("playground-3d-probe")
                .expect("entity should exist")
                .name,
            "playground-3d-probe"
        );
    }

    #[test]
    fn scene_service_can_spawn_entity_with_transform() {
        let scene = SceneService::default();

        scene.spawn_with_transform(
            "playground-2d-sprite",
            Transform3 {
                translation: Vec3::new(8.0, -2.0, 0.0),
                rotation_euler: Vec3::new(0.0, 0.0, 0.25),
                scale: Vec3::new(2.0, 2.0, 1.0),
            },
        );

        assert_eq!(
            scene
                .entity_by_name("playground-2d-sprite")
                .expect("entity should exist")
                .transform
                .translation,
            Vec3::new(8.0, -2.0, 0.0)
        );
    }

    #[test]
    fn scene_service_tracks_lifecycle_without_mutating_transform() {
        let scene = SceneService::default();
        let transform = Transform3 {
            translation: Vec3::new(8.0, -2.0, 0.0),
            rotation_euler: Vec3::new(0.0, 0.0, 0.25),
            scale: Vec3::new(2.0, 2.0, 1.0),
        };
        scene.spawn_with_transform("actor", transform);

        assert!(scene.set_visible("actor", false));
        assert!(scene.set_simulation_enabled("actor", false));
        assert!(scene.set_collision_enabled("actor", false));

        assert!(!scene.is_visible("actor"));
        assert!(!scene.is_simulation_enabled("actor"));
        assert!(!scene.is_collision_enabled("actor"));
        assert_eq!(scene.transform_of("actor"), Some(transform));
    }

    #[test]
    fn scene_service_tracks_entity_metadata_queries() {
        let scene = SceneService::default();
        scene.spawn("actor");

        assert!(scene.configure_entity_metadata(
            "actor",
            SceneEntityLifecycle::default(),
            vec!["enemy".to_owned(), "flying".to_owned()],
            vec!["wave-1".to_owned()],
            BTreeMap::from([
                ("score_value".to_owned(), ScenePropertyValue::Int(100)),
                (
                    "label".to_owned(),
                    ScenePropertyValue::String("scout".to_owned())
                ),
            ]),
        ));

        assert!(scene.has_tag("actor", "enemy"));
        assert!(scene.has_group("actor", "wave-1"));
        assert_eq!(scene.entities_by_tag("enemy"), vec!["actor".to_owned()]);
        assert_eq!(scene.entities_by_group("wave-1"), vec!["actor".to_owned()]);
        assert_eq!(
            scene.property_of("actor", "score_value"),
            Some(ScenePropertyValue::Int(100))
        );

        assert!(scene.set_property("actor", "score_value", ScenePropertyValue::Int(250)));
        assert_eq!(
            scene.property_of("actor", "score_value"),
            Some(ScenePropertyValue::Int(250))
        );
    }

    #[test]
    fn scene_service_can_remove_entities_by_name() {
        let scene = SceneService::default();
        scene.spawn("core-root");
        scene.spawn("playground-2d-camera");
        scene.spawn("playground-2d-sprite");

        let removed = scene.remove_entities_by_name(&[
            "playground-2d-camera".to_owned(),
            "playground-2d-sprite".to_owned(),
        ]);

        assert_eq!(removed, 2);
        assert_eq!(scene.entity_names(), vec!["core-root".to_owned()]);
    }

    #[test]
    fn entity_pool_acquires_first_free_slot_and_reports_no_slot() {
        let scene = SceneService::default();
        scene.spawn("projectile-a");
        scene.spawn("projectile-b");
        let pools = EntityPoolSceneService::default();
        pools.queue(EntityPoolSceneCommand::new(
            "test",
            "projectiles",
            vec!["projectile-a".to_owned(), "projectile-b".to_owned()],
        ));

        assert_eq!(
            pools.acquire(&scene, "projectiles"),
            Some("projectile-a".to_owned())
        );
        assert_eq!(
            pools.acquire(&scene, "projectiles"),
            Some("projectile-b".to_owned())
        );
        assert_eq!(pools.acquire(&scene, "projectiles"), None);
        assert_eq!(
            pools.active_members("projectiles"),
            vec!["projectile-a".to_owned(), "projectile-b".to_owned()]
        );
        assert_eq!(pools.active_count("projectiles"), 2);
    }

    #[test]
    fn entity_pool_release_deactivates_and_reuses_member_without_moving_it() {
        let scene = SceneService::default();
        let transform = Transform3 {
            translation: Vec3::new(4.0, 5.0, 0.0),
            ..Transform3::default()
        };
        scene.spawn_with_transform("projectile-a", transform);
        let pools = EntityPoolSceneService::default();
        pools.queue(EntityPoolSceneCommand::new(
            "test",
            "projectiles",
            vec!["projectile-a".to_owned()],
        ));

        assert_eq!(
            pools.acquire(&scene, "projectiles"),
            Some("projectile-a".to_owned())
        );
        assert!(pools.release(&scene, "projectiles", "projectile-a"));

        assert!(!scene.is_visible("projectile-a"));
        assert!(!scene.is_simulation_enabled("projectile-a"));
        assert!(!scene.is_collision_enabled("projectile-a"));
        assert_eq!(scene.transform_of("projectile-a"), Some(transform));
        assert_eq!(
            pools.acquire(&scene, "projectiles"),
            Some("projectile-a".to_owned())
        );
    }

    #[test]
    fn entity_pool_release_all_deactivates_every_active_member() {
        let scene = SceneService::default();
        scene.spawn("projectile-a");
        scene.spawn("projectile-b");
        let pools = EntityPoolSceneService::default();
        pools.queue(EntityPoolSceneCommand::new(
            "test",
            "projectiles",
            vec!["projectile-a".to_owned(), "projectile-b".to_owned()],
        ));

        assert!(pools.acquire(&scene, "projectiles").is_some());
        assert!(pools.acquire(&scene, "projectiles").is_some());
        assert_eq!(pools.release_all(&scene, "projectiles"), 2);

        assert_eq!(pools.active_count("projectiles"), 0);
        assert!(!scene.is_visible("projectile-a"));
        assert!(!scene.is_visible("projectile-b"));
        assert!(!scene.is_simulation_enabled("projectile-a"));
        assert!(!scene.is_collision_enabled("projectile-b"));
    }

    #[test]
    fn lifetime_service_expires_after_tick() {
        let lifetimes = LifetimeSceneService::default();
        lifetimes.queue(LifetimeSceneCommand::new(
            "test",
            "projectile-a",
            0.25,
            LifetimeExpirationOutcome::Hide,
        ));

        assert!(lifetimes.tick(0.1).is_empty());
        let expired = lifetimes.tick(0.2);

        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].entity_name, "projectile-a");
        assert!(lifetimes.lifetime("projectile-a").is_none());
    }

    #[test]
    fn entity_pool_exposes_members_for_selector_use() {
        let pools = EntityPoolSceneService::default();
        pools.queue(EntityPoolSceneCommand::new(
            "test",
            "projectiles",
            vec!["projectile-a".to_owned(), "projectile-b".to_owned()],
        ));

        assert_eq!(
            pools.members("projectiles"),
            vec!["projectile-a".to_owned(), "projectile-b".to_owned()]
        );
    }

    #[test]
    fn camera_follow_scene_service_replaces_existing_entity_binding() {
        let service = CameraFollow2dSceneService::default();
        service.queue(CameraFollow2dSceneCommand::new(
            "playground-sidescroller",
            "playground-sidescroller-camera",
            "playground-sidescroller-player",
            Vec2::new(0.0, 0.0),
            1.0,
        ));
        service.queue(CameraFollow2dSceneCommand::new(
            "playground-sidescroller",
            "playground-sidescroller-camera",
            "playground-sidescroller-finish",
            Vec2::new(16.0, 0.0),
            0.5,
        ));

        let binding = service
            .follow("playground-sidescroller-camera")
            .expect("camera follow binding should exist");

        assert_eq!(service.commands().len(), 1);
        assert_eq!(binding.target, "playground-sidescroller-finish");
        assert_eq!(binding.offset, Vec2::new(16.0, 0.0));
        assert_eq!(binding.lerp, 0.5);
    }

    #[test]
    fn parallax_scene_service_tracks_camera_origin() {
        let service = Parallax2dSceneService::default();
        service.queue(Parallax2dSceneCommand::new(
            "playground-sidescroller",
            "playground-sidescroller-background-far",
            "playground-sidescroller-camera",
            Vec2::new(0.12, 0.04),
            Vec2::new(512.0, 256.0),
        ));

        assert!(service.set_camera_origin(
            "playground-sidescroller-background-far",
            Vec2::new(64.0, 64.0),
        ));

        let binding = service.commands();
        assert_eq!(binding.len(), 1);
        assert_eq!(binding[0].camera_origin, Some(Vec2::new(64.0, 64.0)));
        assert_eq!(
            service.entity_names(),
            vec!["playground-sidescroller-background-far".to_owned()]
        );
    }

    #[test]
    fn scene_service_can_rotate_2d_entity_by_name() {
        let scene = SceneService::default();
        scene.spawn_with_transform("square", Transform3::default());

        assert!(scene.rotate_entity_2d("square", 1.0));

        let transform = scene.transform_of("square").expect("entity should exist");
        assert_eq!(transform.rotation_euler.z, 1.0);
    }

    #[test]
    fn scene_service_can_rotate_3d_entity_by_name() {
        let scene = SceneService::default();
        scene.spawn_with_transform("cube", Transform3::default());

        assert!(scene.rotate_entity_3d("cube", Vec3::new(1.0, 2.0, 0.0)));

        let transform = scene.transform_of("cube").expect("entity should exist");
        assert_eq!(transform.rotation_euler.x, 1.0);
        assert_eq!(transform.rotation_euler.y, 2.0);
    }

    #[test]
    fn hydrated_scene_state_replaces_snapshots() {
        let hydrated = HydratedSceneState::default();
        let previous = hydrated.replace(HydratedSceneSnapshot {
            source_mod: Some("playground-2d".to_owned()),
            scene_id: Some("sprite-lab".to_owned()),
            relative_document_path: Some(PathBuf::from("scenes/sprite-lab/scene.yml")),
            entity_names: vec![
                "playground-2d-camera".to_owned(),
                "playground-2d-sprite".to_owned(),
            ],
            component_kinds: vec!["Camera2D x1".to_owned(), "Sprite2D x1".to_owned()],
        });

        assert_eq!(previous, HydratedSceneSnapshot::default());
        assert_eq!(hydrated.snapshot().scene_id.as_deref(), Some("sprite-lab"));
        assert_eq!(hydrated.clear().entity_names.len(), 2);
        assert_eq!(hydrated.snapshot(), HydratedSceneSnapshot::default());
    }

    #[test]
    fn queues_placeholder_scene_commands_and_events() {
        let commands = SceneCommandQueue::default();
        let events = SceneEventQueue::default();

        commands.submit(SceneCommand::SelectScene {
            scene: SceneKey::new("mesh-lab"),
        });
        commands.submit(SceneCommand::ReloadActiveScene);
        events.publish(SceneEvent::SceneSelected {
            scene: SceneKey::new("mesh-lab"),
        });
        events.publish(SceneEvent::SceneReloadRequested {
            scene: SceneKey::new("mesh-lab"),
        });

        assert_eq!(commands.pending().len(), 2);
        assert_eq!(events.pending().len(), 2);
        assert_eq!(commands.drain().len(), 2);
        assert_eq!(events.drain().len(), 2);
    }

    #[test]
    fn scene_plugin_transition_service_defaults_to_empty() {
        let transitions = SceneTransitionService::default();
        assert_eq!(transitions.snapshot().transition_ids.len(), 0);
    }

    #[test]
    fn queues_domain_scene_commands() {
        let commands = SceneCommandQueue::default();

        commands.submit(SceneCommand::SpawnNamedEntity {
            name: "playground-2d-sprite".to_owned(),
            transform: Some(Transform3::default()),
        });

        commands.submit(SceneCommand::QueueSprite2d {
            command: Sprite2dSceneCommand::new(
                "playground-2d",
                "playground-2d-sprite",
                AssetKey::new("playground-2d/textures/sprite-lab"),
                Vec2::new(128.0, 128.0),
            ),
        });
        commands.submit(SceneCommand::QueueText2d {
            command: Text2dSceneCommand::new(
                "playground-2d",
                "playground-2d-label",
                "AMIGO 2D",
                AssetKey::new("playground-2d/fonts/debug-ui"),
                Vec2::new(320.0, 64.0),
            ),
        });
        commands.submit(SceneCommand::QueueTileMap2d {
            command: TileMap2dSceneCommand::new(
                "playground-sidescroller",
                "playground-sidescroller-tilemap",
                AssetKey::new("playground-sidescroller/tilesets/platformer"),
                Vec2::new(16.0, 16.0),
                vec!["....".to_owned(), "####".to_owned()],
            ),
        });
        commands.submit(SceneCommand::QueueKinematicBody2d {
            command: KinematicBody2dSceneCommand::new(
                "playground-sidescroller",
                "playground-sidescroller-player",
                Vec2::ZERO,
                1.0,
                720.0,
            ),
        });
        commands.submit(SceneCommand::QueueAabbCollider2d {
            command: AabbCollider2dSceneCommand::new(
                "playground-sidescroller",
                "playground-sidescroller-player",
                Vec2::new(20.0, 30.0),
                Vec2::new(0.0, 1.0),
                "player",
                vec!["world".to_owned(), "trigger".to_owned()],
            ),
        });
        commands.submit(SceneCommand::QueueTrigger2d {
            command: Trigger2dSceneCommand::new(
                "playground-sidescroller",
                "playground-sidescroller-coin",
                Vec2::new(16.0, 16.0),
                Vec2::ZERO,
                "trigger",
                vec!["player".to_owned()],
                Some("coin.collected".to_owned()),
            ),
        });
        commands.submit(SceneCommand::queue_motion_controller(
            MotionController2dSceneCommand::new(
                "playground-sidescroller",
                "playground-sidescroller-player",
                180.0,
                900.0,
                1200.0,
                500.0,
                900.0,
                -360.0,
                720.0,
            ),
        ));
        commands.submit(SceneCommand::QueueTileMapMarker2d {
            command: TileMapMarker2dSceneCommand::new(
                "playground-sidescroller",
                "playground-sidescroller-player",
                Some("playground-sidescroller-tilemap".to_owned()),
                "P",
                0,
                Vec2::new(0.0, 8.0),
            ),
        });
        commands.submit(SceneCommand::QueueMesh3d {
            command: Mesh3dSceneCommand::new(
                "playground-3d",
                "playground-3d-probe",
                AssetKey::new("playground-3d/meshes/probe"),
            ),
        });
        commands.submit(SceneCommand::QueueMaterial3d {
            command: Material3dSceneCommand::new(
                "playground-3d",
                "playground-3d-probe",
                "debug-surface",
                Some(AssetKey::new("playground-3d/materials/debug-surface")),
            ),
        });

        assert_eq!(commands.pending().len(), 11);
        assert_eq!(commands.drain().len(), 11);
    }

    #[test]
    fn motion_controller_commands_use_canonical_surface() {
        let command = MotionController2dSceneCommand::new(
            "playground-sidescroller",
            "playground-sidescroller-player",
            180.0,
            900.0,
            1200.0,
            500.0,
            900.0,
            -360.0,
            720.0,
        );

        let motion_command = SceneCommand::queue_motion_controller(command.clone());

        assert!(motion_command.is_motion_controller_command());
        assert_eq!(
            motion_command
                .motion_controller_command()
                .expect("motion command should be available")
                .entity_name,
            "playground-sidescroller-player"
        );
        assert_eq!(
            format_scene_command(&motion_command),
            "scene.2d.motion(playground-sidescroller-player, max_speed=180, jump_velocity=-360)"
        );
    }

    #[test]
    fn motion_controller_events_use_canonical_lookup() {
        let motion_event = SceneEvent::motion_controller_queued(7, "player");

        assert_eq!(motion_event.motion_controller_entity_name(), Some("player"));
    }
}
