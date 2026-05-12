use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use amigo_session::SceneCommandHandler;
use amigo_audio_api::AudioPlaybackMode;

pub(crate) struct SceneAudioCommandHandler;

impl<'a> SceneCommandHandler<AppSceneCommandContext<'a>, SceneCommand, AmigoResult<()>>
    for SceneAudioCommandHandler
{
    fn name(&self) -> &'static str {
        "scene-audio"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        amigo_audio_api::can_handle_audio_scene_command(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'a>, command: SceneCommand) -> AmigoResult<()> {
        if let SceneCommand::QueueAudioCue { command } = &command {
                crate::app_helpers::register_audio_clip_reference(
                    ctx.asset_catalog,
                    ctx.audio_scene_service,
                    &command.clip,
                    AudioPlaybackMode::OneShot,
                );
        }
        let outcome = amigo_audio_api::handle_audio_scene_command(
            amigo_audio_api::AudioSceneCommandContext {
                audio_scene_service: ctx.audio_scene_service,
            },
            command,
        )?;
        ctx.dev_console_state.write_line(format!(
            "queued audio cue `{}` -> `{}` from mod `{}`",
            outcome.name, outcome.clip, outcome.source_mod
        ));
        Ok(())
    }
}


