use std::sync::Arc;

use amigo_2d_motion::{projectile_launch_2d, Motion2dSceneService};
use amigo_2d_physics::Physics2dSceneService;
use amigo_scene::{EntityPoolSceneService, LifetimeSceneService, SceneService};

#[derive(Clone)]
pub struct ProjectilesApi {
    pub(crate) scene: Option<Arc<SceneService>>,
    pub(crate) motion_scene: Option<Arc<Motion2dSceneService>>,
    pub(crate) physics_scene: Option<Arc<Physics2dSceneService>>,
    pub(crate) pools: Option<Arc<EntityPoolSceneService>>,
    pub(crate) lifetimes: Option<Arc<LifetimeSceneService>>,
}

impl ProjectilesApi {
    pub fn fire_from(&mut self, emitter_id: &str, source_entity: &str) -> bool {
        let Some(scene) = self.scene.as_ref() else {
            return false;
        };
        let Some(motion_scene) = self.motion_scene.as_ref() else {
            return false;
        };
        let Some(pools) = self.pools.as_ref() else {
            return false;
        };

        let Some(command) = motion_scene.projectile_emitter(emitter_id) else {
            return false;
        };
        let Some(source_transform) = scene.transform_of(source_entity) else {
            return false;
        };

        let source_velocity = self
            .physics_scene
            .as_ref()
            .and_then(|service| service.body_state(source_entity))
            .map(|state| state.velocity)
            .unwrap_or_else(|| motion_scene.current_velocity(source_entity));
        let Some(projectile_entity) = pools.acquire(scene, &command.emitter.pool) else {
            return false;
        };
        let launch = projectile_launch_2d(source_transform, source_velocity, &command.emitter);
        let _ = scene.set_transform(&projectile_entity, launch.transform);
        let _ = motion_scene.set_velocity(&projectile_entity, launch.velocity);
        if let Some(lifetimes) = self.lifetimes.as_ref() {
            let _ = lifetimes.reset_lifetime(&projectile_entity);
        }
        if let Some(physics_scene) = self.physics_scene.as_ref() {
            if let Some(mut body_state) = physics_scene.body_state(&projectile_entity) {
                body_state.velocity = launch.velocity;
                let _ = physics_scene.sync_body_state(&projectile_entity, body_state);
            }
        }
        true
    }

    pub fn release(&mut self, pool_id: &str, entity_name: &str) -> bool {
        let Some(scene) = self.scene.as_ref() else {
            return false;
        };
        let Some(pools) = self.pools.as_ref() else {
            return false;
        };

        pools.release(scene, pool_id, entity_name)
    }
}
