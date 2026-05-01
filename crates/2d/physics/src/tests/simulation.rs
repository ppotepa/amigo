use amigo_math::Vec2;
use amigo_scene::SceneEntityId;

use crate::{
    AabbCollider2d, CollisionLayer, CollisionMask, move_and_collide, overlaps_trigger,
    overlaps_trigger_with_translation,
};
use crate::{Physics2dSceneService, Trigger2d, Trigger2dCommand};
use crate::{StaticCollider2d, StaticCollider2dCommand};

#[test]
fn captures_physics_world_snapshot() {
    let service = Physics2dSceneService::default();
    service.queue_body(crate::KinematicBody2dCommand {
        entity_id: SceneEntityId::new(1),
        entity_name: "player".to_owned(),
        body: crate::KinematicBody2d {
            velocity: Vec2::new(4.0, -8.0),
            gravity_scale: 1.0,
            terminal_velocity: 720.0,
        },
    });
    service.queue_aabb_collider(crate::AabbCollider2dCommand {
        entity_id: SceneEntityId::new(1),
        entity_name: "player".to_owned(),
        collider: AabbCollider2d {
            size: Vec2::new(20.0, 30.0),
            offset: Vec2::new(0.0, 1.0),
            layer: CollisionLayer::new("player"),
            mask: CollisionMask::new(vec![CollisionLayer::new("world")]),
        },
    });
    service.queue_static_collider(StaticCollider2dCommand {
        entity_id: SceneEntityId::new(2),
        entity_name: "ground".to_owned(),
        collider: StaticCollider2d {
            size: Vec2::new(64.0, 16.0),
            offset: Vec2::ZERO,
            layer: CollisionLayer::new("world"),
        },
    });
    service.queue_trigger(crate::Trigger2dCommand {
        entity_id: SceneEntityId::new(3),
        entity_name: "coin".to_owned(),
        trigger: Trigger2d {
            size: Vec2::new(16.0, 16.0),
            offset: Vec2::ZERO,
            layer: CollisionLayer::new("trigger"),
            mask: CollisionMask::new(vec![CollisionLayer::new("player")]),
            topic: Some("coin.collected".to_owned()),
        },
    });
    service.set_trigger_overlap_active("coin", "player", true);

    let world = service.world();
    assert_eq!(world.kinematic_bodies.len(), 1);
    assert_eq!(world.aabb_colliders.len(), 1);
    assert_eq!(world.static_colliders.len(), 1);
    assert_eq!(world.triggers.len(), 1);
    assert_eq!(
        world.body_states.get("player").map(|state| state.velocity),
        Some(Vec2::new(4.0, -8.0))
    );
    assert!(
        world
            .active_trigger_overlaps
            .contains(&("coin".to_owned(), "player".to_owned()))
    );
}

#[test]
fn body_falling_onto_static_collider_becomes_grounded() {
    let collider = AabbCollider2d {
        size: Vec2::new(20.0, 30.0),
        offset: Vec2::ZERO,
        layer: CollisionLayer::new("player"),
        mask: CollisionMask::new(vec![CollisionLayer::new("world")]),
    };
    let ground = StaticCollider2dCommand {
        entity_id: SceneEntityId::new(2),
        entity_name: "ground".to_owned(),
        collider: StaticCollider2d {
            size: Vec2::new(128.0, 16.0),
            offset: Vec2::new(64.0, 8.0),
            layer: CollisionLayer::new("world"),
        },
    };

    let result = move_and_collide(
        Vec2::new(64.0, 40.0),
        &collider,
        Vec2::new(0.0, -80.0),
        0.5,
        &[ground],
    );

    assert!(result.grounded.grounded);
    assert_eq!(result.velocity.y, 0.0);
    assert!(result.translation.y >= 31.0);
}

#[test]
fn move_and_collide_blocks_horizontal_penetration() {
    let collider = AabbCollider2d {
        size: Vec2::new(20.0, 30.0),
        offset: Vec2::ZERO,
        layer: CollisionLayer::new("player"),
        mask: CollisionMask::new(vec![CollisionLayer::new("world")]),
    };
    let wall = StaticCollider2dCommand {
        entity_id: SceneEntityId::new(2),
        entity_name: "wall".to_owned(),
        collider: StaticCollider2d {
            size: Vec2::new(16.0, 96.0),
            offset: Vec2::new(40.0, 48.0),
            layer: CollisionLayer::new("world"),
        },
    };

    let result = move_and_collide(
        Vec2::new(10.0, 48.0),
        &collider,
        Vec2::new(120.0, 0.0),
        0.25,
        &[wall],
    );

    assert!(result.grounded.hit_wall);
    assert_eq!(result.velocity.x, 0.0);
    assert!(result.translation.x <= 22.0);
}

#[test]
fn trigger_overlap_detects_matching_layers() {
    let collider = AabbCollider2d {
        size: Vec2::new(20.0, 30.0),
        offset: Vec2::ZERO,
        layer: CollisionLayer::new("player"),
        mask: CollisionMask::new(vec![CollisionLayer::new("trigger")]),
    };
    let trigger = Trigger2dCommand {
        entity_id: SceneEntityId::new(2),
        entity_name: "coin".to_owned(),
        trigger: Trigger2d {
            size: Vec2::new(16.0, 16.0),
            offset: Vec2::new(64.0, 64.0),
            layer: CollisionLayer::new("trigger"),
            mask: CollisionMask::new(vec![CollisionLayer::new("player")]),
            topic: Some("coin.collected".to_owned()),
        },
    };

    assert!(overlaps_trigger(Vec2::new(64.0, 64.0), &collider, &trigger));
    assert!(!overlaps_trigger(
        Vec2::new(120.0, 64.0),
        &collider,
        &trigger
    ));
}

#[test]
fn trigger_overlap_uses_runtime_translation_override() {
    let collider = AabbCollider2d {
        size: Vec2::new(20.0, 30.0),
        offset: Vec2::ZERO,
        layer: CollisionLayer::new("player"),
        mask: CollisionMask::new(vec![CollisionLayer::new("trigger")]),
    };
    let trigger = Trigger2dCommand {
        entity_id: SceneEntityId::new(2),
        entity_name: "coin".to_owned(),
        trigger: Trigger2d {
            size: Vec2::new(16.0, 16.0),
            offset: Vec2::ZERO,
            layer: CollisionLayer::new("trigger"),
            mask: CollisionMask::new(vec![CollisionLayer::new("player")]),
            topic: Some("coin.collected".to_owned()),
        },
    };

    assert!(overlaps_trigger_with_translation(
        Vec2::new(64.0, 64.0),
        &collider,
        &trigger,
        Some(Vec2::new(64.0, 64.0)),
    ));
    assert!(!overlaps_trigger_with_translation(
        Vec2::new(64.0, 64.0),
        &collider,
        &trigger,
        Some(Vec2::new(-10000.0, -10000.0)),
    ));
}
