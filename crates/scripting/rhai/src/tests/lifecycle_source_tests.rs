use super::*;

#[test]
fn executes_scene_spawn_script() {
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
            "test-script",
            r#"
                    world.entities.create("camera-2d");
                    world.entities.create("player");
                "#,
        )
        .expect("script execution should succeed");

    assert_eq!(scene.entity_count(), 2);
    assert_eq!(
        scene.entity_names(),
        vec!["camera-2d".to_owned(), "player".to_owned()]
    );
}

#[test]
fn exposes_launch_selection_to_scripts() {
    let launch_selection = Arc::new(LaunchSelection::new(
        Some("core-game".to_owned()),
        Some("dev-shell".to_owned()),
        vec!["core".to_owned(), "core-game".to_owned()],
        true,
    ));
    let catalog = Arc::new(ModCatalog::from_discovered_mods(vec![discovered_mod(
        "core-game",
        &["dev_interface", "console_shell"],
        &["dev-shell", "console"],
    )]));
    let runtime = RhaiScriptRuntime::new(
        None,
        None,
        None,
        None,
        Some(launch_selection),
        Some(catalog),
        None,
        None,
        None,
        None,
    );

    runtime
        .execute(
            "launch-selection-script",
            r#"
                    if world.mod.current_id() != "core-game" { throw("wrong mod"); }
                    if world.mod.scenes().len != 2 { throw("wrong scene count"); }
                    if !world.mod.has_scene("dev-shell") { throw("missing scene"); }
                    if !world.runtime.dev_mode() { throw("dev mode disabled"); }
                "#,
        )
        .expect("script should be able to inspect launch selection");
}

