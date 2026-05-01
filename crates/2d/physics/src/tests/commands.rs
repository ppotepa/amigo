use amigo_math::Vec2;
use amigo_scene::{
    AabbCollider2dSceneCommand, CircleCollider2dSceneCommand, KinematicBody2dSceneCommand,
    SceneEntityId, SceneService, Trigger2dSceneCommand,
};

use crate::{
    AabbCollider2d, AabbCollider2dCommand, CircleCollider2d, CircleCollider2dCommand,
    CollisionLayer, CollisionMask, KinematicBody2d, KinematicBody2dCommand, Physics2dSceneService,
    StaticCollider2d, StaticCollider2dCommand, Trigger2d, Trigger2dCommand,
    queue_aabb_collider_scene_command, queue_circle_collider_scene_command,
    queue_kinematic_body_scene_command, queue_trigger_scene_command,
};

#[test]
fn stores_physics2d_commands() {
    let service = Physics2dSceneService::default();

    service.queue_body(KinematicBody2dCommand {
        entity_id: SceneEntityId::new(1),
        entity_name: "player".to_owned(),
        body: KinematicBody2d {
            velocity: Vec2::new(10.0, -30.0),
            gravity_scale: 1.0,
            terminal_velocity: 720.0,
        },
    });
    service.queue_aabb_collider(AabbCollider2dCommand {
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
    service.queue_trigger(Trigger2dCommand {
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

    assert_eq!(service.kinematic_bodies().len(), 1);
    assert_eq!(service.aabb_colliders().len(), 1);
    assert_eq!(service.static_colliders().len(), 1);
    assert_eq!(service.triggers().len(), 1);
    assert_eq!(
        service.entity_names(),
        vec!["coin".to_owned(), "ground".to_owned(), "player".to_owned()]
    );

    service.clear();
    assert!(service.kinematic_bodies().is_empty());
    assert!(service.aabb_colliders().is_empty());
    assert!(service.static_colliders().is_empty());
    assert!(service.triggers().is_empty());
}

#[test]
fn queues_scene_commands_through_physics_helpers() {
    let scene = SceneService::default();
    let service = Physics2dSceneService::default();

    let body_entity = queue_kinematic_body_scene_command(
        &scene,
        &service,
        &KinematicBody2dSceneCommand::new(
            "playground-sidescroller",
            "player",
            Vec2::new(4.0, -8.0),
            1.0,
            720.0,
        ),
    );
    let collider_entity = queue_aabb_collider_scene_command(
        &scene,
        &service,
        &AabbCollider2dSceneCommand::new(
            "playground-sidescroller",
            "player",
            Vec2::new(20.0, 30.0),
            Vec2::new(0.0, 1.0),
            "player",
            vec!["world".to_owned(), "trigger".to_owned()],
        ),
    );
    let trigger_entity = queue_trigger_scene_command(
        &scene,
        &service,
        &Trigger2dSceneCommand::new(
            "playground-sidescroller",
            "coin",
            Vec2::new(16.0, 16.0),
            Vec2::ZERO,
            "trigger",
            vec!["player".to_owned()],
            Some("coin.collected".to_owned()),
        ),
    );

    assert_eq!(body_entity, collider_entity);
    assert_ne!(body_entity, trigger_entity);
    assert_eq!(service.kinematic_bodies().len(), 1);
    assert_eq!(service.aabb_collider("player").is_some(), true);
    assert_eq!(service.triggers().len(), 1);
    assert_eq!(
        scene.entity_names(),
        vec!["player".to_owned(), "coin".to_owned()]
    );
}

#[test]
fn queues_circle_collider_scene_commands() {
    let scene = SceneService::default();
    let service = Physics2dSceneService::default();

    let entity = queue_circle_collider_scene_command(
        &scene,
        &service,
        &CircleCollider2dSceneCommand::new("test-mod", "test-actor", 10.0, Vec2::new(0.0, 2.0)),
    );

    assert_eq!(
        service.circle_collider("test-actor"),
        Some(CircleCollider2dCommand {
            entity_id: entity,
            entity_name: "test-actor".to_owned(),
            collider: CircleCollider2d {
                radius: 10.0,
                offset: Vec2::new(0.0, 2.0),
            },
        })
    );
    assert_eq!(scene.entity_names(), vec!["test-actor".to_owned()]);
}
