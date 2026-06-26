use std::sync::Arc;

use amigo_scene::{ScenePropertyValue, SceneService};

use crate::bindings::entities::{
    disable_entity, enable_entity, entity_exists, entity_has_group, entity_has_tag,
    entity_property, entity_property_bool, entity_property_float, entity_property_int,
    entity_property_string, hide_entity, is_entity_collision_enabled, is_entity_enabled,
    is_entity_visible, rotate_entity_2d, rotate_entity_3d, set_entity_collision_enabled,
    set_entity_position_2d, set_entity_position_3d, set_entity_property, set_entity_rotation_3d,
    show_entity,
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

    pub fn inspect_entity_name(&self) -> String {
        self.entity_name.clone()
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

    pub fn set_position_3d(&mut self, x: rhai::FLOAT, y: rhai::FLOAT, z: rhai::FLOAT) -> bool {
        set_entity_position_3d(
            self.scene.as_ref(),
            &self.entity_name,
            x as f32,
            y as f32,
            z as f32,
        )
    }

    pub fn set_rotation_3d(&mut self, x: rhai::FLOAT, y: rhai::FLOAT, z: rhai::FLOAT) -> bool {
        set_entity_rotation_3d(
            self.scene.as_ref(),
            &self.entity_name,
            x as f32,
            y as f32,
            z as f32,
        )
    }

    pub fn hide(&mut self) -> bool {
        hide_entity(self.scene.as_ref(), &self.entity_name)
    }

    pub fn show(&mut self) -> bool {
        show_entity(self.scene.as_ref(), &self.entity_name)
    }

    pub fn enable(&mut self) -> bool {
        enable_entity(self.scene.as_ref(), &self.entity_name)
    }

    pub fn disable(&mut self) -> bool {
        disable_entity(self.scene.as_ref(), &self.entity_name)
    }

    pub fn set_collision_enabled(&mut self, enabled: bool) -> bool {
        set_entity_collision_enabled(self.scene.as_ref(), &self.entity_name, enabled)
    }

    pub fn is_visible(&mut self) -> bool {
        is_entity_visible(self.scene.as_ref(), &self.entity_name)
    }

    pub fn is_enabled(&mut self) -> bool {
        is_entity_enabled(self.scene.as_ref(), &self.entity_name)
    }

    pub fn collision_enabled(&mut self) -> bool {
        is_entity_collision_enabled(self.scene.as_ref(), &self.entity_name)
    }

    pub fn has_tag(&mut self, tag: &str) -> bool {
        entity_has_tag(self.scene.as_ref(), &self.entity_name, tag)
    }

    pub fn has_group(&mut self, group: &str) -> bool {
        entity_has_group(self.scene.as_ref(), &self.entity_name, group)
    }

    pub fn property(&mut self, key: &str) -> rhai::Dynamic {
        entity_property(self.scene.as_ref(), &self.entity_name, key)
    }

    pub fn property_int(&mut self, key: &str) -> rhai::INT {
        entity_property_int(self.scene.as_ref(), &self.entity_name, key)
    }

    pub fn property_float(&mut self, key: &str) -> rhai::FLOAT {
        entity_property_float(self.scene.as_ref(), &self.entity_name, key)
    }

    pub fn property_bool(&mut self, key: &str) -> bool {
        entity_property_bool(self.scene.as_ref(), &self.entity_name, key)
    }

    pub fn property_string(&mut self, key: &str) -> String {
        entity_property_string(self.scene.as_ref(), &self.entity_name, key)
    }

    pub fn opacity(&mut self) -> rhai::FLOAT {
        self.property_float("opacity")
    }

    pub fn set_opacity(&mut self, value: rhai::FLOAT) {
        let _ = set_entity_property(
            self.scene.as_ref(),
            &self.entity_name,
            "opacity",
            ScenePropertyValue::Float(value as f64),
        );
    }

    pub fn visible(&mut self) -> bool {
        self.is_visible()
    }

    pub fn set_visible(&mut self, value: bool) {
        if value {
            let _ = self.show();
        } else {
            let _ = self.hide();
        }
    }

    pub fn enabled(&mut self) -> bool {
        self.is_enabled()
    }

    pub fn set_enabled(&mut self, value: bool) {
        if value {
            let _ = self.enable();
        } else {
            let _ = self.disable();
        }
    }

    pub fn collision(&mut self) -> bool {
        self.collision_enabled()
    }

    pub fn set_collision(&mut self, value: bool) {
        let _ = self.set_collision_enabled(value);
    }
}
