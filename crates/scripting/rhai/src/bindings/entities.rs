use std::sync::Arc;

use amigo_math::Vec3;
use amigo_scene::SceneService;

use crate::bindings::common::string_array;
use crate::handles::EntityRef;

#[derive(Clone)]
pub struct EntitiesApi {
    pub(crate) scene: Option<Arc<SceneService>>,
}

impl EntitiesApi {
    pub fn named(&mut self, entity_name: &str) -> EntityRef {
        EntityRef::new(self.scene.clone(), entity_name)
    }

    pub fn create(&mut self, entity_name: &str) -> EntityRef {
        spawn_entity(self.scene.as_ref(), entity_name);
        EntityRef::new(self.scene.clone(), entity_name)
    }

    pub fn exists(&mut self, entity_name: &str) -> bool {
        entity_exists(self.scene.as_ref(), entity_name)
    }

    pub fn count(&mut self) -> rhai::INT {
        entity_count(self.scene.as_ref())
    }

    pub fn names(&mut self) -> rhai::Array {
        entity_names(self.scene.as_ref())
    }
}

pub fn spawn_entity(scene: Option<&Arc<SceneService>>, entity_name: &str) -> rhai::INT {
    scene
        .map(|scene| scene.spawn(entity_name).raw() as rhai::INT)
        .unwrap_or(-1)
}

pub fn entity_count(scene: Option<&Arc<SceneService>>) -> rhai::INT {
    scene
        .map(|scene| scene.entity_count() as rhai::INT)
        .unwrap_or(0)
}

pub fn entity_names(scene: Option<&Arc<SceneService>>) -> rhai::Array {
    string_array(scene.map(|scene| scene.entity_names()).unwrap_or_default())
}

pub fn rotate_entity_2d(
    scene: Option<&Arc<SceneService>>,
    entity_name: &str,
    delta_radians: f32,
) -> bool {
    scene
        .map(|scene| scene.rotate_entity_2d(entity_name, delta_radians))
        .unwrap_or(false)
}

pub fn rotate_entity_3d(
    scene: Option<&Arc<SceneService>>,
    entity_name: &str,
    x: f32,
    y: f32,
    z: f32,
) -> bool {
    scene
        .map(|scene| scene.rotate_entity_3d(entity_name, Vec3::new(x, y, z)))
        .unwrap_or(false)
}

pub fn entity_exists(scene: Option<&Arc<SceneService>>, entity_name: &str) -> bool {
    scene
        .and_then(|scene| scene.entity_by_name(entity_name))
        .is_some()
}
