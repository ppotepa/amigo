use amigo_scene::{CollisionEventRule2dSceneCommand, EntitySelector, SceneService};

use super::common::queue_circle;
use crate::{
    Physics2dSceneService, evaluate_collision_event_rules, queue_collision_event_rule_scene_command,
};

#[test]
fn circle_collider_overlap_uses_scene_transforms() {
    let scene = SceneService::default();
    scene.spawn("bullet");
    scene.spawn("asteroid");
    assert!(scene.set_transform(
        "bullet",
        amigo_math::Transform3 {
            translation: amigo_math::Vec3::new(10.0, 12.0, 0.0),
            ..Default::default()
        },
    ));
    assert!(scene.set_transform(
        "asteroid",
        amigo_math::Transform3 {
            translation: amigo_math::Vec3::new(20.0, 12.0, 0.0),
            ..Default::default()
        },
    ));

    let service = Physics2dSceneService::default();
    queue_circle(&service, "bullet", 0, 4.0);
    queue_circle(&service, "asteroid", 1, 8.0);

    assert!(crate::circle_colliders_overlap(
        &scene, &service, "bullet", "asteroid"
    ));
    assert!(!crate::circle_colliders_overlap(
        &scene, &service, "bullet", "missing"
    ));
}

#[test]
fn collision_event_rule_publishes_once_and_reenters_after_separation() {
    let scene = SceneService::default();
    scene.spawn("source");
    scene.spawn("target");
    assert!(scene.set_transform(
        "target",
        amigo_math::Transform3 {
            translation: amigo_math::Vec3::new(8.0, 0.0, 0.0),
            ..Default::default()
        },
    ));

    let service = Physics2dSceneService::default();
    queue_circle(&service, "source", 0, 5.0);
    queue_circle(&service, "target", 1, 5.0);
    queue_collision_event_rule_scene_command(
        &service,
        &CollisionEventRule2dSceneCommand::new(
            "test-mod",
            "source-hits-target",
            EntitySelector::Entity("source".to_owned()),
            EntitySelector::Entity("target".to_owned()),
            "collision.hit",
            true,
        ),
    );

    let first = evaluate_collision_event_rules(&scene, &service);
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].source_entity, "source");
    assert_eq!(first[0].target_entity, "target");

    assert!(evaluate_collision_event_rules(&scene, &service).is_empty());

    assert!(scene.set_transform(
        "target",
        amigo_math::Transform3 {
            translation: amigo_math::Vec3::new(100.0, 0.0, 0.0),
            ..Default::default()
        },
    ));
    assert!(evaluate_collision_event_rules(&scene, &service).is_empty());

    assert!(scene.set_transform(
        "target",
        amigo_math::Transform3 {
            translation: amigo_math::Vec3::new(8.0, 0.0, 0.0),
            ..Default::default()
        },
    ));
    assert_eq!(evaluate_collision_event_rules(&scene, &service).len(), 1);
}
