use amigo_scene::{EntityPoolSceneService, EntitySelector, SceneService};

use crate::Physics2dSceneService;

pub fn resolve_collision_candidates(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
    selector: &EntitySelector,
) -> Vec<String> {
    resolve_collision_candidates_with_pools(scene_service, physics_scene_service, None, selector)
}

pub fn resolve_collision_candidates_with_pools(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
    pool_scene_service: Option<&EntityPoolSceneService>,
    selector: &EntitySelector,
) -> Vec<String> {
    let names = match selector {
        EntitySelector::Entity(entity_name) => vec![entity_name.clone()],
        EntitySelector::Tag(tag) => scene_service.entities_by_tag(tag),
        EntitySelector::Group(group) => scene_service.entities_by_group(group),
        EntitySelector::Pool(pool) => pool_scene_service
            .map(|service| service.members(pool))
            .unwrap_or_default(),
    };

    names
        .into_iter()
        .filter(|entity_name| {
            entity_eligible_for_collision(scene_service, entity_name)
                && physics_scene_service.circle_collider(entity_name).is_some()
        })
        .collect()
}

pub fn first_overlap_by_selector(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
    source_entity_name: &str,
    selector: &EntitySelector,
) -> Option<String> {
    resolve_collision_candidates(scene_service, physics_scene_service, selector)
        .into_iter()
        .find(|candidate| {
            candidate != source_entity_name
                && crate::simulation::circle_colliders_overlap(
                    scene_service,
                    physics_scene_service,
                    source_entity_name,
                    candidate,
                )
        })
}

pub fn first_overlap_by_selector_with_pools(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
    pool_scene_service: Option<&EntityPoolSceneService>,
    source_entity_name: &str,
    selector: &EntitySelector,
) -> Option<String> {
    resolve_collision_candidates_with_pools(
        scene_service,
        physics_scene_service,
        pool_scene_service,
        selector,
    )
    .into_iter()
    .find(|candidate| {
        candidate != source_entity_name
            && crate::simulation::circle_colliders_overlap(
                scene_service,
                physics_scene_service,
                source_entity_name,
                candidate,
            )
    })
}

pub fn entity_eligible_for_collision(scene_service: &SceneService, entity_name: &str) -> bool {
    scene_service
        .lifecycle_of(entity_name)
        .map(|lifecycle| lifecycle.simulation_enabled && lifecycle.collision_enabled)
        .unwrap_or(false)
}
