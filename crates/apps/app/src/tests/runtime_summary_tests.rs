use super::*;

#[test]
fn runtime_can_process_console_commands_after_bootstrap() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "core-game".to_owned()])
            .with_startup_mod("core-game")
            .with_startup_scene("console")
            .with_dev_mode(true),
    )
    .expect("console bootstrap should succeed");

    runtime
        .resolve::<DevConsoleQueue>()
        .expect("dev console queue should exist")
        .submit(DevConsoleCommand::new("diagnostics"));

    let updated = refresh_runtime_summary(&runtime)
        .expect("runtime refresh should process queued console command");

    assert!(
        updated
            .console_commands
            .iter()
            .any(|command| command == "diagnostics")
    );
    assert!(
        updated
            .console_output
            .iter()
            .any(|line| line.contains("window=winit input=winit render=wgpu script=rhai"))
    );
}

#[test]
fn runtime_render_plan_and_graph_for_default_packet_are_world_to_present() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "core-game".to_owned()])
            .with_startup_mod("core-game")
            .with_startup_scene("console")
            .with_dev_mode(true),
    )
    .expect("console bootstrap should succeed");

    runtime
        .resolve::<DevConsoleQueue>()
        .expect("dev console queue should exist")
        .submit(DevConsoleCommand::new("render.plan"));

    let plan = refresh_runtime_summary(&runtime)
        .expect("runtime refresh should process render plan command");

    assert!(plan.console_output.iter().any(|line| {
        line.contains("render.plan: no composition captured yet")
            || line.contains("view=")
            || line.contains("world -> present")
    }));

    runtime
        .resolve::<DevConsoleQueue>()
        .expect("dev console queue should exist")
        .submit(DevConsoleCommand::new("render.graph"));

    let graph = refresh_runtime_summary(&runtime)
        .expect("runtime refresh should process render graph command");

    assert!(
        graph
            .console_output
            .iter()
            .any(|line| line.contains("warnings:"))
            || graph
                .console_output
                .iter()
                .any(|line| line.contains("render.graph: no graph captured yet"))
    );
    assert!(
        !graph
            .console_output
            .iter()
            .any(|line| line.contains("non-present node '"))
    );
    assert!(
        !graph
            .console_output
            .iter()
            .any(|line| line.contains("non-present node '"))
    );
}

#[test]
fn runtime_render_graph_with_lens_droplets_has_plan_and_no_surface_write_warnings() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "core-game".to_owned()])
            .with_startup_mod("core-game")
            .with_startup_scene("console")
            .with_dev_mode(true),
    )
    .expect("console bootstrap should succeed");

    let post_fx_service = runtime
        .resolve::<amigo_runtime_bundles::amigo_2d_post_fx::PostFx2dService>()
        .expect("post-fx service should exist");
    post_fx_service.set_scene_stack(amigo_runtime_bundles::amigo_2d_post_fx::PostFx2dStack {
        effects: vec![amigo_runtime_bundles::amigo_2d_post_fx::PostFx2d::LensDroplets(
            amigo_runtime_bundles::amigo_2d_post_fx::PostFxLensDroplets2d {
                enabled: true,
                affects_world: true,
                affects_game_ui: true,
                affects_debug_ui: true,
                ..amigo_runtime_bundles::amigo_2d_post_fx::PostFxLensDroplets2d::default()
            },
        )],
    });

    runtime
        .resolve::<DevConsoleQueue>()
        .expect("dev console queue should exist")
        .submit(DevConsoleCommand::new("render.plan"));

    let plan = refresh_runtime_summary(&runtime)
        .expect("runtime refresh should process render plan command");

    assert!(
        plan.console_output
            .iter()
            .any(|line| line.contains("lens_droplets"))
            || plan
                .console_output
                .iter()
                .any(|line| line.contains("render.plan: no composition captured yet"))
    );

    runtime
        .resolve::<DevConsoleQueue>()
        .expect("dev console queue should exist")
        .submit(DevConsoleCommand::new("postfx.stats"));

    let stats = refresh_runtime_summary(&runtime)
        .expect("runtime refresh should process postfx stats command");

    assert!(
        stats
            .console_output
            .iter()
            .any(|line| line.contains("postfx.effects=")
                && line.contains("lens_droplets_active=true"))
    );

    runtime
        .resolve::<DevConsoleQueue>()
        .expect("dev console queue should exist")
        .submit(DevConsoleCommand::new("render.graph"));

    let graph = refresh_runtime_summary(&runtime)
        .expect("runtime refresh should process render graph command");

    assert!(
        graph
            .console_output
            .iter()
            .any(|line| line.contains("lens_droplets") || line.contains("no graph captured yet"))
    );
    assert!(!graph.console_output.iter().any(|line| {
        line.contains("non-present node '") && line.contains("writes surface resource")
    }));
}

