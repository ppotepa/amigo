use std::collections::BTreeMap;

use rapier3d::prelude::*;

use crate::model::{
    BoxCollider3dCommand, PhysicsBodyState3d, PhysicsSpawner3dCommand, PhysicsSpawner3dState,
    PhysicsWorld3d, RigidBody3dCommand, StaticBoxCollider3dCommand,
};

pub(crate) struct Physics3dState {
    pub(crate) rigid_bodies: BTreeMap<String, RigidBody3dCommand>,
    pub(crate) box_colliders: BTreeMap<String, BoxCollider3dCommand>,
    pub(crate) static_box_colliders: Vec<StaticBoxCollider3dCommand>,
    pub(crate) body_states: BTreeMap<String, PhysicsBodyState3d>,
    pub(crate) spawners: BTreeMap<String, PhysicsSpawner3dCommand>,
    pub(crate) spawner_states: BTreeMap<String, PhysicsSpawner3dState>,
    pub(crate) world: PhysicsWorld3d,
    pub(crate) rapier: Physics3dRapierWorld,
    pub(crate) rigid_body_handles: BTreeMap<String, RigidBodyHandle>,
    pub(crate) collider_handles: BTreeMap<String, Vec<ColliderHandle>>,
}

impl Default for Physics3dState {
    fn default() -> Self {
        Self {
            rigid_bodies: BTreeMap::new(),
            box_colliders: BTreeMap::new(),
            static_box_colliders: Vec::new(),
            body_states: BTreeMap::new(),
            spawners: BTreeMap::new(),
            spawner_states: BTreeMap::new(),
            world: PhysicsWorld3d::default(),
            rapier: Physics3dRapierWorld::default(),
            rigid_body_handles: BTreeMap::new(),
            collider_handles: BTreeMap::new(),
        }
    }
}

pub(crate) struct Physics3dRapierWorld {
    pub(crate) pipeline: PhysicsPipeline,
    pub(crate) gravity: Vector,
    pub(crate) integration_parameters: IntegrationParameters,
    pub(crate) island_manager: IslandManager,
    pub(crate) broad_phase: BroadPhaseBvh,
    pub(crate) narrow_phase: NarrowPhase,
    pub(crate) rigid_bodies: RigidBodySet,
    pub(crate) colliders: ColliderSet,
    pub(crate) impulse_joints: ImpulseJointSet,
    pub(crate) multibody_joints: MultibodyJointSet,
    pub(crate) ccd_solver: CCDSolver,
}

impl Default for Physics3dRapierWorld {
    fn default() -> Self {
        Self {
            pipeline: PhysicsPipeline::new(),
            gravity: Vector::new(0.0, -9.81, 0.0),
            integration_parameters: IntegrationParameters::default(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhaseBvh::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
        }
    }
}
