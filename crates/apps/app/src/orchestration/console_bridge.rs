use super::*;

pub(crate) fn handle_console_command(
    command: amigo_scripting_api::DevConsoleCommand,
    scene_command_queue: &SceneCommandQueue,
    script_event_queue: &ScriptEventQueue,
    dev_console_state: &DevConsoleState,
    runtime_diagnostics: &RuntimeDiagnostics,
    asset_catalog: &AssetCatalog,
) {
    dev_console_state.record_command(command.line.clone());

    let parts = command
        .line
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();

    match parts.as_slice() {
        [help] if help == "help" => {
            dev_console_state.write_line(
                "available placeholder commands: help, diagnostics, assets, scene select <scene-id>, scene reload, asset reload <asset-key>",
            );
        }
        [diagnostics] if diagnostics == "diagnostics" => {
            dev_console_state.write_line(format!(
                "window={} input={} render={} script={} loaded_mods={} assets_loaded={} assets_prepared={} assets_failed={} assets_pending={}",
                runtime_diagnostics.window_backend,
                runtime_diagnostics.input_backend,
                runtime_diagnostics.render_backend,
                runtime_diagnostics.script_backend,
                runtime_diagnostics.loaded_mods.len(),
                asset_catalog.loaded_assets().len(),
                asset_catalog.prepared_assets().len(),
                asset_catalog.failed_assets().len(),
                asset_catalog.pending_loads().len()
            ));
        }
        [assets] if assets == "assets" => {
            let loaded = asset_catalog
                .loaded_assets()
                .into_iter()
                .map(|asset| asset.key.as_str().to_owned())
                .collect::<Vec<_>>();
            let prepared = asset_catalog
                .prepared_assets()
                .into_iter()
                .map(|asset| format!("{} ({})", asset.key.as_str(), asset.kind.as_str()))
                .collect::<Vec<_>>();
            let failed = asset_catalog
                .failed_assets()
                .into_iter()
                .map(|asset| format!("{}: {}", asset.key.as_str(), asset.reason))
                .collect::<Vec<_>>();
            let pending = asset_catalog
                .pending_loads()
                .into_iter()
                .map(|request| request.key.as_str().to_owned())
                .collect::<Vec<_>>();

            dev_console_state.write_line(format!(
                "assets loaded={} prepared={} failed={} pending={}",
                crate::app_helpers::display_string_list(&loaded),
                crate::app_helpers::display_string_list(&prepared),
                crate::app_helpers::display_string_list(&failed),
                crate::app_helpers::display_string_list(&pending)
            ));
        }
        [scene, select, scene_id] if scene == "scene" && select == "select" => {
            scene_command_queue.submit(SceneCommand::SelectScene {
                scene: SceneKey::new(scene_id.clone()),
            });
            script_event_queue.publish(ScriptEvent::new(
                "dev-console.scene-select-requested",
                vec![scene_id.clone()],
            ));
            dev_console_state.write_line(format!(
                "queued placeholder scene selection for `{scene_id}`"
            ));
        }
        [scene, reload] if scene == "scene" && reload == "reload" => {
            scene_command_queue.submit(SceneCommand::ReloadActiveScene);
            script_event_queue.publish(ScriptEvent::new(
                "dev-console.scene-reload-requested",
                Vec::<String>::new(),
            ));
            dev_console_state.write_line("queued placeholder reload for the active scene");
        }
        [asset, reload, asset_key] if asset == "asset" && reload == "reload" => {
            request_asset_reload(
                asset_catalog,
                asset_key,
                AssetLoadPriority::Immediate,
                dev_console_state,
            );
            script_event_queue.publish(ScriptEvent::new(
                "dev-console.asset-reload-requested",
                vec![asset_key.clone()],
            ));
        }
        _ => {
            dev_console_state.write_line(format!(
                "unknown placeholder console command: {}",
                command.line
            ));
        }
    }
}

pub(crate) fn request_asset_reload(
    asset_catalog: &AssetCatalog,
    asset_key: &str,
    priority: AssetLoadPriority,
    dev_console_state: &DevConsoleState,
) {
    let asset_key = AssetKey::new(asset_key);
    if asset_catalog.manifest(&asset_key).is_none() {
        dev_console_state.write_line(format!(
            "cannot reload unknown asset `{}`",
            asset_key.as_str()
        ));
        return;
    }

    asset_catalog.request_reload(AssetLoadRequest::new(asset_key.clone(), priority));
    dev_console_state.write_line(format!("queued asset reload for `{}`", asset_key.as_str()));
}
