use super::*;

#[test]
fn exposes_runtime_catalog_and_diagnostics_to_scripts() {
    let catalog = Arc::new(ModCatalog::from_discovered_mods(vec![
        discovered_mod(
            "core-game",
            &["dev_interface", "console_shell"],
            &["dev-shell", "console"],
        ),
        discovered_mod("playground-2d", &["2d"], &["sprite-lab", "text-lab"]),
    ]));
    let diagnostics = Arc::new(RuntimeDiagnostics::new(
        "winit",
        "winit",
        "wgpu",
        "rhai",
        vec!["core-game".to_owned(), "playground-2d".to_owned()],
        vec!["2d.sprite".to_owned(), "3d.mesh".to_owned()],
        vec![
            "amigo-modding".to_owned(),
            "amigo-scripting-rhai".to_owned(),
        ],
        vec!["amigo_core::RuntimeDiagnostics".to_owned()],
    ));
    let scene = Arc::new(SceneService::default());
    scene.spawn("core-game-shell");
    let assets = Arc::new(AssetCatalog::default());
    let command_queue = Arc::new(ScriptCommandQueue::default());
    let launch_selection = Arc::new(LaunchSelection::new(
        Some("playground-2d".to_owned()),
        Some("sprite-lab".to_owned()),
        vec!["core".to_owned(), "playground-2d".to_owned()],
        true,
    ));
    assets.register_manifest(AssetManifest {
        key: AssetKey::new("playground-2d/images/sprite-lab"),
        source: AssetSourceKind::Mod("playground-2d".to_owned()),
        tags: vec!["phase3".to_owned(), "2d".to_owned(), "sprite".to_owned()],
    });
    assets.request_load(AssetLoadRequest::new(
        AssetKey::new("playground-2d/images/sprite-lab"),
        AssetLoadPriority::Immediate,
    ));
    assets.mark_loaded(amigo_assets::LoadedAsset {
        key: AssetKey::new("playground-2d/images/sprite-lab"),
        source: AssetSourceKind::Mod("playground-2d".to_owned()),
        resolved_path: PathBuf::from("mods/playground-2d/assets/images/sprite-lab.image.yml"),
        byte_len: 84,
    });
    let prepared = prepare_debug_placeholder_asset(
        &assets
            .loaded_asset(&AssetKey::new("playground-2d/images/sprite-lab"))
            .expect("loaded asset should exist"),
        r#"
            kind = "sprite-2d"
            label = "Sprite Lab Placeholder"
            format = "debug-placeholder"
        "#,
    )
    .expect("prepared asset should parse");
    assert_eq!(prepared.kind, PreparedAssetKind::Sprite2d);
    assets.mark_prepared(prepared);
    let runtime = RhaiScriptRuntime::new(
        Some(scene),
        None,
        Some(assets),
        None,
        Some(launch_selection),
        Some(catalog),
        Some(diagnostics),
        Some(command_queue.clone()),
        None,
        None,
    );

    runtime
        .execute(
            "catalog-script",
            r#"
                let sprite = world.assets.get("playground-2d/images/sprite-lab");
                if world.entities.count() != 1 { throw("wrong entity count"); }
                if !world.assets.has("playground-2d/images/sprite-lab") { throw("world assets missing key"); }
                if world.assets.registered().len != 1 { throw("wrong registered asset count"); }
                if world.assets.by_mod("playground-2d").len != 1 { throw("wrong world mod asset count"); }
                if world.assets.pending().len != 0 { throw("wrong world pending asset count"); }
                if world.assets.loaded().len != 1 { throw("wrong world loaded asset count"); }
                if world.assets.prepared().len != 1 { throw("wrong world prepared asset count"); }
                if world.assets.failed().len != 0 { throw("wrong world failed asset count"); }
                if sprite.key() != "playground-2d/images/sprite-lab" { throw("wrong asset key"); }
                if !sprite.exists() { throw("asset ref should exist"); }
                if sprite.state() != "prepared" { throw("wrong asset ref state"); }
                if sprite.source() != "mod:playground-2d" { throw("wrong asset ref source"); }
                if sprite.path().len == 0 { throw("missing asset ref path"); }
                if sprite.kind() != "sprite-2d" { throw("wrong asset ref kind"); }
                if sprite.label() != "Sprite Lab Placeholder" { throw("wrong asset ref label"); }
                if sprite.format() != "debug-placeholder" { throw("wrong asset ref format"); }
                if sprite.tags().len != 3 { throw("wrong asset ref tags"); }
                if sprite.reason().len != 0 { throw("unexpected asset ref reason"); }
                if !world.assets.reload("playground-2d/images/sprite-lab") { throw("failed to queue world asset reload"); }
                if !sprite.reload() { throw("failed to queue asset ref reload"); }

                if world.mod.current_id() != "playground-2d" { throw("wrong current mod"); }
                if world.mod.scenes().len != 2 { throw("wrong world scene count"); }
                if !world.mod.has_scene("text-lab") { throw("missing world mod scene"); }
                if world.mod.capabilities().len != 1 { throw("wrong world capability count"); }
                if world.mod.loaded().len != 2 { throw("wrong world loaded mod count"); }

                if world.runtime.window_backend() != "winit" { throw("wrong world window backend"); }
                if world.runtime.input_backend() != "winit" { throw("wrong world input backend"); }
                if world.runtime.render_backend() != "wgpu" { throw("wrong world render backend"); }
                if world.runtime.script_backend() != "rhai" { throw("wrong world script backend"); }
                if world.runtime.capabilities().len != 2 { throw("wrong world runtime capability count"); }
                if world.runtime.plugins().len != 2 { throw("wrong world runtime plugin count"); }
                if world.runtime.services().len != 1 { throw("wrong world runtime service count"); }
                if !world.runtime.dev_mode() { throw("world runtime should be in dev mode"); }
            "#,
        )
        .expect("script should be able to inspect runtime catalog and diagnostics");

    assert_eq!(command_queue.pending().len(), 2);
    assert_eq!(command_queue.pending()[0].namespace, "asset");
    assert_eq!(command_queue.pending()[1].namespace, "asset");
}

