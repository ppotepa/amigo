use std::collections::{BTreeMap, BTreeSet};

use crate::model::{
    AabbCollider2dCommand, CircleCollider2dCommand, CollisionEventRule2dCommand,
    KinematicBody2dCommand, PhysicsBodyState2d, StaticCollider2dCommand, Trigger2dCommand,
};

#[derive(Debug, Default)]
pub(crate) struct Physics2dState {
    pub(crate) kinematic_bodies: BTreeMap<String, KinematicBody2dCommand>,
    pub(crate) aabb_colliders: BTreeMap<String, AabbCollider2dCommand>,
    pub(crate) circle_colliders: BTreeMap<String, CircleCollider2dCommand>,
    pub(crate) static_colliders: Vec<StaticCollider2dCommand>,
    pub(crate) triggers: Vec<Trigger2dCommand>,
    pub(crate) collision_event_rules: BTreeMap<String, CollisionEventRule2dCommand>,
    pub(crate) body_states: BTreeMap<String, PhysicsBodyState2d>,
    pub(crate) active_trigger_overlaps: BTreeSet<(String, String)>,
    pub(crate) active_collision_rule_overlaps: BTreeSet<(String, String, String)>,
}
