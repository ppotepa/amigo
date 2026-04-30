use std::sync::Arc;

use amigo_2d_physics::{
    circle_colliders_overlap, first_overlap_by_selector, resolve_collision_candidates,
    Physics2dSceneService,
};
use amigo_scene::{EntitySelector, SceneService};
use rhai::{Array, ImmutableString};

#[derive(Clone)]
pub struct PhysicsApi {
    pub(crate) scene: Option<Arc<SceneService>>,
    pub(crate) physics_scene: Option<Arc<Physics2dSceneService>>,
}

impl PhysicsApi {
    pub fn overlaps(&mut self, left_entity_name: &str, right_entity_name: &str) -> bool {
        let Some(scene) = self.scene.as_ref() else {
            return false;
        };
        let Some(physics_scene) = self.physics_scene.as_ref() else {
            return false;
        };
        if left_entity_name.is_empty() || right_entity_name.is_empty() {
            return false;
        }

        circle_colliders_overlap(scene, physics_scene, left_entity_name, right_entity_name)
    }

    pub fn first_overlap(&mut self, left_entity_name: &str, candidates: Array) -> String {
        if left_entity_name.is_empty() {
            return String::new();
        }

        for candidate in candidates {
            let candidate_name = if let Some(name) = candidate.clone().try_cast::<String>() {
                name
            } else if let Some(name) = candidate.try_cast::<ImmutableString>() {
                name.to_string()
            } else {
                continue;
            };

            if candidate_name.is_empty() {
                continue;
            }

            if self.overlaps(left_entity_name, &candidate_name) {
                return candidate_name;
            }
        }

        String::new()
    }

    pub fn first_overlap_index(&mut self, left_entity_name: &str, candidates: Array) -> rhai::INT {
        if left_entity_name.is_empty() {
            return -1;
        }

        for (index, candidate) in candidates.into_iter().enumerate() {
            let candidate_name = if let Some(name) = candidate.clone().try_cast::<String>() {
                name
            } else if let Some(name) = candidate.try_cast::<ImmutableString>() {
                name.to_string()
            } else {
                continue;
            };

            if candidate_name.is_empty() {
                continue;
            }

            if self.overlaps(left_entity_name, &candidate_name) {
                return index as rhai::INT;
            }
        }

        -1
    }

    pub fn first_overlap_by_tag(&mut self, left_entity_name: &str, tag: &str) -> String {
        self.first_overlap_by_selector(left_entity_name, "tag", tag)
    }

    pub fn first_overlap_by_group(&mut self, left_entity_name: &str, group: &str) -> String {
        self.first_overlap_by_selector(left_entity_name, "group", group)
    }

    pub fn first_overlap_by_selector(
        &mut self,
        left_entity_name: &str,
        selector_kind: &str,
        selector_value: &str,
    ) -> String {
        let Some(scene) = self.scene.as_ref() else {
            return String::new();
        };
        let Some(physics_scene) = self.physics_scene.as_ref() else {
            return String::new();
        };
        if left_entity_name.is_empty() {
            return String::new();
        }
        let Some(selector) = selector_from_parts(selector_kind, selector_value) else {
            return String::new();
        };

        first_overlap_by_selector(scene, physics_scene, left_entity_name, &selector)
            .unwrap_or_default()
    }

    pub fn overlaps_by_tag(&mut self, left_entity_name: &str, tag: &str) -> bool {
        !self.first_overlap_by_tag(left_entity_name, tag).is_empty()
    }

    pub fn overlaps_by_group(&mut self, left_entity_name: &str, group: &str) -> bool {
        !self
            .first_overlap_by_group(left_entity_name, group)
            .is_empty()
    }

    pub fn selector_candidates(&mut self, selector_kind: &str, selector_value: &str) -> Array {
        let Some(scene) = self.scene.as_ref() else {
            return Array::new();
        };
        let Some(physics_scene) = self.physics_scene.as_ref() else {
            return Array::new();
        };
        let Some(selector) = selector_from_parts(selector_kind, selector_value) else {
            return Array::new();
        };

        resolve_collision_candidates(scene, physics_scene, &selector)
            .into_iter()
            .map(Into::into)
            .collect()
    }
}

fn selector_from_parts(kind: &str, value: &str) -> Option<EntitySelector> {
    if value.is_empty() {
        return None;
    }

    match kind {
        "entity" => Some(EntitySelector::Entity(value.to_owned())),
        "tag" => Some(EntitySelector::Tag(value.to_owned())),
        "group" => Some(EntitySelector::Group(value.to_owned())),
        "pool" => Some(EntitySelector::Pool(value.to_owned())),
        _ => None,
    }
}
