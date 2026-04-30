use std::sync::Arc;

use amigo_math::Vec3;
use amigo_scene::{ScenePropertyValue, SceneService};

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

    pub fn distance(&mut self, left_entity: &str, right_entity: &str) -> rhai::FLOAT {
        entity_distance(self.scene.as_ref(), left_entity, right_entity)
    }

    pub fn set_position_2d(&mut self, entity_name: &str, x: rhai::FLOAT, y: rhai::FLOAT) -> bool {
        set_entity_position_2d(self.scene.as_ref(), entity_name, x as f32, y as f32)
    }

    pub fn hide(&mut self, entity_name: &str) -> bool {
        hide_entity(self.scene.as_ref(), entity_name)
    }

    pub fn show(&mut self, entity_name: &str) -> bool {
        show_entity(self.scene.as_ref(), entity_name)
    }

    pub fn enable(&mut self, entity_name: &str) -> bool {
        enable_entity(self.scene.as_ref(), entity_name)
    }

    pub fn disable(&mut self, entity_name: &str) -> bool {
        disable_entity(self.scene.as_ref(), entity_name)
    }

    pub fn set_collision_enabled(&mut self, entity_name: &str, enabled: bool) -> bool {
        set_entity_collision_enabled(self.scene.as_ref(), entity_name, enabled)
    }

    pub fn is_visible(&mut self, entity_name: &str) -> bool {
        is_entity_visible(self.scene.as_ref(), entity_name)
    }

    pub fn is_enabled(&mut self, entity_name: &str) -> bool {
        is_entity_enabled(self.scene.as_ref(), entity_name)
    }

    pub fn collision_enabled(&mut self, entity_name: &str) -> bool {
        is_entity_collision_enabled(self.scene.as_ref(), entity_name)
    }

    pub fn hide_many(&mut self, entity_names: rhai::Array) -> rhai::INT {
        hide_entities(self.scene.as_ref(), entity_names)
    }

    pub fn by_tag(&mut self, tag: &str) -> rhai::Array {
        entities_by_tag(self.scene.as_ref(), tag)
    }

    pub fn by_group(&mut self, group: &str) -> rhai::Array {
        entities_by_group(self.scene.as_ref(), group)
    }

    pub fn active_by_tag(&mut self, tag: &str) -> rhai::Array {
        active_entities_by_tag(self.scene.as_ref(), tag)
    }

    pub fn has_tag(&mut self, entity_name: &str, tag: &str) -> bool {
        entity_has_tag(self.scene.as_ref(), entity_name, tag)
    }

    pub fn has_group(&mut self, entity_name: &str, group: &str) -> bool {
        entity_has_group(self.scene.as_ref(), entity_name, group)
    }

    pub fn property(&mut self, entity_name: &str, key: &str) -> rhai::Dynamic {
        entity_property(self.scene.as_ref(), entity_name, key)
    }

    pub fn property_int(&mut self, entity_name: &str, key: &str) -> rhai::INT {
        entity_property_int(self.scene.as_ref(), entity_name, key)
    }

    pub fn property_float(&mut self, entity_name: &str, key: &str) -> rhai::FLOAT {
        entity_property_float(self.scene.as_ref(), entity_name, key)
    }

    pub fn property_bool(&mut self, entity_name: &str, key: &str) -> bool {
        entity_property_bool(self.scene.as_ref(), entity_name, key)
    }

    pub fn property_string(&mut self, entity_name: &str, key: &str) -> String {
        entity_property_string(self.scene.as_ref(), entity_name, key)
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

pub fn entity_distance(
    scene: Option<&Arc<SceneService>>,
    left_entity: &str,
    right_entity: &str,
) -> rhai::FLOAT {
    let Some(scene) = scene else {
        return -1.0;
    };
    let Some(left) = scene.transform_of(left_entity) else {
        return -1.0;
    };
    let Some(right) = scene.transform_of(right_entity) else {
        return -1.0;
    };

    let dx = right.translation.x - left.translation.x;
    let dy = right.translation.y - left.translation.y;
    let dz = right.translation.z - left.translation.z;
    ((dx * dx + dy * dy + dz * dz).sqrt()) as rhai::FLOAT
}

pub fn set_entity_position_2d(
    scene: Option<&Arc<SceneService>>,
    entity_name: &str,
    x: f32,
    y: f32,
) -> bool {
    let Some(scene) = scene else {
        return false;
    };
    let Some(mut transform) = scene.transform_of(entity_name) else {
        return false;
    };

    transform.translation.x = x;
    transform.translation.y = y;
    scene.set_transform(entity_name, transform)
}

pub fn hide_entity(scene: Option<&Arc<SceneService>>, entity_name: &str) -> bool {
    scene
        .map(|scene| scene.set_visible(entity_name, false))
        .unwrap_or(false)
}

pub fn show_entity(scene: Option<&Arc<SceneService>>, entity_name: &str) -> bool {
    scene
        .map(|scene| scene.set_visible(entity_name, true))
        .unwrap_or(false)
}

pub fn enable_entity(scene: Option<&Arc<SceneService>>, entity_name: &str) -> bool {
    scene
        .map(|scene| scene.set_simulation_enabled(entity_name, true))
        .unwrap_or(false)
}

pub fn disable_entity(scene: Option<&Arc<SceneService>>, entity_name: &str) -> bool {
    scene
        .map(|scene| scene.set_simulation_enabled(entity_name, false))
        .unwrap_or(false)
}

pub fn set_entity_collision_enabled(
    scene: Option<&Arc<SceneService>>,
    entity_name: &str,
    enabled: bool,
) -> bool {
    scene
        .map(|scene| scene.set_collision_enabled(entity_name, enabled))
        .unwrap_or(false)
}

pub fn is_entity_visible(scene: Option<&Arc<SceneService>>, entity_name: &str) -> bool {
    scene
        .map(|scene| scene.is_visible(entity_name))
        .unwrap_or(false)
}

pub fn is_entity_enabled(scene: Option<&Arc<SceneService>>, entity_name: &str) -> bool {
    scene
        .map(|scene| scene.is_simulation_enabled(entity_name))
        .unwrap_or(false)
}

pub fn is_entity_collision_enabled(scene: Option<&Arc<SceneService>>, entity_name: &str) -> bool {
    scene
        .map(|scene| scene.is_collision_enabled(entity_name))
        .unwrap_or(false)
}

pub fn hide_entities(scene: Option<&Arc<SceneService>>, entity_names: rhai::Array) -> rhai::INT {
    let mut hidden = 0;
    for entity_name in entity_names {
        let Ok(entity_name) = entity_name.into_string() else {
            continue;
        };
        if hide_entity(scene, &entity_name) {
            hidden += 1;
        }
    }

    hidden
}

pub fn entities_by_tag(scene: Option<&Arc<SceneService>>, tag: &str) -> rhai::Array {
    string_array(
        scene
            .map(|scene| scene.entities_by_tag(tag))
            .unwrap_or_default(),
    )
}

pub fn entities_by_group(scene: Option<&Arc<SceneService>>, group: &str) -> rhai::Array {
    string_array(
        scene
            .map(|scene| scene.entities_by_group(group))
            .unwrap_or_default(),
    )
}

pub fn active_entities_by_tag(scene: Option<&Arc<SceneService>>, tag: &str) -> rhai::Array {
    string_array(
        scene
            .map(|scene| scene.active_entities_by_tag(tag))
            .unwrap_or_default(),
    )
}

pub fn entity_has_tag(scene: Option<&Arc<SceneService>>, entity_name: &str, tag: &str) -> bool {
    scene
        .map(|scene| scene.has_tag(entity_name, tag))
        .unwrap_or(false)
}

pub fn entity_has_group(scene: Option<&Arc<SceneService>>, entity_name: &str, group: &str) -> bool {
    scene
        .map(|scene| scene.has_group(entity_name, group))
        .unwrap_or(false)
}

pub fn entity_property(
    scene: Option<&Arc<SceneService>>,
    entity_name: &str,
    key: &str,
) -> rhai::Dynamic {
    scene
        .and_then(|scene| scene.property_of(entity_name, key))
        .map(dynamic_from_property)
        .unwrap_or(rhai::Dynamic::UNIT)
}

pub fn entity_property_int(
    scene: Option<&Arc<SceneService>>,
    entity_name: &str,
    key: &str,
) -> rhai::INT {
    match scene.and_then(|scene| scene.property_of(entity_name, key)) {
        Some(ScenePropertyValue::Int(value)) => value as rhai::INT,
        Some(ScenePropertyValue::Float(value)) => value as rhai::INT,
        _ => 0,
    }
}

pub fn entity_property_float(
    scene: Option<&Arc<SceneService>>,
    entity_name: &str,
    key: &str,
) -> rhai::FLOAT {
    match scene.and_then(|scene| scene.property_of(entity_name, key)) {
        Some(ScenePropertyValue::Float(value)) => value as rhai::FLOAT,
        Some(ScenePropertyValue::Int(value)) => value as rhai::FLOAT,
        _ => 0.0,
    }
}

pub fn entity_property_bool(
    scene: Option<&Arc<SceneService>>,
    entity_name: &str,
    key: &str,
) -> bool {
    matches!(
        scene.and_then(|scene| scene.property_of(entity_name, key)),
        Some(ScenePropertyValue::Bool(true))
    )
}

pub fn entity_property_string(
    scene: Option<&Arc<SceneService>>,
    entity_name: &str,
    key: &str,
) -> String {
    match scene.and_then(|scene| scene.property_of(entity_name, key)) {
        Some(ScenePropertyValue::String(value)) => value,
        Some(ScenePropertyValue::Bool(value)) => value.to_string(),
        Some(ScenePropertyValue::Int(value)) => value.to_string(),
        Some(ScenePropertyValue::Float(value)) => value.to_string(),
        None => String::new(),
    }
}

fn dynamic_from_property(value: ScenePropertyValue) -> rhai::Dynamic {
    match value {
        ScenePropertyValue::Bool(value) => rhai::Dynamic::from(value),
        ScenePropertyValue::Int(value) => rhai::Dynamic::from(value as rhai::INT),
        ScenePropertyValue::Float(value) => rhai::Dynamic::from(value as rhai::FLOAT),
        ScenePropertyValue::String(value) => rhai::Dynamic::from(value),
    }
}
