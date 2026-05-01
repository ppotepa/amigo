use amigo_math::Vec2;

use crate::model::{AabbCollider2d, StaticCollider2dCommand, Trigger2dCommand};

#[derive(Clone, Copy)]
struct Rect2d {
    min: Vec2,
    max: Vec2,
}

pub fn move_and_collide(
    translation: Vec2,
    collider: &AabbCollider2d,
    velocity: Vec2,
    delta_seconds: f32,
    static_colliders: &[StaticCollider2dCommand],
) -> crate::model::PhysicsStepResult2d {
    let mut translation = translation;
    let mut velocity = velocity;
    let mut grounded = crate::model::GroundedState::default();
    let delta = Vec2::new(velocity.x * delta_seconds, velocity.y * delta_seconds);
    let half_size = Vec2::new(collider.size.x * 0.5, collider.size.y * 0.5);

    translation.x += delta.x;
    for static_collider in static_colliders {
        if !collider.mask.allows(&static_collider.collider.layer) {
            continue;
        }
        if !intersects_rect(
            dynamic_rect(translation, collider.offset, half_size),
            static_rect(static_collider),
        ) {
            continue;
        }

        if delta.x > 0.0 {
            translation.x = static_rect(static_collider).min.x - half_size.x - collider.offset.x;
            grounded.hit_wall = true;
        } else if delta.x < 0.0 {
            translation.x = static_rect(static_collider).max.x + half_size.x - collider.offset.x;
            grounded.hit_wall = true;
        }
        velocity.x = 0.0;
    }

    translation.y += delta.y;
    for static_collider in static_colliders {
        if !collider.mask.allows(&static_collider.collider.layer) {
            continue;
        }
        if !intersects_rect(
            dynamic_rect(translation, collider.offset, half_size),
            static_rect(static_collider),
        ) {
            continue;
        }

        if delta.y < 0.0 {
            translation.y = static_rect(static_collider).max.y + half_size.y - collider.offset.y;
            grounded.grounded = true;
        } else if delta.y > 0.0 {
            translation.y = static_rect(static_collider).min.y - half_size.y - collider.offset.y;
            grounded.hit_ceiling = true;
        }
        velocity.y = 0.0;
    }

    crate::model::PhysicsStepResult2d {
        translation,
        velocity,
        grounded,
    }
}

pub fn overlaps_trigger(
    translation: Vec2,
    collider: &AabbCollider2d,
    trigger: &Trigger2dCommand,
) -> bool {
    overlaps_trigger_with_translation(translation, collider, trigger, None)
}

pub fn overlaps_trigger_with_translation(
    translation: Vec2,
    collider: &AabbCollider2d,
    trigger: &Trigger2dCommand,
    trigger_translation: Option<Vec2>,
) -> bool {
    if !collider.mask.allows(&trigger.trigger.layer)
        || !trigger.trigger.mask.allows(&collider.layer)
    {
        return false;
    }

    intersects_rect(
        dynamic_rect(
            translation,
            collider.offset,
            Vec2::new(collider.size.x * 0.5, collider.size.y * 0.5),
        ),
        trigger_rect(trigger, trigger_translation),
    )
}

fn dynamic_rect(translation: Vec2, offset: Vec2, half_size: Vec2) -> Rect2d {
    let center = Vec2::new(translation.x + offset.x, translation.y + offset.y);
    Rect2d {
        min: Vec2::new(center.x - half_size.x, center.y - half_size.y),
        max: Vec2::new(center.x + half_size.x, center.y + half_size.y),
    }
}

fn static_rect(collider: &StaticCollider2dCommand) -> Rect2d {
    let half_size = Vec2::new(
        collider.collider.size.x * 0.5,
        collider.collider.size.y * 0.5,
    );
    Rect2d {
        min: Vec2::new(
            collider.collider.offset.x - half_size.x,
            collider.collider.offset.y - half_size.y,
        ),
        max: Vec2::new(
            collider.collider.offset.x + half_size.x,
            collider.collider.offset.y + half_size.y,
        ),
    }
}

fn trigger_rect(trigger: &Trigger2dCommand, translation: Option<Vec2>) -> Rect2d {
    let half_size = Vec2::new(trigger.trigger.size.x * 0.5, trigger.trigger.size.y * 0.5);
    let center = translation.unwrap_or(trigger.trigger.offset);
    Rect2d {
        min: Vec2::new(center.x - half_size.x, center.y - half_size.y),
        max: Vec2::new(center.x + half_size.x, center.y + half_size.y),
    }
}

fn intersects_rect(left: Rect2d, right: Rect2d) -> bool {
    left.min.x < right.max.x
        && left.max.x > right.min.x
        && left.min.y < right.max.y
        && left.max.y > right.min.y
}
