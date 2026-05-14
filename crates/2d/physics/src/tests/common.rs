use crate::{CircleCollider2d, CircleCollider2dCommand, Physics2dSceneService};
use amigo_math::Vec2;
use amigo_scene::SceneEntityId;

pub fn queue_circle(
    service: &Physics2dSceneService,
    entity_name: impl Into<String>,
    entity_id: u64,
    radius: f32,
) {
    service.queue_circle_collider(CircleCollider2dCommand {
        entity_id: SceneEntityId::new(entity_id),
        entity_name: entity_name.into(),
        collider: CircleCollider2d {
            radius,
            offset: Vec2::ZERO,
        },
    });
}
