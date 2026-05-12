use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{SceneCommand, format_scene_command};

use crate::{AudioClipKey, AudioCue, AudioSceneService};

pub struct AudioSceneCommandContext<'a> {
    pub audio_scene_service: &'a AudioSceneService,
}

pub struct AudioSceneCommandOutcome {
    pub name: String,
    pub clip: String,
    pub source_mod: String,
}

pub fn can_handle_audio_scene_command(command: &SceneCommand) -> bool {
    matches!(command, SceneCommand::QueueAudioCue { .. })
}

pub fn handle_audio_scene_command(
    ctx: AudioSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<AudioSceneCommandOutcome> {
    match command {
        SceneCommand::QueueAudioCue { command } => {
            ctx.audio_scene_service.register_cue(AudioCue::new(
                command.name.clone(),
                AudioClipKey::new(command.clip.as_str().to_owned()),
                command.min_interval,
            ));
            Ok(AudioSceneCommandOutcome {
                name: command.name,
                clip: command.clip.as_str().to_owned(),
                source_mod: command.source_mod,
            })
        }
        other => Err(AmigoError::Message(format!(
            "audio scene command handler cannot handle {}",
            format_scene_command(&other)
        ))),
    }
}