#[test]
fn queues_placeholder_script_and_console_messages() {
    let command_queue = Arc::new(ScriptCommandQueue::default());
    let event_queue = Arc::new(ScriptEventQueue::default());
    let console_queue = Arc::new(DevConsoleQueue::default());
    let launch_selection = Arc::new(LaunchSelection::new(
        Some("playground-2d".to_owned()),
        Some("sprite-lab".to_owned()),
        vec!["core".to_owned(), "playground-2d".to_owned()],
        true,
    ));
    let runtime = RhaiScriptRuntime::new(
        None,
        None,
        None,
        None,
        Some(launch_selection),
        None,
        None,
        Some(command_queue.clone()),
        Some(event_queue.clone()),
        Some(console_queue.clone()),
    );

    runtime
        .execute(
            "queue-script",
            r#"
                world.scene.select("dev-shell");
                world.scene.reload();
                world.assets.reload("playground-2d/images/sprite-lab");
                world.dev.event("scene.selected", "dev-shell");
                world.dev.command("help");
                world.sprite2d.queue("playground-2d-sprite", "playground-2d/images/sprite-lab", 128, 128);
                world.text2d.queue("playground-2d-label", "AMIGO 2D", "playground-2d/fonts/debug-ui", 320, 64);
                world.mesh3d.queue("playground-3d-probe", "playground-3d/meshes/probe");
                world.material3d.bind("playground-3d-probe", "debug-surface", "playground-3d/materials/debug-surface");
            "#,
        )
        .expect("script should be able to queue placeholder bridge messages");

    assert_eq!(command_queue.pending().len(), 7);
    assert_eq!(event_queue.pending().len(), 1);
    assert_eq!(console_queue.pending().len(), 1);
    assert_eq!(command_queue.pending()[1].namespace, "scene".to_owned());
    assert_eq!(command_queue.pending()[2].namespace, "asset".to_owned());
    assert_eq!(command_queue.pending()[3].namespace, "2d.sprite".to_owned());
    assert_eq!(command_queue.pending()[4].namespace, "2d.text".to_owned());
    assert_eq!(command_queue.pending()[5].namespace, "3d.mesh".to_owned());
    assert_eq!(
        command_queue.pending()[6].namespace,
        "3d.material".to_owned()
    );
}

