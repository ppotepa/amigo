use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

use amigo_math::Vec2;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_scene::{
    AabbCollider2dSceneCommand, CircleCollider2dSceneCommand,
    CollisionEventRule2dSceneCommand as SceneCollisionEventRule2dCommand, EntityPoolSceneService,
    EntitySelector, KinematicBody2dSceneCommand as SceneKinematicBody2dCommand, SceneEntityId,
    SceneService, Trigger2dSceneCommand as SceneTrigger2dSceneCommand,
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
pub struct CircleCollider2d {
    pub radius: f32,
    pub offset: Vec2,
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
pub struct CircleCollider2dCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub collider: CircleCollider2d,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionEventRule2d {
    pub id: String,
    pub source: EntitySelector,
    pub target: EntitySelector,
    pub event: String,
    pub once_per_overlap: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionEventRule2dCommand {
    pub source_mod: String,
    pub rule: CollisionEventRule2d,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionEvent2d {
    pub rule_id: String,
    pub topic: String,
    pub source_entity: String,
    pub target_entity: String,
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
    pub circle_colliders: Vec<CircleCollider2dCommand>,
    pub static_colliders: Vec<StaticCollider2dCommand>,
    pub triggers: Vec<Trigger2dCommand>,
    pub collision_event_rules: Vec<CollisionEventRule2dCommand>,
    pub body_states: BTreeMap<String, PhysicsBodyState2d>,
    pub active_trigger_overlaps: BTreeSet<(String, String)>,
    pub active_collision_rule_overlaps: BTreeSet<(String, String, String)>,
}

pub type PhysicsWorld2D = PhysicsWorld2d;

#[derive(Debug, Default)]
struct Physics2dState {
    kinematic_bodies: BTreeMap<String, KinematicBody2dCommand>,
    aabb_colliders: BTreeMap<String, AabbCollider2dCommand>,
    circle_colliders: BTreeMap<String, CircleCollider2dCommand>,
    static_colliders: Vec<StaticCollider2dCommand>,
    triggers: Vec<Trigger2dCommand>,
    collision_event_rules: BTreeMap<String, CollisionEventRule2dCommand>,
    body_states: BTreeMap<String, PhysicsBodyState2d>,
    active_trigger_overlaps: BTreeSet<(String, String)>,
    active_collision_rule_overlaps: BTreeSet<(String, String, String)>,
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

    pub fn queue_circle_collider(&self, command: CircleCollider2dCommand) {
        self.state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned")
            .circle_colliders
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

    pub fn queue_collision_event_rule(&self, command: CollisionEventRule2dCommand) {
        self.state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned")
            .collision_event_rules
            .insert(command.rule.id.clone(), command);
    }

    pub fn clear(&self) {
        let mut state = self
            .state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned");
        state.kinematic_bodies.clear();
        state.aabb_colliders.clear();
        state.circle_colliders.clear();
        state.static_colliders.clear();
        state.triggers.clear();
        state.collision_event_rules.clear();
        state.body_states.clear();
        state.active_trigger_overlaps.clear();
        state.active_collision_rule_overlaps.clear();
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

    pub fn circle_colliders(&self) -> Vec<CircleCollider2dCommand> {
        let state = self
            .state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned");
        state.circle_colliders.values().cloned().collect()
    }

    pub fn triggers(&self) -> Vec<Trigger2dCommand> {
        self.state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned")
            .triggers
            .clone()
    }

    pub fn collision_event_rules(&self) -> Vec<CollisionEventRule2dCommand> {
        self.state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned")
            .collision_event_rules
            .values()
            .cloned()
            .collect()
    }

    pub fn world(&self) -> PhysicsWorld2d {
        let state = self
            .state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned");
        PhysicsWorld2d {
            kinematic_bodies: state.kinematic_bodies.values().cloned().collect(),
            aabb_colliders: state.aabb_colliders.values().cloned().collect(),
            circle_colliders: state.circle_colliders.values().cloned().collect(),
            static_colliders: state.static_colliders.clone(),
            triggers: state.triggers.clone(),
            collision_event_rules: state.collision_event_rules.values().cloned().collect(),
            body_states: state.body_states.clone(),
            active_trigger_overlaps: state.active_trigger_overlaps.clone(),
            active_collision_rule_overlaps: state.active_collision_rule_overlaps.clone(),
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

    pub fn circle_collider(&self, entity_name: &str) -> Option<CircleCollider2dCommand> {
        self.state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned")
            .circle_colliders
            .get(entity_name)
            .cloned()
    }

    pub fn set_circle_radius(&self, entity_name: &str, radius: f32) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned");
        let Some(collider) = state.circle_colliders.get_mut(entity_name) else {
            return false;
        };
        collider.collider.radius = radius.max(0.0);
        true
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

    fn is_collision_rule_overlap_active(
        &self,
        rule_id: &str,
        source_entity: &str,
        target_entity: &str,
    ) -> bool {
        self.state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned")
            .active_collision_rule_overlaps
            .contains(&(
                rule_id.to_owned(),
                source_entity.to_owned(),
                target_entity.to_owned(),
            ))
    }

    fn set_collision_rule_overlap_active(
        &self,
        rule_id: &str,
        source_entity: &str,
        target_entity: &str,
        active: bool,
    ) {
        let mut state = self
            .state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned");
        let key = (
            rule_id.to_owned(),
            source_entity.to_owned(),
            target_entity.to_owned(),
        );
        if active {
            state.active_collision_rule_overlaps.insert(key);
        } else {
            state.active_collision_rule_overlaps.remove(&key);
        }
    }

    pub fn entity_names(&self) -> Vec<String> {
        let state = self
            .state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned");
        let mut entity_names = state.kinematic_bodies.keys().cloned().collect::<Vec<_>>();
        entity_names.extend(state.aabb_colliders.keys().cloned());
        entity_names.extend(state.circle_colliders.keys().cloned());
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

pub fn queue_circle_collider_scene_command(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
    command: &CircleCollider2dSceneCommand,
) -> SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    physics_scene_service.queue_circle_collider(CircleCollider2dCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        collider: CircleCollider2d {
            radius: command.radius.max(0.0),
            offset: command.offset,
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

pub fn queue_collision_event_rule_scene_command(
    physics_scene_service: &Physics2dSceneService,
    command: &SceneCollisionEventRule2dCommand,
) {
    physics_scene_service.queue_collision_event_rule(CollisionEventRule2dCommand {
        source_mod: command.source_mod.clone(),
        rule: CollisionEventRule2d {
            id: command.id.clone(),
            source: command.source.clone(),
            target: command.target.clone(),
            event: command.event.clone(),
            once_per_overlap: command.once_per_overlap,
        },
    });
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

pub fn circle_colliders_overlap(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
    left_entity_name: &str,
    right_entity_name: &str,
) -> bool {
    if !entity_eligible_for_collision(scene_service, left_entity_name)
        || !entity_eligible_for_collision(scene_service, right_entity_name)
    {
        return false;
    }
    let Some(left_collider) = physics_scene_service.circle_collider(left_entity_name) else {
        return false;
    };
    let Some(right_collider) = physics_scene_service.circle_collider(right_entity_name) else {
        return false;
    };
    let Some(left_transform) = scene_service.transform_of(left_entity_name) else {
        return false;
    };
    let Some(right_transform) = scene_service.transform_of(right_entity_name) else {
        return false;
    };

    let left_center = Vec2::new(
        left_transform.translation.x + left_collider.collider.offset.x,
        left_transform.translation.y + left_collider.collider.offset.y,
    );
    let right_center = Vec2::new(
        right_transform.translation.x + right_collider.collider.offset.x,
        right_transform.translation.y + right_collider.collider.offset.y,
    );
    let dx = right_center.x - left_center.x;
    let dy = right_center.y - left_center.y;
    let radius = left_collider.collider.radius + right_collider.collider.radius;

    dx * dx + dy * dy <= radius * radius
}

pub fn resolve_collision_candidates(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
    selector: &EntitySelector,
) -> Vec<String> {
    resolve_collision_candidates_with_pools(scene_service, physics_scene_service, None, selector)
}

pub fn resolve_collision_candidates_with_pools(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
    pool_scene_service: Option<&EntityPoolSceneService>,
    selector: &EntitySelector,
) -> Vec<String> {
    let names = match selector {
        EntitySelector::Entity(entity_name) => vec![entity_name.clone()],
        EntitySelector::Tag(tag) => scene_service.entities_by_tag(tag),
        EntitySelector::Group(group) => scene_service.entities_by_group(group),
        EntitySelector::Pool(pool) => pool_scene_service
            .map(|service| service.members(pool))
            .unwrap_or_default(),
    };

    names
        .into_iter()
        .filter(|entity_name| {
            entity_eligible_for_collision(scene_service, entity_name)
                && physics_scene_service.circle_collider(entity_name).is_some()
        })
        .collect()
}

pub fn first_overlap_by_selector(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
    source_entity_name: &str,
    selector: &EntitySelector,
) -> Option<String> {
    resolve_collision_candidates(scene_service, physics_scene_service, selector)
        .into_iter()
        .find(|candidate| {
            candidate != source_entity_name
                && circle_colliders_overlap(
                    scene_service,
                    physics_scene_service,
                    source_entity_name,
                    candidate,
                )
        })
}

pub fn first_overlap_by_selector_with_pools(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
    pool_scene_service: Option<&EntityPoolSceneService>,
    source_entity_name: &str,
    selector: &EntitySelector,
) -> Option<String> {
    resolve_collision_candidates_with_pools(
        scene_service,
        physics_scene_service,
        pool_scene_service,
        selector,
    )
    .into_iter()
    .find(|candidate| {
        candidate != source_entity_name
            && circle_colliders_overlap(
                scene_service,
                physics_scene_service,
                source_entity_name,
                candidate,
            )
    })
}

pub fn evaluate_collision_event_rules(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
) -> Vec<CollisionEvent2d> {
    evaluate_collision_event_rules_with_pools(scene_service, physics_scene_service, None)
}

pub fn evaluate_collision_event_rules_with_pools(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
    pool_scene_service: Option<&EntityPoolSceneService>,
) -> Vec<CollisionEvent2d> {
    let mut events = Vec::new();

    for command in physics_scene_service.collision_event_rules() {
        let rule = command.rule;
        let sources = resolve_collision_candidates_with_pools(
            scene_service,
            physics_scene_service,
            pool_scene_service,
            &rule.source,
        );
        let targets = resolve_collision_candidates_with_pools(
            scene_service,
            physics_scene_service,
            pool_scene_service,
            &rule.target,
        );
        let mut current_overlaps = BTreeSet::new();

        for source_entity in &sources {
            for target_entity in &targets {
                if source_entity == target_entity {
                    continue;
                }
                if !circle_colliders_overlap(
                    scene_service,
                    physics_scene_service,
                    source_entity,
                    target_entity,
                ) {
                    continue;
                }

                current_overlaps.insert((source_entity.clone(), target_entity.clone()));
                let was_active = physics_scene_service.is_collision_rule_overlap_active(
                    &rule.id,
                    source_entity,
                    target_entity,
                );
                if !rule.once_per_overlap || !was_active {
                    events.push(CollisionEvent2d {
                        rule_id: rule.id.clone(),
                        topic: rule.event.clone(),
                        source_entity: source_entity.clone(),
                        target_entity: target_entity.clone(),
                    });
                }
                physics_scene_service.set_collision_rule_overlap_active(
                    &rule.id,
                    source_entity,
                    target_entity,
                    true,
                );
            }
        }

        let active_overlaps = {
            let state = physics_scene_service
                .state
                .lock()
                .expect("physics2d scene service mutex should not be poisoned");
            state
                .active_collision_rule_overlaps
                .iter()
                .filter(|(rule_id, _, _)| rule_id == &rule.id)
                .cloned()
                .collect::<Vec<_>>()
        };

        for active in active_overlaps {
            let (_, source_entity, target_entity) = active;
            if !current_overlaps.contains(&(source_entity.clone(), target_entity.clone())) {
                physics_scene_service.set_collision_rule_overlap_active(
                    &rule.id,
                    &source_entity,
                    &target_entity,
                    false,
                );
            }
        }
    }

    events
}

fn entity_eligible_for_collision(scene_service: &SceneService, entity_name: &str) -> bool {
    scene_service
        .lifecycle_of(entity_name)
        .map(|lifecycle| lifecycle.simulation_enabled && lifecycle.collision_enabled)
        .unwrap_or(false)
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
        min: Vec2::new(center.x - half_size.x, center.y - half_size.y),
        max: Vec2::new(center.x + half_size.x, center.y + half_size.y),
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
        AabbCollider2d, AabbCollider2dCommand, CircleCollider2d, CircleCollider2dCommand,
        CollisionLayer, CollisionMask, KinematicBody2d, KinematicBody2dCommand,
        Physics2dSceneService, StaticCollider2d, StaticCollider2dCommand, Trigger2d,
        Trigger2dCommand, circle_colliders_overlap, evaluate_collision_event_rules,
        first_overlap_by_selector, move_and_collide, overlaps_trigger,
        overlaps_trigger_with_translation, queue_aabb_collider_scene_command,
        queue_circle_collider_scene_command, queue_collision_event_rule_scene_command,
        queue_kinematic_body_scene_command, queue_trigger_scene_command,
        resolve_collision_candidates, resolve_collision_candidates_with_pools,
    };
    use amigo_math::Vec2;
    use amigo_scene::{
        AabbCollider2dSceneCommand, CircleCollider2dSceneCommand, CollisionEventRule2dSceneCommand,
        EntityPoolSceneCommand, EntityPoolSceneService, EntitySelector,
        KinematicBody2dSceneCommand as SceneKinematicBody2dSceneCommand, SceneEntityId,
        SceneEntityLifecycle, SceneService, Trigger2dSceneCommand as SceneTrigger2dSceneCommand,
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
    fn queues_circle_collider_scene_commands() {
        let scene = SceneService::default();
        let service = Physics2dSceneService::default();

        let entity = queue_circle_collider_scene_command(
            &scene,
            &service,
            &CircleCollider2dSceneCommand::new("test-mod", "test-actor", 10.0, Vec2::new(0.0, 2.0)),
        );

        assert_eq!(
            service.circle_collider("test-actor"),
            Some(CircleCollider2dCommand {
                entity_id: entity,
                entity_name: "test-actor".to_owned(),
                collider: CircleCollider2d {
                    radius: 10.0,
                    offset: Vec2::new(0.0, 2.0),
                },
            })
        );
        assert_eq!(scene.entity_names(), vec!["test-actor".to_owned()]);
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

    #[test]
    fn circle_collider_overlap_uses_scene_transforms() {
        let scene = SceneService::default();
        scene.spawn("bullet");
        scene.spawn("asteroid");
        assert!(scene.set_transform(
            "bullet",
            amigo_math::Transform3 {
                translation: amigo_math::Vec3::new(10.0, 12.0, 0.0),
                ..Default::default()
            },
        ));
        assert!(scene.set_transform(
            "asteroid",
            amigo_math::Transform3 {
                translation: amigo_math::Vec3::new(20.0, 12.0, 0.0),
                ..Default::default()
            },
        ));

        let service = Physics2dSceneService::default();
        service.queue_circle_collider(CircleCollider2dCommand {
            entity_id: SceneEntityId::new(0),
            entity_name: "bullet".to_owned(),
            collider: CircleCollider2d {
                radius: 4.0,
                offset: Vec2::ZERO,
            },
        });
        service.queue_circle_collider(CircleCollider2dCommand {
            entity_id: SceneEntityId::new(1),
            entity_name: "asteroid".to_owned(),
            collider: CircleCollider2d {
                radius: 8.0,
                offset: Vec2::ZERO,
            },
        });

        assert!(circle_colliders_overlap(
            &scene, &service, "bullet", "asteroid"
        ));
        assert!(!circle_colliders_overlap(
            &scene, &service, "bullet", "missing"
        ));
    }

    #[test]
    fn set_circle_radius_updates_existing_circle_collider() {
        let service = Physics2dSceneService::default();
        queue_circle(&service, "asteroid", 1, 8.0);

        assert!(service.set_circle_radius("asteroid", 32.0));
        assert_eq!(
            service
                .circle_collider("asteroid")
                .expect("circle collider should exist")
                .collider
                .radius,
            32.0
        );
        assert!(service.set_circle_radius("asteroid", -4.0));
        assert_eq!(
            service
                .circle_collider("asteroid")
                .expect("circle collider should exist")
                .collider
                .radius,
            0.0
        );
        assert!(!service.set_circle_radius("missing", 12.0));
    }

    #[test]
    fn selector_queries_resolve_tag_and_group_candidates() {
        let scene = SceneService::default();
        scene.spawn("source");
        scene.spawn("tagged");
        scene.spawn("grouped");
        assert!(scene.configure_entity_metadata(
            "tagged",
            SceneEntityLifecycle::default(),
            vec!["target".to_owned()],
            Vec::new(),
            Default::default(),
        ));
        assert!(scene.configure_entity_metadata(
            "grouped",
            SceneEntityLifecycle::default(),
            Vec::new(),
            vec!["targets".to_owned()],
            Default::default(),
        ));
        assert!(scene.set_transform(
            "source",
            amigo_math::Transform3 {
                translation: amigo_math::Vec3::new(0.0, 0.0, 0.0),
                ..Default::default()
            },
        ));
        assert!(scene.set_transform(
            "tagged",
            amigo_math::Transform3 {
                translation: amigo_math::Vec3::new(8.0, 0.0, 0.0),
                ..Default::default()
            },
        ));
        assert!(scene.set_transform(
            "grouped",
            amigo_math::Transform3 {
                translation: amigo_math::Vec3::new(24.0, 0.0, 0.0),
                ..Default::default()
            },
        ));

        let service = Physics2dSceneService::default();
        queue_circle(&service, "source", 0, 5.0);
        queue_circle(&service, "tagged", 1, 5.0);
        queue_circle(&service, "grouped", 2, 5.0);

        assert_eq!(
            resolve_collision_candidates(&scene, &service, &EntitySelector::Tag("target".into())),
            vec!["tagged".to_owned()]
        );
        assert_eq!(
            resolve_collision_candidates(
                &scene,
                &service,
                &EntitySelector::Group("targets".into())
            ),
            vec!["grouped".to_owned()]
        );
        assert_eq!(
            first_overlap_by_selector(
                &scene,
                &service,
                "source",
                &EntitySelector::Tag("target".into())
            ),
            Some("tagged".to_owned())
        );
        assert_eq!(
            first_overlap_by_selector(
                &scene,
                &service,
                "source",
                &EntitySelector::Group("targets".into())
            ),
            None
        );
    }

    #[test]
    fn overlap_queries_ignore_simulation_or_collision_disabled_entities() {
        let scene = SceneService::default();
        scene.spawn("source");
        scene.spawn("collision-disabled");
        scene.spawn("simulation-disabled");
        assert!(scene.set_collision_enabled("collision-disabled", false));
        assert!(scene.set_simulation_enabled("simulation-disabled", false));

        let service = Physics2dSceneService::default();
        queue_circle(&service, "source", 0, 10.0);
        queue_circle(&service, "collision-disabled", 1, 10.0);
        queue_circle(&service, "simulation-disabled", 2, 10.0);

        assert!(!circle_colliders_overlap(
            &scene,
            &service,
            "source",
            "collision-disabled"
        ));
        assert!(!circle_colliders_overlap(
            &scene,
            &service,
            "source",
            "simulation-disabled"
        ));
    }

    #[test]
    fn collision_event_rule_publishes_once_and_reenters_after_separation() {
        let scene = SceneService::default();
        scene.spawn("source");
        scene.spawn("target");
        assert!(scene.set_transform(
            "target",
            amigo_math::Transform3 {
                translation: amigo_math::Vec3::new(8.0, 0.0, 0.0),
                ..Default::default()
            },
        ));

        let service = Physics2dSceneService::default();
        queue_circle(&service, "source", 0, 5.0);
        queue_circle(&service, "target", 1, 5.0);
        queue_collision_event_rule_scene_command(
            &service,
            &CollisionEventRule2dSceneCommand::new(
                "test-mod",
                "source-hits-target",
                EntitySelector::Entity("source".to_owned()),
                EntitySelector::Entity("target".to_owned()),
                "collision.hit",
                true,
            ),
        );

        let first = evaluate_collision_event_rules(&scene, &service);
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].source_entity, "source");
        assert_eq!(first[0].target_entity, "target");

        assert!(evaluate_collision_event_rules(&scene, &service).is_empty());

        assert!(scene.set_transform(
            "target",
            amigo_math::Transform3 {
                translation: amigo_math::Vec3::new(100.0, 0.0, 0.0),
                ..Default::default()
            },
        ));
        assert!(evaluate_collision_event_rules(&scene, &service).is_empty());

        assert!(scene.set_transform(
            "target",
            amigo_math::Transform3 {
                translation: amigo_math::Vec3::new(8.0, 0.0, 0.0),
                ..Default::default()
            },
        ));
        assert_eq!(evaluate_collision_event_rules(&scene, &service).len(), 1);
    }

    #[test]
    fn pool_selector_is_deferred_until_pool_service_exists() {
        let scene = SceneService::default();
        let service = Physics2dSceneService::default();

        assert!(
            resolve_collision_candidates(
                &scene,
                &service,
                &EntitySelector::Pool("projectiles".to_owned())
            )
            .is_empty()
        );
    }

    #[test]
    fn pool_selector_resolves_members_when_pool_service_is_provided() {
        let scene = SceneService::default();
        scene.spawn("projectile-a");
        scene.spawn("projectile-b");
        let service = Physics2dSceneService::default();
        queue_circle(&service, "projectile-a", 0, 5.0);
        queue_circle(&service, "projectile-b", 1, 5.0);
        let pools = EntityPoolSceneService::default();
        pools.queue(EntityPoolSceneCommand::new(
            "test",
            "projectiles",
            vec!["projectile-a".to_owned(), "projectile-b".to_owned()],
        ));

        assert_eq!(
            resolve_collision_candidates_with_pools(
                &scene,
                &service,
                Some(&pools),
                &EntitySelector::Pool("projectiles".to_owned())
            ),
            vec!["projectile-a".to_owned(), "projectile-b".to_owned()]
        );
    }

    fn queue_circle(
        service: &Physics2dSceneService,
        entity_name: impl Into<String>,
        entity_id: u64,
        radius: f32,
    ) {
        service.queue_circle_collider(CircleCollider2dCommand {
            entity_id: SceneEntityId::new(entity_id),
            entity_name: entity_name.into(),
            collider: CircleCollider2d {
                radius,
                offset: Vec2::ZERO,
            },
        });
    }
}
