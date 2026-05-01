use std::sync::Mutex;

use crate::model::{
    AabbCollider2dCommand, CircleCollider2dCommand, CollisionEventRule2dCommand, GroundedState,
    KinematicBody2dCommand, PhysicsBodyState2d, PhysicsWorld2d, StaticCollider2dCommand,
    Trigger2dCommand,
};
use crate::registry::Physics2dState;

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

    pub fn circle_collider(&self, entity_name: &str) -> Option<CircleCollider2dCommand> {
        self.state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned")
            .circle_colliders
            .get(entity_name)
            .cloned()
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

    pub(crate) fn is_collision_rule_overlap_active(
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

    pub(crate) fn set_collision_rule_overlap_active(
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

    pub(crate) fn collision_rule_active_overlaps_for(
        &self,
        rule_id: &str,
    ) -> Vec<(String, String, String)> {
        self.state
            .lock()
            .expect("physics2d scene service mutex should not be poisoned")
            .active_collision_rule_overlaps
            .iter()
            .filter(|(id, _, _)| id == &rule_id)
            .cloned()
            .collect()
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
