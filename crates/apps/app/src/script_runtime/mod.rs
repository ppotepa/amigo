//! App-side scripting runtime integration.
//! This module now delegates script command dispatch to engine-registered
//! `RuntimeScriptCommandHandler` implementations.

use super::*;
use amigo_runtime::{HandlerDispatcher, Runtime, RuntimePlugin};
use amigo_session::RuntimeSession;

pub(crate) struct ScriptCommandRuntimePlugin;

impl RuntimePlugin for ScriptCommandRuntimePlugin {
    fn name(&self) -> &'static str {
        "amigo-app-script-command-registry"
    }

    fn register(&self, _services: &mut ServiceRegistry) -> AmigoResult<()> {
        Ok(())
    }
}

pub(crate) fn dispatch_script_command_with_runtime(
    runtime: &Runtime,
    command: ScriptCommand,
) -> AmigoResult<()> {
    let dev_console_state = required::<DevConsoleState>(runtime)?;
    let handlers =
        runtime.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;

    let result = HandlerDispatcher::new(handlers).dispatch_first(|handler| {
        handler
            .can_handle(&command)
            .then(|| handler.handle(runtime, command.clone()))
    });

    if let Some(result) = result {
        result?;
    } else {
        dev_console_state.write_line(format!(
            "unhandled script command: {}",
            crate::app_helpers::format_script_command(&command)
        ));
    }

    Ok(())
}

pub(crate) fn dispatch_script_command_for_session(
    session: &RuntimeSession,
    command: ScriptCommand,
) -> AmigoResult<()> {
    let command_label = crate::app_helpers::format_script_command(&command);
    session.begin_script_command_dispatch(command_label.clone());
    if let Err(error) = dispatch_script_command_with_runtime(session.runtime(), command) {
        session.mark_script_dispatch_error(command_label, error.to_string());
        return Err(error);
    }

    session.complete_script_command_dispatch();
    Ok(())
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_script_command(
    command: ScriptCommand,
    scene_command_queue: &amigo_scene::SceneCommandQueue,
    script_event_queue: &amigo_scripting_api::ScriptEventQueue,
    dev_console_state: &amigo_scripting_api::DevConsoleState,
    asset_catalog: &amigo_assets::AssetCatalog,
    ui_state: &amigo_runtime_bundles::amigo_ui::UiStateService,
    audio_command_queue: &amigo_runtime_bundles::amigo_audio_api::AudioCommandQueue,
    audio_scene_service: &amigo_runtime_bundles::amigo_audio_api::AudioSceneService,
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
            let _ = amigo_runtime_bundles::amigo_ui::handle_ui_script_command(
                amigo_runtime_bundles::amigo_ui::UiScriptCommandContext {
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

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_script_command_with_layered_image_service(
    command: ScriptCommand,
    scene_command_queue: &amigo_scene::SceneCommandQueue,
    script_event_queue: &amigo_scripting_api::ScriptEventQueue,
    dev_console_state: &amigo_scripting_api::DevConsoleState,
    asset_catalog: &amigo_assets::AssetCatalog,
    layered_images: &amigo_runtime_bundles::amigo_2d_layered_image::LayeredImageSceneService,
    _render_layers: &amigo_runtime_bundles::amigo_2d_composition::RenderLayer2dSceneService,
    _global_lights: &amigo_runtime_bundles::amigo_2d_lighting::GlobalLight2dSceneService,
    _light_groups: &amigo_runtime_bundles::amigo_2d_lighting::LightGroup2dSceneService,
    ui_state: &amigo_runtime_bundles::amigo_ui::UiStateService,
    audio_command_queue: &amigo_runtime_bundles::amigo_audio_api::AudioCommandQueue,
    audio_scene_service: &amigo_runtime_bundles::amigo_audio_api::AudioSceneService,
    diagnostics: &RuntimeDiagnostics,
    launch_selection: &LaunchSelection,
) {
    if amigo_runtime_bundles::amigo_2d_layered_image::can_handle_layered_image_script_command(
        &command,
    ) {
        let outcome =
            amigo_runtime_bundles::amigo_2d_layered_image::handle_layered_image_script_command(
                amigo_runtime_bundles::amigo_2d_layered_image::LayeredImageScriptCommandContext {
                    layered_image_scene_service: layered_images,
                },
                command,
            );

        match outcome {
            amigo_runtime_bundles::amigo_2d_layered_image::LayeredImageScriptCommandOutcome::Updated(message)
            | amigo_runtime_bundles::amigo_2d_layered_image::LayeredImageScriptCommandOutcome::ParseError(message) => {
                dev_console_state.write_line(message);
            }
            amigo_runtime_bundles::amigo_2d_layered_image::LayeredImageScriptCommandOutcome::Unhandled => {}
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

#[cfg(test)]
fn dispatch_audio_script_command_for_test(
    command: ScriptCommand,
    audio_command_queue: &amigo_runtime_bundles::amigo_audio_api::AudioCommandQueue,
    audio_scene_service: &amigo_runtime_bundles::amigo_audio_api::AudioSceneService,
    launch_selection: &LaunchSelection,
) {
    let outcome = amigo_runtime_bundles::amigo_audio_api::handle_audio_script_command(
        amigo_runtime_bundles::amigo_audio_api::AudioScriptCommandContext {
            audio_command_queue,
            audio_scene_service,
        },
        command,
        |clip_name| resolve_test_asset_key(launch_selection, clip_name),
    );

    match outcome {
        amigo_runtime_bundles::amigo_audio_api::AudioScriptCommandOutcome::PlayOnce {
            asset_key,
        }
        | amigo_runtime_bundles::amigo_audio_api::AudioScriptCommandOutcome::SourceStarted {
            asset_key,
            ..
        }
        | amigo_runtime_bundles::amigo_audio_api::AudioScriptCommandOutcome::Preloaded {
            asset_key,
            ..
        } => {
            audio_scene_service.register_clip(amigo_runtime_bundles::amigo_audio_api::AudioClip {
                key: amigo_runtime_bundles::amigo_audio_api::AudioClipKey::new(
                    asset_key.as_str().to_owned(),
                ),
                mode: amigo_runtime_bundles::amigo_audio_api::AudioPlaybackMode::OneShot,
            });
        }
        _ => {}
    }
}

#[cfg(test)]
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

#[cfg(test)]
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
