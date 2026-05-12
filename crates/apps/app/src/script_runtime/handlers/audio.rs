use super::super::super::*;
use super::super::AppScriptCommandContext;
use amigo_session::ScriptCommandHandler;

pub(super) struct AudioScriptCommandHandler;

impl<'a> ScriptCommandHandler<AppScriptCommandContext<'a>, ScriptCommand, ()>
    for AudioScriptCommandHandler
{
    fn name(&self) -> &'static str {
        "audio"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        command.namespace == "audio"
            && matches!(
                (command.name.as_str(), command.arguments.len()),
                ("preload", 1) | ("play", 1) | ("start-realtime", 1)
            )
    }

    fn handle(&self, ctx: &AppScriptCommandContext<'a>, command: ScriptCommand) {
        let outcome = amigo_audio_api::handle_audio_script_command(
            amigo_audio_api::AudioScriptCommandContext {
                audio_command_queue: ctx.audio_command_queue,
                audio_scene_service: ctx.audio_scene_service,
            },
            command.clone(),
            |name| crate::app_helpers::resolve_mod_audio_asset_key(ctx.launch_selection, name),
        );

        match outcome {
            amigo_audio_api::AudioScriptCommandOutcome::Preloaded { asset_key, mode } => {
                crate::app_helpers::register_audio_clip_reference(
                    ctx.asset_catalog,
                    ctx.audio_scene_service,
                    &asset_key,
                    mode,
                );
                ctx.dev_console_state
                    .write_line(format!("preloaded audio clip `{}`", asset_key.as_str()));
            }
            amigo_audio_api::AudioScriptCommandOutcome::PlayOnce { asset_key } => {
                crate::app_helpers::register_audio_clip_reference(
                    ctx.asset_catalog,
                    ctx.audio_scene_service,
                    &asset_key,
                    AudioPlaybackMode::OneShot,
                );
                ctx.dev_console_state
                    .write_line(format!("queued audio one-shot `{}`", asset_key.as_str()));
            }
            amigo_audio_api::AudioScriptCommandOutcome::CueQueued { cue_name, clip } => {
                ctx.dev_console_state.write_line(format!(
                    "queued audio cue `{}` as one-shot `{}`",
                    cue_name,
                    clip.as_str()
                ));
            }
            amigo_audio_api::AudioScriptCommandOutcome::CueMissing { cue_name } => {
                ctx.dev_console_state
                    .write_line(format!("unknown audio cue `{cue_name}`"));
            }
            amigo_audio_api::AudioScriptCommandOutcome::CueNotReady { .. } => {}
            amigo_audio_api::AudioScriptCommandOutcome::SourceStarted { source, asset_key } => {
                crate::app_helpers::register_audio_clip_reference(
                    ctx.asset_catalog,
                    ctx.audio_scene_service,
                    &asset_key,
                    AudioPlaybackMode::Looping,
                );
                ctx.dev_console_state.write_line(format!(
                    "queued realtime audio source `{}` using `{}`",
                    source,
                    asset_key.as_str()
                ));
            }
            amigo_audio_api::AudioScriptCommandOutcome::SourceStopped { source } => {
                ctx.dev_console_state
                    .write_line(format!("queued stop for audio source `{source}`"));
            }
            amigo_audio_api::AudioScriptCommandOutcome::ParamSet { .. } => {}
            amigo_audio_api::AudioScriptCommandOutcome::VolumeSet { bus, value } => {
                ctx.dev_console_state.write_line(format!(
                    "queued audio bus volume `{bus}` = {}",
                    value.clamp(0.0, 1.0)
                ));
            }
            amigo_audio_api::AudioScriptCommandOutcome::ParseError { message } => {
                ctx.dev_console_state.write_line(message);
            }
            amigo_audio_api::AudioScriptCommandOutcome::Unhandled => {
                ctx.dev_console_state.write_line(format!(
                "{} could not handle command: {}",
                self.name(),
                crate::app_helpers::format_script_command(&command)
            ));
            }
        }
    }
}



