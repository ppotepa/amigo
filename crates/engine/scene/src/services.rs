use std::collections::BTreeMap;
use std::sync::Mutex;

use amigo_math::{Transform3, Vec3};

use crate::*;

#[derive(Debug, Default)]
struct SceneState {
    next_id: u64,
    entities: BTreeMap<u64, SceneEntity>,
    selected_scene: Option<SceneKey>,
}

impl SceneState {
    pub fn spawn_with_transform(
        &mut self,
        name: impl Into<String>,
        transform: Transform3,
    ) -> SceneEntityId {
        let id = SceneEntityId::new(self.next_id);
        self.next_id += 1;

        let entity = SceneEntity {
            id,
            name: name.into(),
            transform,
            lifecycle: SceneEntityLifecycle::default(),
            tags: Vec::new(),
            groups: Vec::new(),
            properties: BTreeMap::new(),
        };

        self.entities.insert(id.raw(), entity);
        id
    }

    pub fn entities(&self) -> impl Iterator<Item = &SceneEntity> {
        self.entities.values()
    }

    pub fn entity_by_name(&self, name: &str) -> Option<&SceneEntity> {
        self.entities.values().find(|entity| entity.name == name)
    }

    pub fn entity_by_name_mut(&mut self, name: &str) -> Option<&mut SceneEntity> {
        self.entities
            .values_mut()
            .find(|entity| entity.name == name)
    }
}

#[derive(Debug, Default)]
pub struct SceneService {
    state: Mutex<SceneState>,
}

impl SceneService {
    pub fn spawn(&self, name: impl Into<String>) -> SceneEntityId {
        self.spawn_with_transform(name, Transform3::default())
    }

    pub fn spawn_with_transform(
        &self,
        name: impl Into<String>,
        transform: Transform3,
    ) -> SceneEntityId {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        state.spawn_with_transform(name, transform)
    }

    pub fn clear_entities(&self) {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        state.entities.clear();
    }

    pub fn remove_entities_by_name(&self, entity_names: &[String]) -> usize {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        let before = state.entities.len();

        state
            .entities
            .retain(|_, entity| !entity_names.iter().any(|name| name == &entity.name));

        before.saturating_sub(state.entities.len())
    }

    pub fn select_scene(&self, scene: SceneKey) {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        state.selected_scene = Some(scene);
    }

    pub fn selected_scene(&self) -> Option<SceneKey> {
        let state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        state.selected_scene.clone()
    }

    pub fn entity_count(&self) -> usize {
        let state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        state.entities.len()
    }

    pub fn entities(&self) -> Vec<SceneEntity> {
        let state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        state.entities().cloned().collect()
    }

    pub fn entity_by_name(&self, name: &str) -> Option<SceneEntity> {
        let state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        state.entity_by_name(name).cloned()
    }

    pub fn find_or_spawn_named_entity(&self, name: impl Into<String>) -> SceneEntityId {
        let name = name.into();
        self.entity_by_name(&name)
            .map(|entity| entity.id)
            .unwrap_or_else(|| self.spawn(name))
    }

    pub fn transform_of(&self, entity_name: &str) -> Option<Transform3> {
        self.entity_by_name(entity_name)
            .map(|entity| entity.transform)
    }

