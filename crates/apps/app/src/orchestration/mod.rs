//! Startup orchestration helpers for app-specific bridges.
//! This module wires auxiliary subsystems into the main runtime bootstrap path.

use super::*;
use crate::runtime_context::RuntimeContext;

mod audio_bridge;
mod console_bridge;

pub(crate) use audio_bridge::process_audio_command;
pub(crate) use console_bridge::{handle_console_command, request_asset_reload};

const MAX_PLACEHOLDER_BRIDGE_PASSES: usize = 16;
const MAX_RUNTIME_STABILIZATION_PASSES: usize = 16;

pub(crate) fn stabilize_runtime(runtime: &Runtime) -> AmigoResult<PlaceholderBridgeSummary> {
    let mut summary = PlaceholderBridgeSummary::default();

    for _ in 0..MAX_RUNTIME_STABILIZATION_PASSES {
        merge_placeholder_bridge_summary(&mut summary, process_placeholder_bridges(runtime)?);
        assets::process_pending_asset_loads(runtime)?;
        assets::sync_hot_reload_watches(runtime)?;

        if assets::queue_hot_reload_changes(runtime)? == 0 {
            let dev_console_state = required::<DevConsoleState>(runtime)?;
            summary.console_commands = dev_console_state.command_history();
            summary.console_output = dev_console_state.output_lines();
            return Ok(summary);
        }
    }

    Err(AmigoError::Message(format!(
        "runtime stabilization exceeded the maximum of {MAX_RUNTIME_STABILIZATION_PASSES} passes"
    )))
}

fn merge_placeholder_bridge_summary(
    target: &mut PlaceholderBridgeSummary,
    update: PlaceholderBridgeSummary,
) {
    target
        .processed_script_commands
        .extend(update.processed_script_commands);
    target
        .processed_audio_commands
        .extend(update.processed_audio_commands);
    target
        .processed_scene_commands
        .extend(update.processed_scene_commands);
    target
        .processed_script_events
        .extend(update.processed_script_events);
    target.console_commands = update.console_commands;
    target.console_output = update.console_output;
}

pub(crate) fn process_placeholder_bridges(
    runtime: &Runtime,
) -> AmigoResult<PlaceholderBridgeSummary> {
    let ctx = RuntimeContext::new(runtime);
    let script_command_queue = ctx.required::<ScriptCommandQueue>()?;
    let script_event_queue = ctx.required::<ScriptEventQueue>()?;
    let script_lifecycle_state = ctx.required::<ScriptLifecycleState>()?;
    let script_runtime = ctx.required::<ScriptRuntimeService>()?;
    let dev_console_queue = ctx.required::<DevConsoleQueue>()?;
    let dev_console_state = ctx.required::<DevConsoleState>()?;
    let scene_command_queue = ctx.required::<SceneCommandQueue>()?;
    let scene_service = ctx.required::<SceneService>()?;
    let scene_transition_service = ctx.required::<SceneTransitionService>()?;
    let asset_catalog = ctx.required::<AssetCatalog>()?;
    let audio_command_queue = ctx.required::<AudioCommandQueue>()?;
    let audio_state_service = ctx.required::<AudioStateService>()?;
    let diagnostics = ctx.required::<RuntimeDiagnostics>()?;
    let launch_selection = ctx.required::<LaunchSelection>()?;
    let mod_catalog = ctx.required::<ModCatalog>()?;

    let mut summary = PlaceholderBridgeSummary::default();

    for _ in 0..MAX_PLACEHOLDER_BRIDGE_PASSES {
        let mut made_progress = false;

        let script_commands = script_command_queue.drain();
        if !script_commands.is_empty() {
            made_progress = true;
        }
        for command in script_commands {
            summary
                .processed_script_commands
                .push(crate::app_helpers::format_script_command(&command));
            super::script_runtime::dispatch_script_command_with_runtime(runtime, command);
        }

        let console_commands = dev_console_queue.drain();
        if !console_commands.is_empty() {
            made_progress = true;
        }
        for command in console_commands {
            handle_console_command(
                command,
                scene_command_queue.as_ref(),
                script_event_queue.as_ref(),
                dev_console_state.as_ref(),
                diagnostics.as_ref(),
                asset_catalog.as_ref(),
            );
        }

        let script_events = script_event_queue.drain();
        if !script_events.is_empty() {
            made_progress = true;
        }
        for event in script_events {
            summary
                .processed_script_events
                .push(crate::app_helpers::format_script_event(&event));
            for command in
                scene_transition_service.observe_script_event(&event.topic, &event.payload)
            {
                scene_command_queue.submit(command);
            }
            crate::event_pipeline::run_event_pipelines_for_event(runtime, &event)?;
            crate::scripting_runtime::dispatch_script_event_to_active_scripts(
                script_runtime.as_ref(),
                mod_catalog.as_ref(),
                launch_selection.as_ref(),
                scene_service.as_ref(),
                &event,
            )?;
        }

        let audio_commands = audio_command_queue.drain();
        if !audio_commands.is_empty() {
            made_progress = true;
        }
        for command in audio_commands {
            summary
                .processed_audio_commands
                .push(crate::app_helpers::format_audio_command(&command));
            process_audio_command(
                command,
                audio_state_service.as_ref(),
                dev_console_state.as_ref(),
            );
        }

        let scene_commands = scene_command_queue.drain();
        if !scene_commands.is_empty() {
            made_progress = true;
        }
        for command in scene_commands {
            summary
                .processed_scene_commands
                .push(amigo_scene::format_scene_command(&command));
            super::scene_runtime::apply_scene_command(runtime, command)?;
        }

        if scene_command_queue.pending().is_empty()
            && crate::scripting_runtime::sync_active_scene_script_lifecycle(
                scene_service.as_ref(),
                script_lifecycle_state.as_ref(),
                script_runtime.as_ref(),
                mod_catalog.as_ref(),
                launch_selection.as_ref(),
            )?
        {
            made_progress = true;
        }

        if made_progress {
            crate::systems::ui_bindings::tick_ui_bindings(runtime)?;
        }

        if !made_progress {
            break;
        }
    }

    if !script_command_queue.pending().is_empty()
        || !script_event_queue.pending().is_empty()
        || !dev_console_queue.pending().is_empty()
        || !audio_command_queue.snapshot().is_empty()
        || !scene_command_queue.pending().is_empty()
    {
        return Err(AmigoError::Message(format!(
            "placeholder bridge exceeded the maximum of {MAX_PLACEHOLDER_BRIDGE_PASSES} orchestration passes"
        )));
    }

    summary.console_commands = dev_console_state.command_history();
    summary.console_output = dev_console_state.output_lines();

    Ok(summary)
}
