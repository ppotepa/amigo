use std::collections::BTreeSet;

use amigo_scene::{EntityPoolSceneService, SceneService};

use crate::{CollisionEvent2d, Physics2dSceneService};

pub fn evaluate_collision_event_rules(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
) -> Vec<CollisionEvent2d> {
    evaluate_collision_event_rules_with_pools(scene_service, physics_scene_service, None)
}

pub fn evaluate_collision_event_rules_with_pools(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
    pool_scene_service: Option<&EntityPoolSceneService>,
) -> Vec<CollisionEvent2d> {
    let mut events = Vec::new();

    for command in physics_scene_service.collision_event_rules() {
        let rule = command.rule;
        let sources = crate::selectors::resolve_collision_candidates_with_pools(
            scene_service,
            physics_scene_service,
            pool_scene_service,
            &rule.source,
        );
        let targets = crate::selectors::resolve_collision_candidates_with_pools(
            scene_service,
            physics_scene_service,
            pool_scene_service,
            &rule.target,
        );
        let mut current_overlaps = BTreeSet::new();

        for source_entity in &sources {
            for target_entity in &targets {
                if source_entity == target_entity {
                    continue;
                }
                if !crate::simulation::circle_colliders_overlap(
                    scene_service,
                    physics_scene_service,
                    source_entity,
                    target_entity,
                ) {
                    continue;
                }

                current_overlaps.insert((source_entity.clone(), target_entity.clone()));
                let was_active = physics_scene_service.is_collision_rule_overlap_active(
                    &rule.id,
                    source_entity,
                    target_entity,
                );
                if !rule.once_per_overlap || !was_active {
                    events.push(CollisionEvent2d {
                        rule_id: rule.id.clone(),
                        topic: rule.event.clone(),
                        source_entity: source_entity.clone(),
                        target_entity: target_entity.clone(),
                    });
                }
                physics_scene_service.set_collision_rule_overlap_active(
                    &rule.id,
                    source_entity,
                    target_entity,
                    true,
                );
            }
        }

        let active_overlaps = physics_scene_service.collision_rule_active_overlaps_for(&rule.id);

        for active in active_overlaps {
            let (_rule_id, source_entity, target_entity): (String, String, String) = active;
            if !current_overlaps.contains(&(source_entity.clone(), target_entity.clone())) {
                physics_scene_service.set_collision_rule_overlap_active(
                    &rule.id,
                    &source_entity,
                    &target_entity,
                    false,
                );
            }
        }
    }

    events
}
