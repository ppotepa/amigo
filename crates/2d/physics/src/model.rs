use amigo_math::Vec2;
use amigo_scene::SceneEntityId;

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
    pub source: amigo_scene::EntitySelector,
    pub target: amigo_scene::EntitySelector,
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
    pub body_states: std::collections::BTreeMap<String, PhysicsBodyState2d>,
    pub active_trigger_overlaps: std::collections::BTreeSet<(String, String)>,
    pub active_collision_rule_overlaps: std::collections::BTreeSet<(String, String, String)>,
}

pub type PhysicsWorld2D = PhysicsWorld2d;