#[test]
fn queues_world_content_domain_commands() {
    let command_queue = Arc::new(ScriptCommandQueue::default());
    let launch_selection = Arc::new(LaunchSelection::new(
        Some("playground-3d".to_owned()),
        Some("hello-world-cube".to_owned()),
        vec!["core".to_owned(), "playground-3d".to_owned()],
        true,
    ));
    let runtime = RhaiScriptRuntime::new(
        None,
        None,
        None,
        None,
        Some(launch_selection),
        None,
        None,
        Some(command_queue.clone()),
        None,
        None,
    );

    runtime
        .execute(
            "world-content-script",
            r#"
                world.sprite2d.queue("playground-2d-sprite", "playground-2d/images/sprite-lab", 128, 128);
                world.text2d.queue("playground-2d-label", "AMIGO 2D", "playground-2d/fonts/debug-ui", 320, 64);
                world.mesh3d.queue("playground-3d-probe", "playground-3d/meshes/probe");
                world.material3d.bind("playground-3d-probe", "debug-surface", "playground-3d/materials/debug-surface");
                world.text3d.queue("playground-3d-hello", "HELLO WORLD", "playground-3d/fonts/debug-3d", 0.5);
                world.dev.refresh_diagnostics("playground-3d");
            "#,
        )
        .expect("script should be able to queue world content domain commands");

    assert_eq!(command_queue.pending().len(), 6);
    assert_eq!(command_queue.pending()[0].namespace, "2d.sprite".to_owned());
    assert_eq!(command_queue.pending()[1].namespace, "2d.text".to_owned());
    assert_eq!(command_queue.pending()[2].namespace, "3d.mesh".to_owned());
    assert_eq!(
        command_queue.pending()[3].namespace,
        "3d.material".to_owned()
    );
    assert_eq!(command_queue.pending()[4].namespace, "3d.text".to_owned());
    assert_eq!(command_queue.pending()[5].namespace, "dev-shell".to_owned());
}

#[test]
fn queues_world_ui_commands() {
    let command_queue = Arc::new(ScriptCommandQueue::default());
    let runtime = RhaiScriptRuntime::new(
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(command_queue.clone()),
        None,
        None,
    );

    runtime
        .execute(
            "world-ui-script",
            r#"
                if !world.ui.set_text("playground-2d-ui-preview.subtitle", "Updated from Rhai") {
                    throw("set_text should queue a command");
                }
                if !world.ui.set_value("playground-2d-ui-preview.hp-bar", 0.5) {
                    throw("set_value should queue a command");
                }
                if !world.ui.show("playground-2d-ui-preview.root") {
                    throw("show should queue a command");
                }
                if !world.ui.hide("playground-2d-ui-preview.root") {
                    throw("hide should queue a command");
                }
                if !world.ui.enable("playground-2d-ui-preview.root.control-card.button-row.repair-button") {
                    throw("enable should queue a command");
                }
                if !world.ui.disable("playground-2d-ui-preview.root.control-card.button-row.repair-button") {
                    throw("disable should queue a command");
                }
                let hud = #{};
                hud["playground-2d-ui-preview.score"] = "Score: 10";
                hud["playground-2d-ui-preview.status"] = "Ready";
                if world.ui.set_many(hud) != 2 {
                    throw("set_many should queue two commands");
                }
            "#,
        )
        .expect("script should be able to queue world ui commands");

    assert_eq!(command_queue.pending().len(), 8);
    assert_eq!(command_queue.pending()[0].namespace, "ui".to_owned());
    assert_eq!(command_queue.pending()[0].name, "set-text".to_owned());
    assert_eq!(
        command_queue.pending()[0].arguments,
        vec![
            "playground-2d-ui-preview.subtitle".to_owned(),
            "Updated from Rhai".to_owned(),
        ]
    );
    assert_eq!(command_queue.pending()[1].name, "set-value".to_owned());
    assert_eq!(
        command_queue.pending()[1].arguments,
        vec![
            "playground-2d-ui-preview.hp-bar".to_owned(),
            "0.5".to_owned(),
        ]
    );
    assert_eq!(command_queue.pending()[2].name, "show".to_owned());
    assert_eq!(command_queue.pending()[3].name, "hide".to_owned());
    assert_eq!(command_queue.pending()[4].name, "enable".to_owned());
    assert_eq!(command_queue.pending()[5].name, "disable".to_owned());
    assert_eq!(command_queue.pending()[6].name, "set-text".to_owned());
    assert_eq!(
        command_queue.pending()[6].arguments,
        vec![
            "playground-2d-ui-preview.score".to_owned(),
            "Score: 10".to_owned(),
        ]
    );
    assert_eq!(command_queue.pending()[7].name, "set-text".to_owned());
    assert_eq!(
        command_queue.pending()[7].arguments,
        vec![
            "playground-2d-ui-preview.status".to_owned(),
            "Ready".to_owned(),
        ]
    );
}

