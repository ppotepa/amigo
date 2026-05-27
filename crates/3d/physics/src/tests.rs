use amigo_math::{Transform3, Vec3};
use amigo_runtime::RuntimeBuilder;
use amigo_scene::{
    BoxCollider3dSceneCommand, RigidBody3dSceneCommand, SceneCommand, SceneService,
    StaticBoxCollider3dSceneCommand,
};
use amigo_scripting_api::ScriptCommand;

use crate::{
    Physics3dSceneService, Physics3dScriptCommandContext, Physics3dScriptCommandOutcome,
    handle_physics3d_script_command, queue_box_collider_scene_command,
    queue_rigid_body_scene_command, queue_static_box_collider_scene_command,
};

fn runtime_with_scene_and_physics(
    scene: SceneService,
    physics: Physics3dSceneService,
) -> amigo_runtime::Runtime {
    RuntimeBuilder::default()
        .with_service(scene)
        .expect("scene service should register")
        .with_service(physics)
        .expect("physics service should register")
        .with_service(amigo_session::RuntimeFrameClockService::default())
        .expect("frame clock should register")
        .build()
}

fn transform_at(x: f32, y: f32, z: f32) -> Transform3 {
    Transform3 {
        translation: Vec3::new(x, y, z),
        rotation_euler: Vec3::ZERO,
        scale: Vec3::new(1.0, 1.0, 1.0),
    }
}

#[test]
fn queues_body_and_colliders_into_scene_service() {
    let scene = SceneService::default();
    let physics = Physics3dSceneService::default();

    queue_rigid_body_scene_command(
        &scene,
        &physics,
        &RigidBody3dSceneCommand::new("playground-3d", "cube", Vec3::ZERO, 1.0, 0.0),
    );
    queue_box_collider_scene_command(
        &scene,
        &physics,
        &BoxCollider3dSceneCommand::new(
            "playground-3d",
            "cube",
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::ZERO,
        ),
    );
    queue_static_box_collider_scene_command(
        &scene,
        &physics,
        &StaticBoxCollider3dSceneCommand::new(
            "playground-3d",
            "ground",
            Vec3::new(12.0, 1.0, 12.0),
            Vec3::new(0.0, -0.5, 0.0),
        ),
    );

    assert_eq!(scene.entity_names().len(), 2);
    assert_eq!(physics.rigid_bodies().len(), 1);
    assert!(physics.box_collider("cube").is_some());
    assert_eq!(physics.static_box_colliders().len(), 1);
    assert!(physics.body_state("cube").is_some());
}

#[test]
fn script_command_builds_dynamic_body_scene_command() {
    let outcome = handle_physics3d_script_command(
        Physics3dScriptCommandContext {
            selected_mod: "playground-3d",
        },
        ScriptCommand::new(
            "3d.physics",
            "dynamic_box",
            vec![
                "cube".to_owned(),
                "1.0".to_owned(),
                "1.0".to_owned(),
                "1.0".to_owned(),
                "0.0".to_owned(),
                "0.0".to_owned(),
                "0.0".to_owned(),
            ],
        ),
    );

    match outcome {
        Physics3dScriptCommandOutcome::Submit(SceneCommand::Plugin { command }) => {
            let body = command
                .payload_as::<RigidBody3dSceneCommand>()
                .expect("dynamic box should emit rigid body scene command");
            assert_eq!(body.entity_name, "cube");
            assert_eq!(body.source_mod, "playground-3d");
        }
        other => panic!("unexpected script command outcome: {other:?}"),
    }
}

