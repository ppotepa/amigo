use amigo_scene::{
    EntityPoolSceneCommand, EntityPoolSceneService, EntitySelector, SceneEntityLifecycle,
    SceneService,
};

use crate::{
    Physics2dSceneService, circle_colliders_overlap, first_overlap_by_selector,
    resolve_collision_candidates, resolve_collision_candidates_with_pools,
};

use super::common::queue_circle;

#[test]
fn set_circle_radius_updates_existing_circle_collider() {
    let service = Physics2dSceneService::default();
    queue_circle(&service, "asteroid", 1, 8.0);

    assert!(service.set_circle_radius("asteroid", 32.0));
    assert_eq!(
        service
            .circle_collider("asteroid")
            .expect("circle collider should exist")
            .collider
            .radius,
        32.0
    );
    assert!(service.set_circle_radius("asteroid", -4.0));
    assert_eq!(
        service
            .circle_collider("asteroid")
            .expect("circle collider should exist")
            .collider
            .radius,
        0.0
    );
    assert!(!service.set_circle_radius("missing", 12.0));
}

#[test]
fn selector_queries_resolve_tag_and_group_candidates() {
    let scene = SceneService::default();
    scene.spawn("source");
    scene.spawn("tagged");
    scene.spawn("grouped");
    assert!(scene.configure_entity_metadata(
        "tagged",
        SceneEntityLifecycle::default(),
        vec!["target".to_owned()],
        Vec::new(),
        Default::default(),
    ));
    assert!(scene.configure_entity_metadata(
        "grouped",
        SceneEntityLifecycle::default(),
        Vec::new(),
        vec!["targets".to_owned()],
        Default::default(),
    ));
    assert!(scene.set_transform(
        "source",
        amigo_math::Transform3 {
            translation: amigo_math::Vec3::new(0.0, 0.0, 0.0),
            ..Default::default()
        },
    ));
    assert!(scene.set_transform(
        "tagged",
        amigo_math::Transform3 {
            translation: amigo_math::Vec3::new(8.0, 0.0, 0.0),
            ..Default::default()
        },
    ));
    assert!(scene.set_transform(
        "grouped",
        amigo_math::Transform3 {
            translation: amigo_math::Vec3::new(24.0, 0.0, 0.0),
            ..Default::default()
        },
    ));

    let service = Physics2dSceneService::default();
    queue_circle(&service, "source", 0, 5.0);
    queue_circle(&service, "tagged", 1, 5.0);
    queue_circle(&service, "grouped", 2, 5.0);

    assert_eq!(
        resolve_collision_candidates(&scene, &service, &EntitySelector::Tag("target".into())),
        vec!["tagged".to_owned()]
    );
    assert_eq!(
        resolve_collision_candidates(&scene, &service, &EntitySelector::Group("targets".into())),
        vec!["grouped".to_owned()]
    );
    assert_eq!(
        first_overlap_by_selector(
            &scene,
            &service,
            "source",
            &EntitySelector::Tag("target".into())
        ),
        Some("tagged".to_owned())
    );
    assert_eq!(
        first_overlap_by_selector(
            &scene,
            &service,
            "source",
            &EntitySelector::Group("targets".into())
        ),
        None
    );
}

#[test]
fn overlap_queries_ignore_simulation_or_collision_disabled_entities() {
    let scene = SceneService::default();
    scene.spawn("source");
    scene.spawn("collision-disabled");
    scene.spawn("simulation-disabled");
    assert!(scene.set_collision_enabled("collision-disabled", false));
    assert!(scene.set_simulation_enabled("simulation-disabled", false));

    let service = Physics2dSceneService::default();
    queue_circle(&service, "source", 0, 10.0);
    queue_circle(&service, "collision-disabled", 1, 10.0);
    queue_circle(&service, "simulation-disabled", 2, 10.0);

    assert!(!circle_colliders_overlap(
        &scene,
        &service,
        "source",
        "collision-disabled"
    ));
    assert!(!circle_colliders_overlap(
        &scene,
        &service,
        "source",
        "simulation-disabled"
    ));
}

#[test]
fn pool_selector_is_deferred_until_pool_service_exists() {
    let scene = SceneService::default();
    let service = Physics2dSceneService::default();

    assert!(
        resolve_collision_candidates(
            &scene,
            &service,
            &EntitySelector::Pool("projectiles".to_owned())
        )
        .is_empty()
    );
}

#[test]
fn pool_selector_resolves_members_when_pool_service_is_provided() {
    let scene = SceneService::default();
    scene.spawn("projectile-a");
    scene.spawn("projectile-b");
    let service = Physics2dSceneService::default();
    queue_circle(&service, "projectile-a", 0, 5.0);
    queue_circle(&service, "projectile-b", 1, 5.0);
    let pools = EntityPoolSceneService::default();
    pools.queue(EntityPoolSceneCommand::new(
        "test",
        "projectiles",
        vec!["projectile-a".to_owned(), "projectile-b".to_owned()],
    ));

    assert_eq!(
        resolve_collision_candidates_with_pools(
            &scene,
            &service,
            Some(&pools),
            &EntitySelector::Pool("projectiles".to_owned())
        ),
        vec!["projectile-a".to_owned(), "projectile-b".to_owned()]
    );
}
