use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

use amigo_math::Vec2;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_scene::{
    AabbCollider2dSceneCommand, KinematicBody2dSceneCommand as SceneKinematicBody2dCommand,
    SceneEntityId, SceneService, Trigger2dSceneCommand as SceneTrigger2dSceneCommand,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionLayer(pub String);

impl CollisionLayer {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CollisionMask {
    pub layers: Vec<CollisionLayer>,
}

impl CollisionMask {
    pub fn new(layers: impl Into<Vec<CollisionLayer>>) -> Self {
        Self {
            layers: layers.into(),
        }
    }

    pub fn allows(&self, layer: &CollisionLayer) -> bool {
        self.layers.iter().any(|allowed| allowed == layer)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroundedState {
    pub grounded: bool,
    pub hit_wall: bool,
    pub hit_ceiling: bool,
}

impl Default for GroundedState {
    fn default() -> Self {
        Self {
            grounded: false,
            hit_wall: false,
            hit_ceiling: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsBodyState2d {
    pub velocity: Vec2,
    pub grounded: GroundedState,
}

impl Default for PhysicsBodyState2d {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            grounded: GroundedState::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KinematicBody2d {
    pub velocity: Vec2,
    pub gravity_scale: f32,
    pub terminal_velocity: f32,
}

impl Default for KinematicBody2d {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            gravity_scale: 1.0,
            terminal_velocity: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AabbCollider2d {
    pub size: Vec2,
    pub offset: Vec2,
    pub layer: CollisionLayer,
    pub mask: CollisionMask,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticCollider2d {
    pub size: Vec2,
    pub offset: Vec2,
    pub layer: CollisionLayer,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Trigger2d {
    pub size: Vec2,
    pub offset: Vec2,
    pub layer: CollisionLayer,
    pub mask: CollisionMask,
    pub topic: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollisionHit2d {
    pub entity_name: String,
    pub normal: Vec2,
    pub delta: Vec2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KinematicBody2dCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub body: KinematicBody2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AabbCollider2dCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub collider: AabbCollider2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticCollider2dCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub collider: StaticCollider2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Trigger2dCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub trigger: Trigger2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsStepResult2d {
    pub translation: Vec2,
    pub velocity: Vec2,
    pub grounded: GroundedState,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct PhysicsWorld2d {
    pub kinematic_bodies: Vec<KinematicBody2dCommand>,
    pub aabb_colliders: Vec<AabbCollider2dCommand>,
    pub static_colliders: Vec<StaticCollider2dCommand>,
    pub triggers: Vec<Trigger2dCommand>,
    pub body_states: BTreeMap<String, PhysicsBodyState2d>,
    pub active_trigger_overlaps: BTreeSet<(String, String)>,
}

pub type PhysicsWorld2D = PhysicsWorld2d;

#[derive(Debug, Default)]
struct Physics2dState {
    kinematic_bodies: BTreeMap<String, KinematicBody2dCommand>,
    aabb_colliders: BTreeMap<String, AabbCollider2dCommand>,
    static_colliders: Vec<StaticCollider2dCommand>,
    triggers: Vec<Trigger2dCommand>,
    body_states: BTreeMap<String, PhysicsBodyState2d>,
    active_trigger_overlaps: BTreeSet<(String, String)>,
}

#[derive(Debug, Default)]
pub struct Physics2dSceneService {
    state: Mutex<Physics2dState>,
}

impl Physics2dSceneService {
    pub fn queue_body(&self, command: KinematicBody2dCommand) {
        let mut state = self
            .state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned");
        state.body_states.insert(
            command.entity_name.clone(),
            PhysicsBodyState2d {
                velocity: command.body.velocity,
                grounded: GroundedState::default(),
            },
        );
        state
            .kinematic_bodies
            .insert(command.entity_name.clone(), command);
    }

    pub fn queue_aabb_collider(&self, command: AabbCollider2dCommand) {
        self.state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned")
            .aabb_colliders
            .insert(command.entity_name.clone(), command);
    }

    pub fn queue_static_collider(&self, command: StaticCollider2dCommand) {
        self.state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned")
            .static_colliders
            .push(command);
    }

    pub fn queue_trigger(&self, command: Trigger2dCommand) {
        self.state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned")
            .triggers
            .push(command);
    }

    pub fn clear(&self) {
        let mut state = self
            .state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned");
        state.kinematic_bodies.clear();
        state.aabb_colliders.clear();
        state.static_colliders.clear();
        state.triggers.clear();
        state.body_states.clear();
        state.active_trigger_overlaps.clear();
    }

    pub fn kinematic_bodies(&self) -> Vec<KinematicBody2dCommand> {
        let state = self
            .state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned");
        state.kinematic_bodies.values().cloned().collect()
    }

    pub fn static_colliders(&self) -> Vec<StaticCollider2dCommand> {
        self.state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned")
            .static_colliders
            .clone()
    }

    pub fn aabb_colliders(&self) -> Vec<AabbCollider2dCommand> {
        let state = self
            .state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned");
        state.aabb_colliders.values().cloned().collect()
    }

    pub fn triggers(&self) -> Vec<Trigger2dCommand> {
        self.state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned")
            .triggers
            .clone()
    }

    pub fn world(&self) -> PhysicsWorld2d {
        let state = self
            .state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned");
        PhysicsWorld2d {
            kinematic_bodies: state.kinematic_bodies.values().cloned().collect(),
            aabb_colliders: state.aabb_colliders.values().cloned().collect(),
            static_colliders: state.static_colliders.clone(),
            triggers: state.triggers.clone(),
            body_states: state.body_states.clone(),
            active_trigger_overlaps: state.active_trigger_overlaps.clone(),
        }
    }

    pub fn kinematic_body(&self, entity_name: &str) -> Option<KinematicBody2dCommand> {
        self.state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned")
            .kinematic_bodies
            .get(entity_name)
            .cloned()
    }

    pub fn aabb_collider(&self, entity_name: &str) -> Option<AabbCollider2dCommand> {
        self.state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned")
            .aabb_colliders
            .get(entity_name)
            .cloned()
    }

    pub fn body_state(&self, entity_name: &str) -> Option<PhysicsBodyState2d> {
        self.state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned")
            .body_states
            .get(entity_name)
            .cloned()
    }

    pub fn sync_body_state(&self, entity_name: &str, body_state: PhysicsBodyState2d) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned");
        if !state.kinematic_bodies.contains_key(entity_name) {
            return false;
        }
        if let Some(body) = state.kinematic_bodies.get_mut(entity_name) {
            body.body.velocity = body_state.velocity;
        }
        state.body_states.insert(entity_name.to_owned(), body_state);
        true
    }

    pub fn is_trigger_overlap_active(&self, trigger_name: &str, body_name: &str) -> bool {
        self.state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned")
            .active_trigger_overlaps
            .contains(&(trigger_name.to_owned(), body_name.to_owned()))
    }

    pub fn set_trigger_overlap_active(&self, trigger_name: &str, body_name: &str, active: bool) {
        let mut state = self
            .state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned");
        let key = (trigger_name.to_owned(), body_name.to_owned());
        if active {
            state.active_trigger_overlaps.insert(key);
        } else {
            state.active_trigger_overlaps.remove(&key);
        }
    }

    pub fn entity_names(&self) -> Vec<String> {
        let state = self
            .state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned");
        let mut entity_names = state.kinematic_bodies.keys().cloned().collect::<Vec<_>>();
        entity_names.extend(state.aabb_colliders.keys().cloned());
        entity_names.extend(
            state
                .static_colliders
                .iter()
                .map(|command| command.entity_name.clone()),
        );
        entity_names.extend(
            state
                .triggers
                .iter()
                .map(|command| command.entity_name.clone()),
        );
        entity_names.sort();
        entity_names.dedup();
        entity_names
    }
}

#[derive(Debug, Clone)]
pub struct Physics2dDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct Physics2dPlugin;

impl RuntimePlugin for Physics2dPlugin {
    fn name(&self) -> &'static str {
        "amigo-2d-physics"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(Physics2dSceneService::default())?;
        registry.register(Physics2dDomainInfo {
            crate_name: "amigo-2d-physics",
            capability: "physics_2d",
        })
    }
}

pub fn queue_kinematic_body_scene_command(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
    command: &SceneKinematicBody2dCommand,
) -> SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    physics_scene_service.queue_body(KinematicBody2dCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        body: KinematicBody2d {
            velocity: command.velocity,
            gravity_scale: command.gravity_scale,
            terminal_velocity: command.terminal_velocity,
        },
    });
    entity
}

pub fn queue_aabb_collider_scene_command(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
    command: &AabbCollider2dSceneCommand,
) -> SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    physics_scene_service.queue_aabb_collider(AabbCollider2dCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        collider: AabbCollider2d {
            size: command.size,
            offset: command.offset,
            layer: CollisionLayer::new(command.layer.clone()),
            mask: CollisionMask::new(
                command
                    .mask
                    .iter()
                    .cloned()
                    .map(CollisionLayer::new)
                    .collect::<Vec<_>>(),
            ),
        },
    });
    entity
}

pub fn queue_trigger_scene_command(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
    command: &SceneTrigger2dSceneCommand,
) -> SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    physics_scene_service.queue_trigger(Trigger2dCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        trigger: Trigger2d {
            size: command.size,
            offset: command.offset,
            layer: CollisionLayer::new(command.layer.clone()),
            mask: CollisionMask::new(
                command
                    .mask
                    .iter()
                    .cloned()
                    .map(CollisionLayer::new)
                    .collect::<Vec<_>>(),
            ),
            topic: command.event.clone(),
        },
    });
    entity
}

pub fn move_and_collide(
    translation: Vec2,
    collider: &AabbCollider2d,
    velocity: Vec2,
    delta_seconds: f32,
    static_colliders: &[StaticCollider2dCommand],
) -> PhysicsStepResult2d {
    let mut translation = translation;
    let mut velocity = velocity;
    let mut grounded = GroundedState::default();
    let delta = Vec2::new(velocity.x * delta_seconds, velocity.y * delta_seconds);
    let half_size = Vec2::new(collider.size.x * 0.5, collider.size.y * 0.5);

    translation.x += delta.x;
    for static_collider in static_colliders {
        if !collider.mask.allows(&static_collider.collider.layer) {
            continue;
        }
        if !intersects_rect(
            dynamic_rect(translation, collider.offset, half_size),
            static_rect(static_collider),
        ) {
            continue;
        }

        if delta.x > 0.0 {
            translation.x = static_rect(static_collider).min.x - half_size.x - collider.offset.x;
            grounded.hit_wall = true;
        } else if delta.x < 0.0 {
            translation.x = static_rect(static_collider).max.x + half_size.x - collider.offset.x;
            grounded.hit_wall = true;
        }
        velocity.x = 0.0;
    }

    translation.y += delta.y;
    for static_collider in static_colliders {
        if !collider.mask.allows(&static_collider.collider.layer) {
            continue;
        }
        if !intersects_rect(
            dynamic_rect(translation, collider.offset, half_size),
            static_rect(static_collider),
        ) {
            continue;
        }

        if delta.y < 0.0 {
            translation.y = static_rect(static_collider).max.y + half_size.y - collider.offset.y;
            grounded.grounded = true;
        } else if delta.y > 0.0 {
            translation.y = static_rect(static_collider).min.y - half_size.y - collider.offset.y;
            grounded.hit_ceiling = true;
        }
        velocity.y = 0.0;
    }

    PhysicsStepResult2d {
        translation,
        velocity,
        grounded,
    }
}

pub fn overlaps_trigger(
    translation: Vec2,
    collider: &AabbCollider2d,
    trigger: &Trigger2dCommand,
) -> bool {
    overlaps_trigger_with_translation(translation, collider, trigger, None)
}

pub fn overlaps_trigger_with_translation(
    translation: Vec2,
    collider: &AabbCollider2d,
    trigger: &Trigger2dCommand,
    trigger_translation: Option<Vec2>,
) -> bool {
    if !collider.mask.allows(&trigger.trigger.layer)
        || !trigger.trigger.mask.allows(&collider.layer)
    {
        return false;
    }

    intersects_rect(
        dynamic_rect(
            translation,
            collider.offset,
            Vec2::new(collider.size.x * 0.5, collider.size.y * 0.5),
        ),
        trigger_rect(trigger, trigger_translation),
    )
}

#[derive(Clone, Copy)]
struct Rect2d {
    min: Vec2,
    max: Vec2,
}

fn dynamic_rect(translation: Vec2, offset: Vec2, half_size: Vec2) -> Rect2d {
    let center = Vec2::new(translation.x + offset.x, translation.y + offset.y);
    Rect2d {
        min: Vec2::new(center.x - half_size.x, center.y - half_size.y),
        max: Vec2::new(center.x + half_size.x, center.y + half_size.y),
    }
}

fn static_rect(collider: &StaticCollider2dCommand) -> Rect2d {
    let half_size = Vec2::new(
        collider.collider.size.x * 0.5,
        collider.collider.size.y * 0.5,
    );
    Rect2d {
        min: Vec2::new(
            collider.collider.offset.x - half_size.x,
            collider.collider.offset.y - half_size.y,
        ),
        max: Vec2::new(
            collider.collider.offset.x + half_size.x,
            collider.collider.offset.y + half_size.y,
        ),
    }
}

fn trigger_rect(trigger: &Trigger2dCommand, translation: Option<Vec2>) -> Rect2d {
    let half_size = Vec2::new(trigger.trigger.size.x * 0.5, trigger.trigger.size.y * 0.5);
    let center = translation.unwrap_or(trigger.trigger.offset);
    Rect2d {
        min: Vec2::new(
            center.x - half_size.x,
            center.y - half_size.y,
        ),
        max: Vec2::new(
            center.x + half_size.x,
            center.y + half_size.y,
        ),
    }
}

fn intersects_rect(left: Rect2d, right: Rect2d) -> bool {
    left.min.x < right.max.x
        && left.max.x > right.min.x
        && left.min.y < right.max.y
        && left.max.y > right.min.y
}

#[cfg(test)]
mod tests {
    use super::{
        AabbCollider2d, AabbCollider2dCommand, CollisionLayer, CollisionMask, KinematicBody2d,
        KinematicBody2dCommand, Physics2dSceneService, StaticCollider2d, StaticCollider2dCommand,
        Trigger2d, Trigger2dCommand, move_and_collide, overlaps_trigger,
        overlaps_trigger_with_translation,
        queue_aabb_collider_scene_command, queue_kinematic_body_scene_command,
        queue_trigger_scene_command,
    };
    use amigo_math::Vec2;
    use amigo_scene::{
        AabbCollider2dSceneCommand, KinematicBody2dSceneCommand as SceneKinematicBody2dSceneCommand,
        SceneEntityId, SceneService, Trigger2dSceneCommand as SceneTrigger2dSceneCommand,
    };

    #[test]
    fn stores_physics2d_commands() {
        let service = Physics2dSceneService::default();

        service.queue_body(KinematicBody2dCommand {
            entity_id: SceneEntityId::new(1),
            entity_name: "player".to_owned(),
            body: KinematicBody2d {
                velocity: Vec2::new(10.0, -30.0),
                gravity_scale: 1.0,
                terminal_velocity: 720.0,
            },
        });
        service.queue_aabb_collider(AabbCollider2dCommand {
            entity_id: SceneEntityId::new(1),
            entity_name: "player".to_owned(),
            collider: AabbCollider2d {
                size: Vec2::new(20.0, 30.0),
                offset: Vec2::new(0.0, 1.0),
                layer: CollisionLayer::new("player"),
                mask: CollisionMask::new(vec![CollisionLayer::new("world")]),
            },
        });
        service.queue_static_collider(StaticCollider2dCommand {
            entity_id: SceneEntityId::new(2),
            entity_name: "ground".to_owned(),
            collider: StaticCollider2d {
                size: Vec2::new(64.0, 16.0),
                offset: Vec2::ZERO,
                layer: CollisionLayer::new("world"),
            },
        });
        service.queue_trigger(Trigger2dCommand {
            entity_id: SceneEntityId::new(3),
            entity_name: "coin".to_owned(),
            trigger: Trigger2d {
                size: Vec2::new(16.0, 16.0),
                offset: Vec2::ZERO,
                layer: CollisionLayer::new("trigger"),
                mask: CollisionMask::new(vec![CollisionLayer::new("player")]),
                topic: Some("coin.collected".to_owned()),
            },
        });

        assert_eq!(service.kinematic_bodies().len(), 1);
        assert_eq!(service.aabb_colliders().len(), 1);
        assert_eq!(service.static_colliders().len(), 1);
        assert_eq!(service.triggers().len(), 1);
        assert_eq!(
            service.entity_names(),
            vec!["coin".to_owned(), "ground".to_owned(), "player".to_owned()]
        );

        service.clear();
        assert!(service.kinematic_bodies().is_empty());
        assert!(service.aabb_colliders().is_empty());
        assert!(service.static_colliders().is_empty());
        assert!(service.triggers().is_empty());
    }

    #[test]
    fn queues_scene_commands_through_physics_helpers() {
        let scene = SceneService::default();
        let service = Physics2dSceneService::default();

        let body_entity = queue_kinematic_body_scene_command(
            &scene,
            &service,
            &SceneKinematicBody2dSceneCommand::new(
                "playground-sidescroller",
                "player",
                Vec2::new(4.0, -8.0),
                1.0,
                720.0,
            ),
        );
        let collider_entity = queue_aabb_collider_scene_command(
            &scene,
            &service,
            &AabbCollider2dSceneCommand::new(
                "playground-sidescroller",
                "player",
                Vec2::new(20.0, 30.0),
                Vec2::new(0.0, 1.0),
                "player",
                vec!["world".to_owned(), "trigger".to_owned()],
            ),
        );
        let trigger_entity = queue_trigger_scene_command(
            &scene,
            &service,
            &SceneTrigger2dSceneCommand::new(
                "playground-sidescroller",
                "coin",
                Vec2::new(16.0, 16.0),
                Vec2::ZERO,
                "trigger",
                vec!["player".to_owned()],
                Some("coin.collected".to_owned()),
            ),
        );

        assert_eq!(body_entity, collider_entity);
        assert_ne!(body_entity, trigger_entity);
        assert_eq!(service.kinematic_bodies().len(), 1);
        assert_eq!(service.aabb_colliders().len(), 1);
        assert_eq!(service.triggers().len(), 1);
        assert_eq!(
            scene.entity_names(),
            vec!["player".to_owned(), "coin".to_owned()]
        );
    }

    #[test]
    fn captures_physics_world_snapshot() {
        let service = Physics2dSceneService::default();
        service.queue_body(KinematicBody2dCommand {
            entity_id: SceneEntityId::new(1),
            entity_name: "player".to_owned(),
            body: KinematicBody2d {
                velocity: Vec2::new(4.0, -8.0),
                gravity_scale: 1.0,
                terminal_velocity: 720.0,
            },
        });
        service.queue_aabb_collider(AabbCollider2dCommand {
            entity_id: SceneEntityId::new(1),
            entity_name: "player".to_owned(),
            collider: AabbCollider2d {
                size: Vec2::new(20.0, 30.0),
                offset: Vec2::new(0.0, 1.0),
                layer: CollisionLayer::new("player"),
                mask: CollisionMask::new(vec![CollisionLayer::new("world")]),
            },
        });
        service.queue_static_collider(StaticCollider2dCommand {
            entity_id: SceneEntityId::new(2),
            entity_name: "ground".to_owned(),
            collider: StaticCollider2d {
                size: Vec2::new(64.0, 16.0),
                offset: Vec2::ZERO,
                layer: CollisionLayer::new("world"),
            },
        });
        service.queue_trigger(Trigger2dCommand {
            entity_id: SceneEntityId::new(3),
            entity_name: "coin".to_owned(),
            trigger: Trigger2d {
                size: Vec2::new(16.0, 16.0),
                offset: Vec2::ZERO,
                layer: CollisionLayer::new("trigger"),
                mask: CollisionMask::new(vec![CollisionLayer::new("player")]),
                topic: Some("coin.collected".to_owned()),
            },
        });
        service.set_trigger_overlap_active("coin", "player", true);

        let world = service.world();
        assert_eq!(world.kinematic_bodies.len(), 1);
        assert_eq!(world.aabb_colliders.len(), 1);
        assert_eq!(world.static_colliders.len(), 1);
        assert_eq!(world.triggers.len(), 1);
        assert_eq!(
            world.body_states.get("player").map(|state| state.velocity),
            Some(Vec2::new(4.0, -8.0))
        );
        assert!(
            world
                .active_trigger_overlaps
                .contains(&("coin".to_owned(), "player".to_owned()))
        );
    }

    #[test]
    fn body_falling_onto_static_collider_becomes_grounded() {
        let collider = AabbCollider2d {
            size: Vec2::new(20.0, 30.0),
            offset: Vec2::ZERO,
            layer: CollisionLayer::new("player"),
            mask: CollisionMask::new(vec![CollisionLayer::new("world")]),
        };
        let ground = StaticCollider2dCommand {
            entity_id: SceneEntityId::new(2),
            entity_name: "ground".to_owned(),
            collider: StaticCollider2d {
                size: Vec2::new(128.0, 16.0),
                offset: Vec2::new(64.0, 8.0),
                layer: CollisionLayer::new("world"),
            },
        };

        let result = move_and_collide(
            Vec2::new(64.0, 40.0),
            &collider,
            Vec2::new(0.0, -80.0),
            0.5,
            &[ground],
        );

        assert!(result.grounded.grounded);
        assert_eq!(result.velocity.y, 0.0);
        assert!(result.translation.y >= 31.0);
    }

    #[test]
    fn move_and_collide_blocks_horizontal_penetration() {
        let collider = AabbCollider2d {
            size: Vec2::new(20.0, 30.0),
            offset: Vec2::ZERO,
            layer: CollisionLayer::new("player"),
            mask: CollisionMask::new(vec![CollisionLayer::new("world")]),
        };
        let wall = StaticCollider2dCommand {
            entity_id: SceneEntityId::new(2),
            entity_name: "wall".to_owned(),
            collider: StaticCollider2d {
                size: Vec2::new(16.0, 96.0),
                offset: Vec2::new(40.0, 48.0),
                layer: CollisionLayer::new("world"),
            },
        };

        let result = move_and_collide(
            Vec2::new(10.0, 48.0),
            &collider,
            Vec2::new(120.0, 0.0),
            0.25,
            &[wall],
        );

        assert!(result.grounded.hit_wall);
        assert_eq!(result.velocity.x, 0.0);
        assert!(result.translation.x <= 22.0);
    }

    #[test]
    fn trigger_overlap_detects_matching_layers() {
        let collider = AabbCollider2d {
            size: Vec2::new(20.0, 30.0),
            offset: Vec2::ZERO,
            layer: CollisionLayer::new("player"),
            mask: CollisionMask::new(vec![CollisionLayer::new("trigger")]),
        };
        let trigger = Trigger2dCommand {
            entity_id: SceneEntityId::new(2),
            entity_name: "coin".to_owned(),
            trigger: Trigger2d {
                size: Vec2::new(16.0, 16.0),
                offset: Vec2::new(64.0, 64.0),
                layer: CollisionLayer::new("trigger"),
                mask: CollisionMask::new(vec![CollisionLayer::new("player")]),
                topic: Some("coin.collected".to_owned()),
            },
        };

        assert!(overlaps_trigger(Vec2::new(64.0, 64.0), &collider, &trigger));
        assert!(!overlaps_trigger(
            Vec2::new(120.0, 64.0),
            &collider,
            &trigger
        ));
    }

    #[test]
    fn trigger_overlap_uses_runtime_translation_override() {
        let collider = AabbCollider2d {
            size: Vec2::new(20.0, 30.0),
            offset: Vec2::ZERO,
            layer: CollisionLayer::new("player"),
            mask: CollisionMask::new(vec![CollisionLayer::new("trigger")]),
        };
        let trigger = Trigger2dCommand {
            entity_id: SceneEntityId::new(2),
            entity_name: "coin".to_owned(),
            trigger: Trigger2d {
                size: Vec2::new(16.0, 16.0),
                offset: Vec2::ZERO,
                layer: CollisionLayer::new("trigger"),
                mask: CollisionMask::new(vec![CollisionLayer::new("player")]),
                topic: Some("coin.collected".to_owned()),
            },
        };

        assert!(overlaps_trigger_with_translation(
            Vec2::new(64.0, 64.0),
            &collider,
            &trigger,
            Some(Vec2::new(64.0, 64.0)),
        ));
        assert!(!overlaps_trigger_with_translation(
            Vec2::new(64.0, 64.0),
            &collider,
            &trigger,
            Some(Vec2::new(-10000.0, -10000.0)),
        ));
    }
}