#[test]
fn queues_world_audio_commands() {
    let command_queue = Arc::new(ScriptCommandQueue::default());
    let runtime = RhaiScriptRuntime::new(
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(command_queue.clone()),
        None,
        None,
    );

    runtime
        .execute(
            "world-audio-script",
            r#"
                if !world.audio.preload("jump") { throw("preload should queue"); }
                if !world.audio.play("jump") { throw("play should queue"); }
                if !world.audio.play_asset("playground-sidescroller/audio/coin") { throw("play_asset should queue"); }
                if !world.audio.start_realtime("proximity-beep") { throw("start_realtime should queue"); }
                if !world.audio.set_param("proximity-beep", "distance", 128.0) { throw("set_param should queue"); }
                if !world.audio.set_volume("master", 0.75) { throw("set_volume should queue"); }
                if !world.audio.stop("proximity-beep") { throw("stop should queue"); }
            "#,
        )
        .expect("script should be able to queue world audio commands");

    assert_eq!(command_queue.pending().len(), 7);
    assert_eq!(
        command_queue.pending()[0],
        ScriptCommand::audio_preload("jump")
    );
    assert_eq!(
        command_queue.pending()[1],
        ScriptCommand::audio_play("jump")
    );
    assert_eq!(
        command_queue.pending()[2],
        ScriptCommand::audio_play_asset("playground-sidescroller/audio/coin")
    );
    assert_eq!(
        command_queue.pending()[3],
        ScriptCommand::audio_start_realtime("proximity-beep")
    );
    assert_eq!(
        command_queue.pending()[4],
        ScriptCommand::audio_set_param("proximity-beep", "distance", 128.0)
    );
    assert_eq!(
        command_queue.pending()[5],
        ScriptCommand::audio_set_volume("master", 0.75)
    );
    assert_eq!(
        command_queue.pending()[6],
        ScriptCommand::audio_stop("proximity-beep")
    );
}

#[test]
fn rejects_invalid_world_ui_commands() {
    let command_queue = Arc::new(ScriptCommandQueue::default());
    let runtime = RhaiScriptRuntime::new(
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(command_queue.clone()),
        None,
        None,
    );

    runtime
        .execute(
            "world-ui-invalid-script",
            r#"
                if world.ui.set_text("", "Updated from Rhai") { throw("empty path should fail"); }
                if world.ui.show("") { throw("empty show path should fail"); }
                if world.ui.hide("") { throw("empty hide path should fail"); }
                if world.ui.enable("") { throw("empty enable path should fail"); }
                if world.ui.disable("") { throw("empty disable path should fail"); }
                let hud = #{};
                hud[""] = "empty path";
                if world.ui.set_many(hud) != 0 { throw("empty set_many path should fail"); }
            "#,
        )
        .expect("invalid ui script should still execute");

    assert!(
        command_queue.pending().is_empty(),
        "invalid ui commands should not enqueue anything"
    );
}
