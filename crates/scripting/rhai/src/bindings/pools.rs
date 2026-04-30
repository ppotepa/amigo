use std::sync::Arc;

use amigo_scene::{EntityPoolSceneService, LifetimeSceneService, SceneService};

use crate::bindings::common::string_array;

#[derive(Clone)]
pub struct PoolsApi {
    pub(crate) scene: Option<Arc<SceneService>>,
    pub(crate) pools: Option<Arc<EntityPoolSceneService>>,
    pub(crate) lifetimes: Option<Arc<LifetimeSceneService>>,
}

impl PoolsApi {
    pub fn acquire(&mut self, pool_id: &str) -> String {
        let Some(scene) = self.scene.as_ref() else {
            return String::new();
        };
        let Some(pools) = self.pools.as_ref() else {
            return String::new();
        };

        let entity_name = pools.acquire(scene, pool_id).unwrap_or_default();
        if !entity_name.is_empty() {
            if let Some(lifetimes) = self.lifetimes.as_ref() {
                let _ = lifetimes.reset_lifetime(&entity_name);
            }
        }
        entity_name
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

    pub fn members(&mut self, pool_id: &str) -> rhai::Array {
        string_array(
            self.pools
                .as_ref()
                .map(|pools| pools.members(pool_id))
                .unwrap_or_default(),
        )
    }

    pub fn active_members(&mut self, pool_id: &str) -> rhai::Array {
        string_array(
            self.pools
                .as_ref()
                .map(|pools| pools.active_members(pool_id))
                .unwrap_or_default(),
        )
    }
}
