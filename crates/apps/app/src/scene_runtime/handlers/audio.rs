use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;
use amigo_audio_api::{AudioClipKey, AudioCue, AudioPlaybackMode};

pub(crate) struct SceneAudioCommandHandler;

impl SceneCommandHandler for SceneAudioCommandHandler {
    fn name(&self) -> &'static str {
        "scene-audio"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::QueueAudioCue { .. })
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueAudioCue { command } => {
                crate::app_helpers::register_audio_clip_reference(
                    ctx.asset_catalog,
                    ctx.audio_scene_service,
                    &command.clip,
                    AudioPlaybackMode::OneShot,
                );
                ctx.audio_scene_service.register_cue(AudioCue::new(
                    command.name.clone(),
                    AudioClipKey::new(command.clip.as_str().to_owned()),
                    command.min_interval,
                ));
                ctx.dev_console_state.write_line(format!(
                    "queued audio cue `{}` -> `{}` from mod `{}`",
                    command.name,
                    command.clip.as_str(),
                    command.source_mod
                ));
                Ok(())
            }
            _ => Err(AmigoError::Message(format!(
                "{} cannot handle command {}",
                self.name(),
                amigo_scene::format_scene_command(&command)
            ))),
        }
    }
}
