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
            "asset reload playground-2d/textures/sprite-lab",
        ));

    let updated = refresh_runtime_summary(&runtime)
        .expect("runtime refresh should process asset reload command");

    assert!(
        updated
            .console_commands
            .iter()
            .any(|command| command == "asset reload playground-2d/textures/sprite-lab")
    );
    assert!(
        updated
            .prepared_assets
            .iter()
            .any(|asset| asset == "playground-2d/textures/sprite-lab (sprite-2d)")
    );
    assert!(updated.console_output.iter().any(|line| {
        line.contains("queued asset reload for `playground-2d/textures/sprite-lab`")
    }));
    assert!(updated.console_output.iter().any(|line| {
        line.contains("prepared asset `playground-2d/textures/sprite-lab` as `sprite-2d`")
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

    let error = crate::systems::script_components::tick_script_components(&runtime, 0.5)
        .expect_err("update failure should be returned");

    assert_script_component_diagnostic(&error, "update", "update exploded");
}