#[test]
fn runtime_can_reload_active_scene_after_bootstrap() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
            .with_startup_mod("playground-2d")
            .with_startup_scene("sprite-lab")
            .with_dev_mode(true),
    )
    .expect("sprite playground bootstrap should succeed");

    runtime
        .resolve::<DevConsoleQueue>()
        .expect("dev console queue should exist")
        .submit(DevConsoleCommand::new("scene reload"));

    let updated = refresh_runtime_summary(&runtime)
        .expect("runtime refresh should process scene reload command");

    assert_eq!(updated.active_scene.as_deref(), Some("sprite-lab"));
    assert!(
        updated
            .console_commands
            .iter()
            .any(|command| command == "scene reload")
    );
    assert!(
        updated
            .processed_scene_commands
            .iter()
            .any(|command| command == "scene.reload_active")
    );
    assert!(
        updated
            .processed_scene_commands
            .iter()
            .any(|command| command == "scene.select(sprite-lab)")
    );
    assert!(
        updated
            .console_output
            .iter()
            .any(|line| line.contains("reloading active scene `sprite-lab`"))
    );
}

#[test]
fn runtime_can_reload_asset_after_bootstrap() {
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(mods_root())
            .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
            .with_startup_mod("playground-2d")
            .with_startup_scene("sprite-lab")
            .with_dev_mode(true),
    )
    .expect("sprite playground bootstrap should succeed");

    runtime
        .resolve::<DevConsoleQueue>()
        .expect("dev console queue should exist")
        .submit(DevConsoleCommand::new(
            "asset reload playground-2d/spritesheets/sprite-lab",
        ));

    let updated = refresh_runtime_summary(&runtime)
        .expect("runtime refresh should process asset reload command");

    assert!(
        updated
            .console_commands
            .iter()
            .any(|command| command == "asset reload playground-2d/spritesheets/sprite-lab")
    );
    assert!(
        updated
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-2d/spritesheets/sprite-lab (sprite-sheet-2d)")
    );
    assert!(updated.console_output.iter().any(|line| {
        line.contains("queued asset reload for `playground-2d/spritesheets/sprite-lab`")
    }));
    assert!(updated.console_output.iter().any(|line| {
        line.contains("prepared asset `playground-2d/spritesheets/sprite-lab` as `sprite-sheet-2d`")
    }));
}

#[test]
fn script_component_on_attach_errors_include_runtime_diagnostic_context() {
    let temp_mods = copied_mods_root("script-component-attach-error", &["core", "playground-2d"]);
    write_lifecycle_probe(
        &temp_mods,
        r#"
fn on_attach(entity, params) {
throw("attach exploded");
}

fn update(entity, params, dt) {}

fn on_detach(entity, params) {}
"#,
    );

    let error = match bootstrap_with_options(
        BootstrapOptions::new(temp_mods)
            .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
            .with_startup_mod("playground-2d")
            .with_startup_scene("basic-scripting-demo")
            .with_dev_mode(true),
    ) {
        Ok(_) => panic!("on_attach failure should abort bootstrap"),
        Err(error) => error,
    };

    assert_script_component_diagnostic(&error, "on_attach", "attach exploded");
}

#[test]
fn script_component_on_detach_errors_include_runtime_diagnostic_context() {
    let temp_mods = copied_mods_root("script-component-detach-error", &["core", "playground-2d"]);
    write_lifecycle_probe(
        &temp_mods,
        r#"
fn on_attach(entity, params) {}

fn update(entity, params, dt) {}

fn on_detach(entity, params) {
throw("detach exploded");
}
"#,
    );
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(temp_mods)
            .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
            .with_startup_mod("playground-2d")
            .with_startup_scene("basic-scripting-demo")
            .with_dev_mode(true),
    )
    .expect("2d scripting demo bootstrap should succeed");

    runtime
        .resolve::<SceneCommandQueue>()
        .expect("scene command queue should exist")
        .submit(SceneCommand::SelectScene {
            scene: SceneKey::new("hello-world-square"),
        });
    let error = refresh_runtime_summary(&runtime)
        .expect_err("scene transition should return on_detach failure");

    assert_script_component_diagnostic(&error, "on_detach", "detach exploded");
}

#[test]
fn script_component_update_errors_include_runtime_diagnostic_context() {
    let temp_mods = copied_mods_root("script-component-update-error", &["core", "playground-2d"]);
    write_lifecycle_probe(
        &temp_mods,
        r#"
fn on_attach(entity, params) {}

fn update(entity, params, dt) {
throw("update exploded");
}

fn on_detach(entity, params) {}
"#,
    );
    let (runtime, _summary) = bootstrap_with_options(
        BootstrapOptions::new(temp_mods)
            .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
            .with_startup_mod("playground-2d")
            .with_startup_scene("basic-scripting-demo")
            .with_dev_mode(true),
    )
    .expect("2d scripting demo bootstrap should succeed");

    let error = amigo_runtime_bundles::amigo_scripting_rhai::tick_script_components(&runtime, 0.5)
        .expect_err("update failure should be returned");

    assert_script_component_diagnostic(&error, "update", "update exploded");
}








