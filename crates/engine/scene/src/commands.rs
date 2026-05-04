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
    QueueInputActionMap {
        command: InputActionMapSceneCommand,
    },
    QueueBehavior {
        command: BehaviorSceneCommand,
    },
    QueueEventPipeline {
        command: EventPipelineSceneCommand,
    },
    QueueUiModelBindings {
        command: UiModelBindingsSceneCommand,
    },
    QueueScriptComponent {
        command: ScriptComponentSceneCommand,
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
    QueueStaticCollider2d {
        command: StaticCollider2dSceneCommand,
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


