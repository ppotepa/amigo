use super::*;

#[test]
fn handle_script_command_asset_reload_requests_load_and_event() {
    let scene_command_queue = SceneCommandQueue::default();
    let script_event_queue = ScriptEventQueue::default();
    let dev_console_state = DevConsoleState::default();
    let asset_catalog = AssetCatalog::default();
    let ui_state = UiStateService::default();
    let audio_command_queue = AudioCommandQueue::default();
    let audio_scene_service = AudioSceneService::default();
    let diagnostics = RuntimeDiagnostics::default();
    let launch_selection = LaunchSelection::new(
        Some("playground-sidescroller".to_owned()),
        Some("vertical-slice".to_owned()),
        Vec::new(),
        true,
    );
    asset_catalog.register_manifest(AssetManifest {
        key: AssetKey::new("playground-sidescroller/audio/jump"),
        source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
        tags: vec!["audio".to_owned(), "generated".to_owned()],
    });

    script_runtime::dispatch_script_command(
        ScriptCommand::new(
            "asset",
            "reload",
            vec!["playground-sidescroller/audio/jump".to_owned()],
        ),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );

    assert!(
        asset_catalog
            .pending_loads()
            .iter()
            .any(|request| request.key.as_str() == "playground-sidescroller/audio/jump")
    );
    assert!(script_event_queue.pending().iter().any(|event| {
        event.topic == "asset.reload-requested"
            && event.payload == vec!["playground-sidescroller/audio/jump".to_owned()]
    }));
}

#[test]
fn handle_script_command_queues_and_processes_audio_state() {
    let scene_command_queue = SceneCommandQueue::default();
    let script_event_queue = ScriptEventQueue::default();
    let dev_console_state = DevConsoleState::default();
    let asset_catalog = AssetCatalog::default();
    let ui_state = UiStateService::default();
    let audio_command_queue = AudioCommandQueue::default();
    let audio_scene_service = AudioSceneService::default();
    let audio_state = AudioStateService::default();
    let diagnostics = RuntimeDiagnostics::default();
    let launch_selection = LaunchSelection::new(
        Some("playground-sidescroller".to_owned()),
        Some("vertical-slice".to_owned()),
        Vec::new(),
        true,
    );

    script_runtime::dispatch_script_command(
        ScriptCommand::audio_play("jump"),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );
    script_runtime::dispatch_script_command(
        ScriptCommand::audio_start_realtime("proximity-beep"),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );
    script_runtime::dispatch_script_command(
        ScriptCommand::audio_set_param("proximity-beep", "distance", 128.0),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );

    let commands = audio_command_queue.drain();
    assert_eq!(commands.len(), 3);
    assert_eq!(audio_scene_service.clips().len(), 2);

    for command in commands {
        process_audio_command(command, &audio_state, &dev_console_state);
    }

    assert!(audio_state.playing_sources().contains_key("proximity-beep"));
    assert_eq!(audio_state.drain_runtime_commands().len(), 3);
    assert_eq!(
        audio_state
            .source_params()
            .get("proximity-beep")
            .and_then(|params| params.get("distance"))
            .copied(),
        Some(128.0)
    );
}

#[test]
fn handle_script_command_queues_scene_commands() {
    let scene_command_queue = SceneCommandQueue::default();
    let script_event_queue = ScriptEventQueue::default();
    let dev_console_state = DevConsoleState::default();
    let asset_catalog = AssetCatalog::default();
    let ui_state = UiStateService::default();
    let audio_command_queue = AudioCommandQueue::default();
    let audio_scene_service = AudioSceneService::default();
    let diagnostics = RuntimeDiagnostics::default();
    let launch_selection = LaunchSelection::new(
        Some("playground-2d".to_owned()),
        Some("screen-space-preview".to_owned()),
        Vec::new(),
        true,
    );

    script_runtime::dispatch_script_command(
        ScriptCommand::new("scene", "select", vec!["sprite-showcase".to_owned()]),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );
    script_runtime::dispatch_script_command(
        ScriptCommand::new("scene", "reload", Vec::new()),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );
    script_runtime::dispatch_script_command(
        ScriptCommand::new("scene", "spawn", vec!["runtime-test-entity".to_owned()]),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );
    script_runtime::dispatch_script_command(
        ScriptCommand::new("scene", "clear", Vec::new()),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );

    let commands = scene_command_queue.pending();
    assert!(matches!(
        commands.first(),
        Some(SceneCommand::SelectScene { scene }) if scene.as_str() == "sprite-showcase"
    ));
    assert!(matches!(
        commands.get(1),
        Some(SceneCommand::ReloadActiveScene)
    ));
    assert!(matches!(
        commands.get(2),
        Some(SceneCommand::SpawnNamedEntity { name, transform }) if name == "runtime-test-entity" && transform.is_none()
    ));
    assert!(matches!(commands.get(3), Some(SceneCommand::ClearEntities)));
}