#[test]
fn rapier_body_falls_and_settles_on_static_ground() {
    let scene = SceneService::default();
    let physics = Physics3dSceneService::default();
    scene.spawn_with_transform("cube", transform_at(0.0, 3.0, 0.0));

    queue_rigid_body_scene_command(
        &scene,
        &physics,
        &RigidBody3dSceneCommand::new("playground-3d", "cube", Vec3::ZERO, 1.0, 0.0),
    );
    queue_box_collider_scene_command(
        &scene,
        &physics,
        &BoxCollider3dSceneCommand::new(
            "playground-3d",
            "cube",
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::ZERO,
        ),
    );
    queue_static_box_collider_scene_command(
        &scene,
        &physics,
        &StaticBoxCollider3dSceneCommand::new(
            "playground-3d",
            "ground",
            Vec3::new(12.0, 1.0, 12.0),
            Vec3::new(0.0, -0.5, 0.0),
        ),
    );

    let runtime = runtime_with_scene_and_physics(scene, physics);
    let clock = runtime
        .resolve::<amigo_session::RuntimeFrameClockService>()
        .expect("frame clock should exist");
    for _ in 0..180 {
        clock.force_single_simulation_tick(1.0 / 60.0);
        crate::tick_physics_3d(&runtime).expect("physics tick should run");
    }

    let scene = runtime
        .resolve::<SceneService>()
        .expect("scene service should exist");
    let cube = scene
        .transform_of("cube")
        .expect("cube transform should be written back");
    assert!(cube.translation.y > 0.45 && cube.translation.y < 0.65);
}

#[test]
fn rapier_dynamic_cubes_stack_without_interpenetrating() {
    let scene = SceneService::default();
    let physics = Physics3dSceneService::default();
    scene.spawn_with_transform("bottom", transform_at(0.0, 1.5, 0.0));
    scene.spawn_with_transform("top", transform_at(0.0, 4.0, 0.0));

    for name in ["bottom", "top"] {
        queue_rigid_body_scene_command(
            &scene,
            &physics,
            &RigidBody3dSceneCommand::new("playground-3d", name, Vec3::ZERO, 1.0, 0.0),
        );
        queue_box_collider_scene_command(
            &scene,
            &physics,
            &BoxCollider3dSceneCommand::new(
                "playground-3d",
                name,
                Vec3::new(1.0, 1.0, 1.0),
                Vec3::ZERO,
            ),
        );
    }
    queue_static_box_collider_scene_command(
        &scene,
        &physics,
        &StaticBoxCollider3dSceneCommand::new(
            "playground-3d",
            "ground",
            Vec3::new(12.0, 1.0, 12.0),
            Vec3::new(0.0, -0.5, 0.0),
        ),
    );

    let runtime = runtime_with_scene_and_physics(scene, physics);
    let clock = runtime
        .resolve::<amigo_session::RuntimeFrameClockService>()
        .expect("frame clock should exist");
    for _ in 0..240 {
        clock.force_single_simulation_tick(1.0 / 60.0);
        crate::tick_physics_3d(&runtime).expect("physics tick should run");
    }

    let scene = runtime
        .resolve::<SceneService>()
        .expect("scene service should exist");
    let bottom = scene.transform_of("bottom").expect("bottom should exist");
    let top = scene.transform_of("top").expect("top should exist");
    assert!(bottom.translation.y > 0.45 && bottom.translation.y < 0.75);
    assert!(
        top.translation.y - bottom.translation.y > 0.85,
        "top cube should rest above bottom cube without deep interpenetration"
    );
}

#[test]
fn rapier_writes_back_angular_motion() {
    let scene = SceneService::default();
    let physics = Physics3dSceneService::default();
    scene.spawn_with_transform("cube", transform_at(0.0, 4.0, 0.0));

    queue_rigid_body_scene_command(
        &scene,
        &physics,
        &RigidBody3dSceneCommand::new("playground-3d", "cube", Vec3::ZERO, 0.0, 0.0)
            .with_angular(Vec3::new(0.8, 1.2, 0.4), 0.0),
    );
    queue_box_collider_scene_command(
        &scene,
        &physics,
        &BoxCollider3dSceneCommand::new(
            "playground-3d",
            "cube",
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::ZERO,
        ),
    );

    let runtime = runtime_with_scene_and_physics(scene, physics);
    let clock = runtime
        .resolve::<amigo_session::RuntimeFrameClockService>()
        .expect("frame clock should exist");
    for _ in 0..30 {
        clock.force_single_simulation_tick(1.0 / 60.0);
        crate::tick_physics_3d(&runtime).expect("physics tick should run");
    }

    let scene = runtime
        .resolve::<SceneService>()
        .expect("scene service should exist");
    let cube = scene.transform_of("cube").expect("cube should exist");
    assert!(cube.rotation_euler.x.abs() > 0.05 || cube.rotation_euler.y.abs() > 0.05);
}
