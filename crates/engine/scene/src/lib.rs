mod document;
mod error;
mod hydration;
mod transition;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;

use amigo_assets::AssetKey;
use amigo_core::TypedId;
use amigo_math::{ColorRgba, Transform2, Transform3, Vec2, Vec3};
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

#[derive(Debug, Clone)]
pub struct SceneEntity {
    pub id: SceneEntityId,
    pub name: String,
    pub transform: Transform3,
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

#[derive(Debug, Clone, PartialEq)]
pub struct PlatformerController2dSceneCommand {
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

impl PlatformerController2dSceneCommand {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneUiTarget {
    ScreenSpace { layer: SceneUiLayer },
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
    pub style: SceneUiStyle,
    pub on_click: Option<SceneUiEventBinding>,
    pub children: Vec<SceneUiNode>,
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
    QueueKinematicBody2d {
        command: KinematicBody2dSceneCommand,
    },
    QueueAabbCollider2d {
        command: AabbCollider2dSceneCommand,
    },
    QueueTrigger2d {
        command: Trigger2dSceneCommand,
    },
    QueuePlatformerController2d {
        command: PlatformerController2dSceneCommand,
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
    KinematicBodyQueued {
        entity_id: u64,
        entity_name: String,
    },
    AabbColliderQueued {
        entity_id: u64,
        entity_name: String,
    },
    TriggerQueued {
        entity_id: u64,
        entity_name: String,
        topic: Option<String>,
    },
    PlatformerControllerQueued {
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
        registry.register(CameraFollow2dSceneService::default())?;
        registry.register(Parallax2dSceneService::default())?;
        registry.register(SceneCommandQueue::default())?;
        registry.register(SceneEventQueue::default())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        AabbCollider2dSceneCommand, CameraFollow2dSceneCommand, CameraFollow2dSceneService,
        HydratedSceneSnapshot, HydratedSceneState, KinematicBody2dSceneCommand,
        Material3dSceneCommand, Mesh3dSceneCommand, Parallax2dSceneCommand, Parallax2dSceneService,
        PlatformerController2dSceneCommand, SceneCommand, SceneCommandQueue, SceneEvent,
        SceneEventQueue, SceneKey, SceneService, SceneTransitionService, Sprite2dSceneCommand,
        Text2dSceneCommand, TileMap2dSceneCommand, TileMapMarker2dSceneCommand,
        Trigger2dSceneCommand,
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
        commands.submit(SceneCommand::QueuePlatformerController2d {
            command: PlatformerController2dSceneCommand::new(
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
        });
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
}
