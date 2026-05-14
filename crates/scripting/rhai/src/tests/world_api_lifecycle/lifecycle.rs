use super::*;

#[test]
fn call_update_does_not_rerun_top_level_script_body() {
    let scene = Arc::new(SceneService::default());
    let runtime = RhaiScriptRuntime::new(
        Some(scene.clone()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    runtime
        .execute(
            "bootstrap-once",
            r#"
                world.entities.create("boot-only");

                fn update(dt) {
                }
            "#,
        )
        .expect("script execution should succeed");

    assert_eq!(scene.entity_count(), 1);

    runtime
        .call_update("bootstrap-once", 1.0 / 60.0)
        .expect("update function should succeed");

    assert_eq!(
        scene.entity_count(),
        1,
        "top-level script body should not be re-evaluated during update ticks"
    );
}

#[test]
fn unload_removes_script_from_registry() {
    let scene = Arc::new(SceneService::default());
    scene.spawn("playground-2d-square");
    let runtime = RhaiScriptRuntime::new(
        Some(scene.clone()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    runtime
        .execute(
            "unload-test",
            r#"
                fn update(dt) {
                    let square = world.entities.named("playground-2d-square");
                    let applied = square.rotate_2d(dt);
                }
            "#,
        )
        .expect("script execution should succeed");
    runtime
        .call_update("unload-test", 1.0)
        .expect("update should run before unload");

    assert_eq!(
        scene
            .transform_of("playground-2d-square")
            .expect("entity should exist")
            .rotation_euler
            .z,
        1.0
    );

    runtime
        .unload("unload-test")
        .expect("unload should succeed");
    runtime
        .call_update("unload-test", 1.0)
        .expect("update on unloaded source should be a no-op");

    assert_eq!(
        scene
            .transform_of("playground-2d-square")
            .expect("entity should still exist")
            .rotation_euler
            .z,
        1.0,
        "unloaded script should no longer receive updates"
    );
}

#[test]
fn can_reexecute_source_after_unload() {
    let scene = Arc::new(SceneService::default());
    scene.spawn("playground-2d-square");
    let runtime = RhaiScriptRuntime::new(
        Some(scene.clone()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    runtime
        .execute(
            "reloadable-script",
            r#"
                fn update(dt) {
                    let square = world.entities.named("playground-2d-square");
                    let applied = square.rotate_2d(dt);
                }
            "#,
        )
        .expect("first script execution should succeed");
    runtime
        .call_update("reloadable-script", 1.0)
        .expect("first update should succeed");
    runtime
        .unload("reloadable-script")
        .expect("unload should succeed");
    runtime
        .execute(
            "reloadable-script",
            r#"
                fn update(dt) {
                    let square = world.entities.named("playground-2d-square");
                    let applied = square.rotate_2d(dt * 2.0);
                }
            "#,
        )
        .expect("second script execution should succeed");
    runtime
        .call_update("reloadable-script", 1.0)
        .expect("second update should succeed");

    assert_eq!(
        scene
            .transform_of("playground-2d-square")
            .expect("entity should exist")
            .rotation_euler
            .z,
        3.0,
        "re-executed script should be registered again under the same source name"
    );
}

#[test]
fn lifecycle_hooks_can_use_world_time_and_dev_domains() {
    let console_queue = Arc::new(DevConsoleQueue::default());
    let runtime = RhaiScriptRuntime::new(
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(console_queue.clone()),
    );

    runtime
        .execute(
            "lifecycle-test",
            r#"
                fn on_enter() {
                    if world.time.frame() != 0 { throw("on_enter should not advance frames"); }
                    if world.time.delta() != 0.0 { throw("on_enter should have zero delta"); }
                    world.dev.command("enter");
                }

                fn update(dt) {
                    if world.time.frame() < 1 { throw("update should advance frames"); }
                    if world.time.delta() <= 0.0 { throw("update should expose delta"); }
                    if world.time.frame() == 2 && world.time.elapsed() < 0.75 { throw("elapsed time should accumulate"); }
                    world.dev.command("tick");
                }

                fn on_event(topic, payload) {
                    if topic != "demo.event" { throw("unexpected event topic"); }
                    if payload.len != 2 { throw("unexpected payload length"); }
                    if world.time.delta() != 0.0 { throw("on_event should be passive"); }
                    world.dev.command("event");
                }

                fn on_exit() {
                    if world.time.delta() != 0.0 { throw("on_exit should be passive"); }
                    world.dev.command("exit");
                }
            "#,
        )
        .expect("script execution should succeed");

    runtime
        .call_on_enter("lifecycle-test")
        .expect("on_enter should succeed");
    runtime
        .call_update("lifecycle-test", 0.25)
        .expect("first update should succeed");
    runtime
        .call_update("lifecycle-test", 0.50)
        .expect("second update should succeed");
    runtime
        .call_on_event(
            "lifecycle-test",
            "demo.event",
            &["one".to_owned(), "two".to_owned()],
        )
        .expect("on_event should succeed");
    runtime
        .call_on_exit("lifecycle-test")
        .expect("on_exit should succeed");

    assert_eq!(console_queue.pending().len(), 5);
    assert_eq!(console_queue.pending()[0].line, "enter".to_owned());
    assert_eq!(console_queue.pending()[1].line, "tick".to_owned());
    assert_eq!(console_queue.pending()[2].line, "tick".to_owned());
    assert_eq!(console_queue.pending()[3].line, "event".to_owned());
    assert_eq!(console_queue.pending()[4].line, "exit".to_owned());
}

#[test]
fn can_call_named_event_pipeline_fallback_function() {
    let console_queue = Arc::new(DevConsoleQueue::default());
    let runtime = RhaiScriptRuntime::new(
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(console_queue.clone()),
    );

    runtime
        .execute(
            "pipeline-fallback-test",
            r#"
                fn on_custom_pipeline_step(topic, payload) {
                    if topic != "demo.pipeline" { throw("unexpected pipeline topic"); }
                    if payload.len != 1 { throw("unexpected payload length"); }
                    world.dev.command("pipeline:" + payload[0]);
                }
            "#,
        )
        .expect("script execution should succeed");

    runtime
        .call_event_function(
            "pipeline-fallback-test",
            "on_custom_pipeline_step",
            "demo.pipeline",
            &["ok".to_owned()],
        )
        .expect("custom pipeline function should succeed");

    assert_eq!(console_queue.pending()[0].line, "pipeline:ok".to_owned());
}
