use super::*;
use amigo_runtime_bundles::RenderLayer2dSceneService;
use amigo_runtime_bundles::{
    can_handle_layered_image_script_command, handle_layered_image_script_command,
    LayeredImageSceneService, LayeredImageScriptCommandContext, LayeredImageScriptCommandOutcome,
};
use amigo_runtime_bundles::{GlobalLight2dSceneService, LightGroup2dSceneService};
use amigo_runtime_bundles::{handle_ui_script_command, UiScriptCommandContext, UiStateService};

#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_script_command(
    command: ScriptCommand,
    scene_command_queue: &amigo_scene::SceneCommandQueue,
    script_event_queue: &amigo_scripting_api::ScriptEventQueue,
    dev_console_state: &amigo_scripting_api::DevConsoleState,
    asset_catalog: &amigo_assets::AssetCatalog,
    ui_state: &UiStateService,
    audio_command_queue: &amigo_runtime_bundles::AudioCommandQueue,
    audio_scene_service: &amigo_runtime_bundles::AudioSceneService,
    _diagnostics: &RuntimeDiagnostics,
    launch_selection: &LaunchSelection,
) {
    match command.namespace.as_str() {
        "asset" => {
            let _ = amigo_assets::handle_asset_script_command(
                amigo_assets::AssetScriptCommandContext {
                    asset_catalog,
                    script_event_queue,
                },
                command,
            );
        }
        "scene" => {
            let _ = amigo_scene::handle_scene_script_command(
                amigo_scene::SceneScriptCommandContext {
                    scene_command_queue,
                },
                command,
            );
        }
        "ui" => {
            let _ = handle_ui_script_command(
                UiScriptCommandContext {
                    ui_state_service: ui_state,
                },
                command,
            );
        }
        "audio" => dispatch_audio_script_command_for_test(
            command,
            audio_command_queue,
            audio_scene_service,
            launch_selection,
        ),
        "debug" => dispatch_debug_script_command_for_test(command, dev_console_state),
        _ => dev_console_state.write_line(format!(
            "unhandled placeholder script command: {}.{}({})",
            command.namespace,
            command.name,
            command.arguments.join(", ")
        )),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_script_command_with_layered_image_service(
    command: ScriptCommand,
    scene_command_queue: &amigo_scene::SceneCommandQueue,
    script_event_queue: &amigo_scripting_api::ScriptEventQueue,
    dev_console_state: &amigo_scripting_api::DevConsoleState,
    asset_catalog: &amigo_assets::AssetCatalog,
    layered_images: &LayeredImageSceneService,
    _render_layers: &RenderLayer2dSceneService,
    _global_lights: &GlobalLight2dSceneService,
    _light_groups: &LightGroup2dSceneService,
    ui_state: &UiStateService,
    audio_command_queue: &amigo_runtime_bundles::AudioCommandQueue,
    audio_scene_service: &amigo_runtime_bundles::AudioSceneService,
    diagnostics: &RuntimeDiagnostics,
    launch_selection: &LaunchSelection,
) {
    if can_handle_layered_image_script_command(
        &command,
    ) {
        let outcome =
            handle_layered_image_script_command(
                LayeredImageScriptCommandContext {
                    layered_image_scene_service: layered_images,
                },
                command,
            );

        match outcome {
            LayeredImageScriptCommandOutcome::Updated(message)
            | LayeredImageScriptCommandOutcome::ParseError(message) => {
                dev_console_state.write_line(message);
            }
            LayeredImageScriptCommandOutcome::Unhandled => {}
        }
        return;
    }

    dispatch_script_command(
        command,
        scene_command_queue,
        script_event_queue,
        dev_console_state,
        asset_catalog,
        ui_state,
        audio_command_queue,
        audio_scene_service,
        diagnostics,
        launch_selection,
    );
}

fn dispatch_audio_script_command_for_test(
    command: ScriptCommand,
    audio_command_queue: &amigo_runtime_bundles::AudioCommandQueue,
    audio_scene_service: &amigo_runtime_bundles::AudioSceneService,
    launch_selection: &LaunchSelection,
) {
    let outcome = amigo_runtime_bundles::handle_audio_script_command(
        amigo_runtime_bundles::AudioScriptCommandContext {
            audio_command_queue,
            audio_scene_service,
        },
        command,
        |clip_name| resolve_test_asset_key(launch_selection, clip_name),
    );

    match outcome {
        amigo_runtime_bundles::AudioScriptCommandOutcome::PlayOnce {
            asset_key,
        }
        | amigo_runtime_bundles::AudioScriptCommandOutcome::SourceStarted {
            asset_key,
            ..
        }
        | amigo_runtime_bundles::AudioScriptCommandOutcome::Preloaded {
            asset_key,
            ..
        } => {
            audio_scene_service.register_clip(amigo_runtime_bundles::AudioClip {
                key: amigo_runtime_bundles::AudioClipKey::new(
                    asset_key.as_str().to_owned(),
                ),
                mode: amigo_runtime_bundles::AudioPlaybackMode::OneShot,
            });
        }
        _ => {}
    }
}

fn dispatch_debug_script_command_for_test(
    command: ScriptCommand,
    dev_console_state: &amigo_scripting_api::DevConsoleState,
) {
    match (command.name.as_str(), command.arguments.as_slice()) {
        ("write-text", [relative_path, contents]) | ("write_text", [relative_path, contents]) => {
            let relative = std::path::Path::new(relative_path);
            let safe = !relative.is_absolute()
                && relative
                    .components()
                    .all(|component| matches!(component, std::path::Component::Normal(_)));
            if safe {
                let path = std::path::PathBuf::from("target")
                    .join("amigo-dev-exports")
                    .join(relative);
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(path, contents);
            } else {
                dev_console_state
                    .write_line(format!("refused unsafe text export path `{relative_path}`"));
            }
        }
        _ => dev_console_state.write_line(format!(
            "debug could not handle command: {}:{} {:?}",
            command.namespace, command.name, command.arguments
        )),
    }
}

fn resolve_test_asset_key(launch_selection: &LaunchSelection, asset_name: &str) -> AssetKey {
    if asset_name.contains('/') {
        return AssetKey::new(asset_name.to_owned());
    }

    launch_selection
        .startup_mod
        .as_ref()
        .map(|root_mod| AssetKey::new(format!("{root_mod}/audio/{asset_name}")))
        .unwrap_or_else(|| AssetKey::new(asset_name.to_owned()))
}