#[test]
fn exposes_world_domains_and_entity_refs_to_scripts() {
    let scene = Arc::new(SceneService::default());
    scene.select_scene(SceneKey::new("hello-world-square"));
    scene.spawn("playground-2d-square");
    scene.configure_entity_metadata(
        "playground-2d-square",
        SceneEntityLifecycle::default(),
        vec!["debug".to_owned(), "actor".to_owned()],
        vec!["preview".to_owned()],
        BTreeMap::from([
            ("score_value".to_owned(), ScenePropertyValue::Int(100)),
            (
                "label".to_owned(),
                ScenePropertyValue::String("square".to_owned()),
            ),
        ]),
    );

    let input = Arc::new(InputState::default());
    input.set_key(KeyCode::Left, true);
    input.set_key(KeyCode::Up, true);

    let command_queue = Arc::new(ScriptCommandQueue::default());
    let event_queue = Arc::new(ScriptEventQueue::default());
    let console_queue = Arc::new(DevConsoleQueue::default());
    let launch_selection = Arc::new(LaunchSelection::new(
        Some("playground-2d".to_owned()),
        Some("hello-world-square".to_owned()),
        vec!["core".to_owned(), "playground-2d".to_owned()],
        true,
    ));
    let catalog = Arc::new(ModCatalog::from_discovered_mods(vec![discovered_mod(
        "playground-2d",
        &["rendering_2d", "text_2d"],
        &["hello-world-square", "hello-world-spritesheet"],
    )]));
    let runtime = RhaiScriptRuntime::new(
        Some(scene.clone()),
        None,
        None,
        Some(input),
        Some(launch_selection),
        Some(catalog),
        None,
        Some(command_queue.clone()),
        Some(event_queue.clone()),
        Some(console_queue.clone()),
    );

    runtime
            .execute(
                "world-api-script",
                r#"
                    let square = world.entities.named("playground-2d-square");

                    if !square.exists() { throw("missing entity"); }
                    if square.name() != "playground-2d-square" { throw("wrong entity name"); }
                    if world.entities.count() != 1 { throw("wrong entity count"); }
                    if world.entities.names().len != 1 { throw("wrong entity names"); }
                    if world.scene.current_id() != "hello-world-square" { throw("wrong current scene"); }
                    if !world.scene.has("hello-world-spritesheet") { throw("missing scene"); }
                    if world.scene.available().len != 2 { throw("wrong available scene count"); }
                    if !world.input.down("ArrowLeft") { throw("missing key down"); }
                    if !world.input.pressed("ArrowUp") { throw("missing key press"); }
                    if !world.input.any_down("A, ArrowLeft") { throw("missing any_down csv"); }
                    if world.input.any_down("A,D") { throw("unexpected any_down csv"); }
                    if !world.input.any_down(["A", "ArrowUp"]) { throw("missing any_down array"); }
                    if !world.input.any_pressed("Space, ArrowUp") { throw("missing any_pressed csv"); }
                    if world.input.any_pressed(["A", "Space"]) { throw("unexpected any_pressed array"); }
                    if world.input.axis("ArrowUp", "ArrowDown") != 1 { throw("wrong positive axis"); }
                    if world.input.axis("ArrowRight", "ArrowLeft") != -1 { throw("wrong negative axis"); }
                    if world.input.axis(["ArrowUp"], ["ArrowLeft"]) != 0 { throw("opposed axis should cancel"); }
                    if world.input.keys().len != 2 { throw("wrong pressed key count"); }

                    square.rotate_2d(1.0);
                    if !world.entities.set_position_2d("playground-2d-square", 12.0, 34.0) {
                        throw("failed to set position through world.entities");
                    }
                    if world.entities.hide_many([square.name()]) != 1 {
                        throw("failed to hide_many through world.entities");
                    }
                    if !square.set_position_2d(56.0, 78.0) {
                        throw("failed to set position through entity ref");
                    }
                    if !square.hide() {
                        throw("failed to hide through entity ref");
                    }
                    if square.is_visible() {
                        throw("hide should clear visible flag");
                    }
                    if !square.show() {
                        throw("failed to show through entity ref");
                    }
                    if !world.entities.is_visible(square.name()) {
                        throw("show should set visible flag");
                    }
                    if !square.disable() || square.is_enabled() {
                        throw("disable should clear simulation flag");
                    }
                    if !world.entities.enable(square.name()) || !square.is_enabled() {
                        throw("enable should set simulation flag");
                    }
                    if !square.set_collision_enabled(false) || square.collision_enabled() {
                        throw("collision flag should be mutable");
                    }
                    if !world.entities.set_collision_enabled(square.name(), true) || !world.entities.collision_enabled(square.name()) {
                        throw("world collision flag helper failed");
                    }
                    if !square.has_tag("debug") || !world.entities.has_tag(square.name(), "actor") {
                        throw("missing tag");
                    }
                    if !square.has_group("preview") || !world.entities.has_group(square.name(), "preview") {
                        throw("missing group");
                    }
                    if world.entities.by_tag("debug").len != 1 {
                        throw("wrong by_tag count");
                    }
                    if world.entities.by_group("preview").len != 1 {
                        throw("wrong by_group count");
                    }
                    if world.entities.active_by_tag("debug").len != 1 {
                        throw("wrong active_by_tag count");
                    }
                    if square.property_int("score_value") != 100 {
                        throw("wrong entity ref property int");
                    }
                    if world.entities.property_string(square.name(), "label") != "square" {
                        throw("wrong world property string");
                    }
                    if !world.entities.set_property_int(square.name(), "score_value", 250) {
                        throw("failed to set int property");
                    }
                    if !world.entities.set_property_string(square.name(), "label", "renamed") {
                        throw("failed to set string property");
                    }
                    if square.property_int("score_value") != 250 {
                        throw("updated property int missing");
                    }
                    if world.entities.property_string(square.name(), "label") != "renamed" {
                        throw("updated world property string missing");
                    }
                    world.scene.select("hello-world-spritesheet");
                    world.dev.event("scene.intent", "hello-world-spritesheet");
                    world.dev.command("help");
                    world.dev.log("hello from world");
                    world.dev.warn("careful");
                    world.dev.refresh_diagnostics("playground-2d");
                "#,
            )
            .expect("script should be able to use the world API");

    assert_eq!(
        scene
            .transform_of("playground-2d-square")
            .expect("square should exist")
            .translation
            .x,
        56.0
    );
    assert_eq!(
        scene
            .transform_of("playground-2d-square")
            .expect("square should exist")
            .translation
            .y,
        78.0
    );
    assert!(scene.is_visible("playground-2d-square"));
    assert_eq!(
        scene
            .transform_of("playground-2d-square")
            .expect("square should exist")
            .rotation_euler
            .z,
        1.0
    );
    assert_eq!(command_queue.pending().len(), 4);
    assert_eq!(command_queue.pending()[0].namespace, "scene");
    assert_eq!(command_queue.pending()[1].namespace, "debug");
    assert_eq!(command_queue.pending()[2].namespace, "debug");
    assert_eq!(command_queue.pending()[3].namespace, "dev-shell");
    assert_eq!(event_queue.pending().len(), 1);
    assert_eq!(console_queue.pending().len(), 1);
}
