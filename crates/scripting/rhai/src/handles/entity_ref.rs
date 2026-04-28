use std::sync::Arc;

use amigo_scene::SceneService;

use crate::bindings::entities::{
    entity_exists, rotate_entity_2d, rotate_entity_3d, set_entity_position_2d,
};

#[derive(Clone)]
pub struct EntityRef {
    scene: Option<Arc<SceneService>>,
    entity_name: String,
}

impl EntityRef {
    pub fn new(scene: Option<Arc<SceneService>>, entity_name: impl Into<String>) -> Self {
        Self {
            scene,
            entity_name: entity_name.into(),
        }
    }

    pub fn name(&mut self) -> String {
        self.entity_name.clone()
    }

    pub fn exists(&mut self) -> bool {
        entity_exists(self.scene.as_ref(), &self.entity_name)
    }

    pub fn rotate_2d(&mut self, delta_radians: rhai::FLOAT) -> bool {
        rotate_entity_2d(self.scene.as_ref(), &self.entity_name, delta_radians as f32)
    }

    pub fn rotate_3d(&mut self, x: rhai::FLOAT, y: rhai::FLOAT, z: rhai::FLOAT) -> bool {
        rotate_entity_3d(
            self.scene.as_ref(),
            &self.entity_name,
            x as f32,
            y as f32,
            z as f32,
        )
    }

    pub fn set_position_2d(&mut self, x: rhai::FLOAT, y: rhai::FLOAT) -> bool {
        set_entity_position_2d(self.scene.as_ref(), &self.entity_name, x as f32, y as f32)
    }
}