#[test]
fn handle_script_command_unknown_command_reports_fallback() {
    let scene_command_queue = SceneCommandQueue::default();
    let script_event_queue = ScriptEventQueue::default();
    let dev_console_state = DevConsoleState::default();
    let asset_catalog = AssetCatalog::default();
    let ui_state = UiStateService::default();
    let audio_command_queue = AudioCommandQueue::default();
    let audio_scene_service = AudioSceneService::default();
    let diagnostics = RuntimeDiagnostics::default();
    let launch_selection = LaunchSelection::new(
        Some("playground-2d".to_owned()),
        Some("screen-space-preview".to_owned()),
        Vec::new(),
        true,
    );

    script_runtime::dispatch_script_command(
        ScriptCommand::new("unknown", "noop", vec!["x".to_owned()]),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );

    assert!(
        dev_console_state
            .output_lines()
            .iter()
            .any(|line| line.contains("unhandled placeholder script command: unknown.noop(x)"))
    );
}

#[test]
fn handle_script_command_updates_ui_state() {
    let scene_command_queue = SceneCommandQueue::default();
    let script_event_queue = ScriptEventQueue::default();
    let dev_console_state = DevConsoleState::default();
    let asset_catalog = AssetCatalog::default();
    let ui_state = UiStateService::default();
    let audio_command_queue = AudioCommandQueue::default();
    let audio_scene_service = AudioSceneService::default();
    let diagnostics = RuntimeDiagnostics::default();
    let launch_selection = LaunchSelection::new(
        Some("playground-2d".to_owned()),
        Some("screen-space-preview".to_owned()),
        Vec::new(),
        true,
    );

    script_runtime::dispatch_script_command(
        ScriptCommand::ui_set_text("playground-2d-ui-preview.subtitle", "Updated from Rhai"),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );
    script_runtime::dispatch_script_command(
        ScriptCommand::ui_set_value("playground-2d-ui-preview.hp-bar", 0.5),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );
    script_runtime::dispatch_script_command(
        ScriptCommand::ui_hide("playground-2d-ui-preview.root"),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );
    script_runtime::dispatch_script_command(
        ScriptCommand::ui_disable(
            "playground-2d-ui-preview.root.control-card.button-row.repair-button",
        ),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );
    script_runtime::dispatch_script_command(
        ScriptCommand::ui_enable(
            "playground-2d-ui-preview.root.control-card.button-row.repair-button",
        ),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );

    assert_eq!(
        ui_state
            .text_override("playground-2d-ui-preview.subtitle")
            .as_deref(),
        Some("Updated from Rhai")
    );
    assert_eq!(
        ui_state.value_override("playground-2d-ui-preview.hp-bar"),
        Some(0.5)
    );
    assert!(!ui_state.is_visible("playground-2d-ui-preview.root"));
    assert!(
        ui_state.is_enabled("playground-2d-ui-preview.root.control-card.button-row.repair-button")
    );
}

#[test]
fn handle_script_command_writes_debug_text_export() {
    let scene_command_queue = SceneCommandQueue::default();
    let script_event_queue = ScriptEventQueue::default();
    let dev_console_state = DevConsoleState::default();
    let asset_catalog = AssetCatalog::default();
    let ui_state = UiStateService::default();
    let audio_command_queue = AudioCommandQueue::default();
    let audio_scene_service = AudioSceneService::default();
    let diagnostics = RuntimeDiagnostics::default();
    let launch_selection = LaunchSelection::new(None, None, Vec::new(), true);
    let relative_path = format!(
        "tests/debug-export-{}.txt",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos()
    );
    let target_path = PathBuf::from("target")
        .join("amigo-dev-exports")
        .join(&relative_path);
    if target_path.exists() {
        fs::remove_file(&target_path).expect("stale debug export should be removable");
    }

    script_runtime::dispatch_script_command(
        ScriptCommand::new(
            "debug",
            "write-text",
            vec![relative_path.clone(), "hello export".to_owned()],
        ),
        &scene_command_queue,
        &script_event_queue,
        &dev_console_state,
        &asset_catalog,
        &ui_state,
        &audio_command_queue,
        &audio_scene_service,
        &diagnostics,
        &launch_selection,
    );

    assert_eq!(
        fs::read_to_string(&target_path).expect("debug export should be written"),
        "hello export"
    );
}

#[test]
fn resolve_existing_asset_path_prefers_metadata_candidates() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("amigo-asset-path-{unique}"));
    fs::create_dir_all(root.join("textures")).expect("temp textures dir should exist");

    let metadata_path = root.join("assets").join("sprites").join("player.sprite.yml");
    fs::write(&metadata_path, "kind: sprite-sheet-2d\nimage: player.png\n")
        .expect("metadata file should be created");

    let resolved = crate::assets::resolve_existing_asset_path(
        root.join("textures").join("player"),
        "test/player",
    )
    .expect("metadata candidate should resolve");

    assert_eq!(resolved, metadata_path);
}
