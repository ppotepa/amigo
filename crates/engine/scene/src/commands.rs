use std::collections::BTreeMap;

use amigo_assets::AssetKey;
use amigo_math::Transform3;

use crate::*;

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
    Plugin {
        command: PluginSceneCommand,
    },
    ReloadActiveScene,
    ClearEntities,
    SetPostFx2dStacks {
        stacks: Vec<amigo_render_api::ScopedPostFx2dStack>,
        lens_certification_reports: Vec<amigo_render_api::LensDroplets2dCertificationReport>,
    },
    ConfigureActivationSet {
        command: ActivationSetSceneCommand,
    },
    ActivateSet {
        id: String,
    },
}

impl SceneCommand {
    pub fn plugin(command: PluginSceneCommand) -> Self {
        Self::Plugin { command }
    }

    pub fn plugin_command(&self) -> Option<&PluginSceneCommand> {
        match self {
            Self::Plugin { command } => Some(command),
            _ => None,
        }
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
    InputActionMapQueued {
        entity_id: u64,
        entity_name: String,
        map_id: String,
    },
    BehaviorQueued {
        entity_id: u64,
        entity_name: String,
    },
    EventPipelineQueued {
        entity_id: u64,
        entity_name: String,
    },
    UiModelBindingsQueued {
        entity_id: u64,
        entity_name: String,
    },
    ScriptComponentQueued {
        entity_id: u64,
        entity_name: String,
        source_name: String,
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
    StaticColliderQueued {
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
    Camera2dQueued {
        entity_id: u64,
        entity_name: String,
        camera_id: String,
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

pub trait RuntimeSceneCommandHandler: Send + Sync {
    fn can_handle(&self, command: &SceneCommand) -> bool;
    fn handle(
        &self,
        runtime: &amigo_runtime::Runtime,
        command: SceneCommand,
    ) -> amigo_core::AmigoResult<()>;
}

pub type RuntimeSceneCommandHandlerRegistry =
    amigo_runtime::HandlerRegistry<dyn RuntimeSceneCommandHandler>;

pub fn register_runtime_scene_command_handler<H>(
    registry: &RuntimeSceneCommandHandlerRegistry,
    handler: H,
) where
    H: RuntimeSceneCommandHandler + 'static,
{
    registry.register_arc(std::sync::Arc::new(handler));
}

impl<T: RuntimeSceneCommandHandler + ?Sized> RuntimeSceneCommandHandler for Box<T> {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        (**self).can_handle(command)
    }

    fn handle(
        &self,
        runtime: &amigo_runtime::Runtime,
        command: SceneCommand,
    ) -> amigo_core::AmigoResult<()> {
        (**self).handle(runtime, command)
    }
}
