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

#[derive(Debug, Clone, PartialEq)]
pub struct Sprite2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub texture: AssetKey,
    pub size: Vec2,
    pub sheet: Option<SpriteSheet2dSceneCommand>,
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
            transform: Transform2::default(),
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
    QueueText2d {
        command: Text2dSceneCommand,
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
    TextQueued {
        entity_id: u64,
        entity_name: String,
        font: AssetKey,
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
        registry.register(SceneCommandQueue::default())?;
        registry.register(SceneEventQueue::default())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        HydratedSceneSnapshot, HydratedSceneState, Material3dSceneCommand, Mesh3dSceneCommand,
        SceneCommand, SceneCommandQueue, SceneEvent, SceneEventQueue, SceneKey, SceneService,
        SceneTransitionService, Sprite2dSceneCommand, Text2dSceneCommand,
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

        assert_eq!(commands.pending().len(), 5);
        assert_eq!(commands.drain().len(), 5);
    }
}
