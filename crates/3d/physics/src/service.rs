use std::sync::Mutex;

use crate::model::{
    BoxCollider3dCommand, PhysicsBodyState3d, PhysicsSpawner3dCommand, PhysicsSpawner3dState,
    PhysicsWorld3d, RigidBody3dCommand, StaticBoxCollider3dCommand,
};
use crate::registry::Physics3dState;

#[derive(Default)]
pub struct Physics3dSceneService {
    state: Mutex<Physics3dState>,
}

impl Physics3dSceneService {
    pub fn queue_rigid_body(&self, command: RigidBody3dCommand) {
        let mut state = self
            .state
            .lock()
            .expect("physics3d scene service mutex should not be poisoned");
        state.body_states.insert(
            command.entity_name.clone(),
            PhysicsBodyState3d {
                velocity: command.body.velocity,
                angular_velocity: command.body.angular_velocity,
                grounded: false,
            },
        );
        state
            .rigid_bodies
            .insert(command.entity_name.clone(), command);
    }

    pub fn configure_world(&self, world: PhysicsWorld3d) {
        let mut state = self
            .state
            .lock()
            .expect("physics3d scene service mutex should not be poisoned");
        state.world = world;
    }

    pub fn queue_spawner(&self, command: PhysicsSpawner3dCommand) {
        let mut state = self
            .state
            .lock()
            .expect("physics3d scene service mutex should not be poisoned");
        state
            .spawner_states
            .entry(command.entity_name.clone())
            .or_default();
        state.spawners.insert(command.entity_name.clone(), command);
    }

    pub fn queue_box_collider(&self, command: BoxCollider3dCommand) {
        self.state
            .lock()
            .expect("physics3d scene service mutex should not be poisoned")
            .box_colliders
            .insert(command.entity_name.clone(), command);
    }

    pub fn queue_static_box_collider(&self, command: StaticBoxCollider3dCommand) {
        self.state
            .lock()
            .expect("physics3d scene service mutex should not be poisoned")
            .static_box_colliders
            .push(command);
    }

    pub fn clear(&self) {
        let mut state = self
            .state
            .lock()
            .expect("physics3d scene service mutex should not be poisoned");
        state.rigid_bodies.clear();
        state.box_colliders.clear();
        state.static_box_colliders.clear();
        state.body_states.clear();
        state.spawners.clear();
        state.spawner_states.clear();
        state.world = PhysicsWorld3d::default();
        state.rapier = crate::registry::Physics3dRapierWorld::default();
        state.rigid_body_handles.clear();
        state.collider_handles.clear();
    }

    pub fn rigid_bodies(&self) -> Vec<RigidBody3dCommand> {
        self.state
            .lock()
            .expect("physics3d scene service mutex should not be poisoned")
            .rigid_bodies
            .values()
            .cloned()
            .collect()
    }

    pub fn box_collider(&self, entity_name: &str) -> Option<BoxCollider3dCommand> {
        self.state
            .lock()
            .expect("physics3d scene service mutex should not be poisoned")
            .box_colliders
            .get(entity_name)
            .cloned()
    }

    pub fn static_box_colliders(&self) -> Vec<StaticBoxCollider3dCommand> {
        self.state
            .lock()
            .expect("physics3d scene service mutex should not be poisoned")
            .static_box_colliders
            .clone()
    }

    pub fn body_state(&self, entity_name: &str) -> Option<PhysicsBodyState3d> {
        self.state
            .lock()
            .expect("physics3d scene service mutex should not be poisoned")
            .body_states
            .get(entity_name)
            .cloned()
    }

    pub fn spawners(&self) -> Vec<PhysicsSpawner3dCommand> {
        self.state
            .lock()
            .expect("physics3d scene service mutex should not be poisoned")
            .spawners
            .values()
            .cloned()
            .collect()
    }

    pub fn spawner_state(&self, entity_name: &str) -> PhysicsSpawner3dState {
        self.state
            .lock()
            .expect("physics3d scene service mutex should not be poisoned")
            .spawner_states
            .get(entity_name)
            .cloned()
            .unwrap_or_default()
    }

    pub fn sync_spawner_state(
        &self,
        entity_name: &str,
        spawner_state: PhysicsSpawner3dState,
    ) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("physics3d scene service mutex should not be poisoned");
        if !state.spawners.contains_key(entity_name) {
            return false;
        }
        state
            .spawner_states
            .insert(entity_name.to_owned(), spawner_state);
        true
    }

    pub fn sync_body_state(&self, entity_name: &str, body_state: PhysicsBodyState3d) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("physics3d scene service mutex should not be poisoned");
        if !state.rigid_bodies.contains_key(entity_name) {
            return false;
        }
        if let Some(body) = state.rigid_bodies.get_mut(entity_name) {
            body.body.velocity = body_state.velocity;
            body.body.angular_velocity = body_state.angular_velocity;
        }
        state.body_states.insert(entity_name.to_owned(), body_state);
        true
    }

    pub(crate) fn with_state_mut<R>(&self, operation: impl FnOnce(&mut Physics3dState) -> R) -> R {
        let mut state = self
            .state
            .lock()
            .expect("physics3d scene service mutex should not be poisoned");
        operation(&mut state)
    }
}