    pub fn set_transform(&self, entity_name: &str, transform: Transform3) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        let Some(entity) = state.entity_by_name_mut(entity_name) else {
            return false;
        };
        entity.transform = transform;
        true
    }

    pub fn lifecycle_of(&self, entity_name: &str) -> Option<SceneEntityLifecycle> {
        self.entity_by_name(entity_name)
            .map(|entity| entity.lifecycle)
    }

    pub fn set_lifecycle(&self, entity_name: &str, lifecycle: SceneEntityLifecycle) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        let Some(entity) = state.entity_by_name_mut(entity_name) else {
            return false;
        };
        entity.lifecycle = lifecycle;
        true
    }

    pub fn set_visible(&self, entity_name: &str, visible: bool) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        let Some(entity) = state.entity_by_name_mut(entity_name) else {
            return false;
        };
        entity.lifecycle.visible = visible;
        true
    }

    pub fn set_simulation_enabled(&self, entity_name: &str, enabled: bool) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        let Some(entity) = state.entity_by_name_mut(entity_name) else {
            return false;
        };
        entity.lifecycle.simulation_enabled = enabled;
        true
    }

    pub fn set_collision_enabled(&self, entity_name: &str, enabled: bool) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        let Some(entity) = state.entity_by_name_mut(entity_name) else {
            return false;
        };
        entity.lifecycle.collision_enabled = enabled;
        true
    }

    pub fn is_visible(&self, entity_name: &str) -> bool {
        self.lifecycle_of(entity_name)
            .map(|lifecycle| lifecycle.visible)
            .unwrap_or(false)
    }

    pub fn is_simulation_enabled(&self, entity_name: &str) -> bool {
        self.lifecycle_of(entity_name)
            .map(|lifecycle| lifecycle.simulation_enabled)
            .unwrap_or(false)
    }

    pub fn is_collision_enabled(&self, entity_name: &str) -> bool {
        self.lifecycle_of(entity_name)
            .map(|lifecycle| lifecycle.collision_enabled)
            .unwrap_or(false)
    }

    pub fn configure_entity_metadata(
        &self,
        entity_name: &str,
        lifecycle: SceneEntityLifecycle,
        tags: Vec<String>,
        groups: Vec<String>,
        properties: BTreeMap<String, ScenePropertyValue>,
    ) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        let Some(entity) = state.entity_by_name_mut(entity_name) else {
            return false;
        };
        entity.lifecycle = lifecycle;
        entity.tags = tags;
        entity.groups = groups;
        entity.properties = properties;
        true
    }

    pub fn tags_of(&self, entity_name: &str) -> Vec<String> {
        self.entity_by_name(entity_name)
            .map(|entity| entity.tags)
            .unwrap_or_default()
    }

    pub fn groups_of(&self, entity_name: &str) -> Vec<String> {
        self.entity_by_name(entity_name)
            .map(|entity| entity.groups)
            .unwrap_or_default()
    }

    pub fn has_tag(&self, entity_name: &str, tag: &str) -> bool {
        self.entity_by_name(entity_name)
            .map(|entity| entity.tags.iter().any(|value| value == tag))
            .unwrap_or(false)
    }

    pub fn has_group(&self, entity_name: &str, group: &str) -> bool {
        self.entity_by_name(entity_name)
            .map(|entity| entity.groups.iter().any(|value| value == group))
            .unwrap_or(false)
    }

    pub fn entities_by_tag(&self, tag: &str) -> Vec<String> {
        self.entities()
            .into_iter()
            .filter(|entity| entity.tags.iter().any(|value| value == tag))
            .map(|entity| entity.name)
            .collect()
    }

    pub fn entities_by_group(&self, group: &str) -> Vec<String> {
        self.entities()
            .into_iter()
            .filter(|entity| entity.groups.iter().any(|value| value == group))
            .map(|entity| entity.name)
            .collect()
    }

    pub fn active_entities_by_tag(&self, tag: &str) -> Vec<String> {
        self.entities()
            .into_iter()
            .filter(|entity| {
                entity.lifecycle.simulation_enabled && entity.tags.iter().any(|value| value == tag)
            })
            .map(|entity| entity.name)
            .collect()
    }

    pub fn property_of(&self, entity_name: &str, key: &str) -> Option<ScenePropertyValue> {
        self.entity_by_name(entity_name)
            .and_then(|entity| entity.properties.get(key).cloned())
    }

    pub fn set_property(
        &self,
        entity_name: &str,
        key: impl Into<String>,
        value: ScenePropertyValue,
    ) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        let Some(entity) = state.entity_by_name_mut(entity_name) else {
            return false;
        };
        entity.properties.insert(key.into(), value);
        true
    }

    pub fn rotate_entity_2d(&self, entity_name: &str, delta_radians: f32) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        let Some(entity) = state.entity_by_name_mut(entity_name) else {
            return false;
        };
        entity.transform.rotation_euler.z += delta_radians;
        true
    }

    pub fn set_entity_rotation_2d(&self, entity_name: &str, radians: f32) -> bool {
        if !radians.is_finite() {
            return false;
        }
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        let Some(entity) = state.entity_by_name_mut(entity_name) else {
            return false;
        };
        entity.transform.rotation_euler.z = radians;
        true
    }

    pub fn rotate_entity_3d(&self, entity_name: &str, delta_euler: Vec3) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("scene state mutex should not be poisoned");
        let Some(entity) = state.entity_by_name_mut(entity_name) else {
            return false;
        };
        entity.transform.rotation_euler.x += delta_euler.x;
        entity.transform.rotation_euler.y += delta_euler.y;
        entity.transform.rotation_euler.z += delta_euler.z;
        true
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.entities()
            .into_iter()
            .map(|entity| entity.name)
            .collect()
    }
}

