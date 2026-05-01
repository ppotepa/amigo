use amigo_math::Vec2;
use amigo_scene::SceneService;

use crate::Physics2dSceneService;

pub fn circle_colliders_overlap(
    scene_service: &SceneService,
    physics_scene_service: &Physics2dSceneService,
    left_entity_name: &str,
    right_entity_name: &str,
) -> bool {
    if !crate::selectors::entity_eligible_for_collision(scene_service, left_entity_name)
        || !crate::selectors::entity_eligible_for_collision(scene_service, right_entity_name)
    {
        return false;
    }
    let Some(left_collider) = physics_scene_service.circle_collider(left_entity_name) else {
        return false;
    };
    let Some(right_collider) = physics_scene_service.circle_collider(right_entity_name) else {
        return false;
    };
    let Some(left_transform) = scene_service.transform_of(left_entity_name) else {
        return false;
    };
    let Some(right_transform) = scene_service.transform_of(right_entity_name) else {
        return false;
    };

    let left_center = Vec2::new(
        left_transform.translation.x + left_collider.collider.offset.x,
        left_transform.translation.y + left_collider.collider.offset.y,
    );
    let right_center = Vec2::new(
        right_transform.translation.x + right_collider.collider.offset.x,
        right_transform.translation.y + right_collider.collider.offset.y,
    );
    let dx = right_center.x - left_center.x;
    let dy = right_center.y - left_center.y;
    let radius = left_collider.collider.radius + right_collider.collider.radius;

    dx * dx + dy * dy <= radius * radius
}
