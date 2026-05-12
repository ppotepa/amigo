use amigo_assets::AssetKey;
use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scripting_api::ScriptCommand;
use amigo_scripting_api::RuntimeScriptCommandHandler;
use crate::{
    AudioClipKey, AudioCommand, AudioCommandQueue, AudioPlaybackMode, AudioSceneService,
    AudioSourceId,
};

pub struct AudioScriptCommandContext<'a> {
    pub audio_command_queue: &'a AudioCommandQueue,
    pub audio_scene_service: &'a AudioSceneService,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioScriptCommandOutcome {
    Preloaded { asset_key: AssetKey, mode: AudioPlaybackMode },
    PlayOnce { asset_key: AssetKey },
    CueQueued { cue_name: String, clip: AudioClipKey },
    CueMissing { cue_name: String },
    CueNotReady { cue_name: String },
    SourceStarted { source: String, asset_key: AssetKey },
    SourceStopped { source: String },
    ParamSet { source: String, param: String, value: f32 },
    VolumeSet { bus: String, value: f32 },
    ParseError { message: String },
    Unhandled,
}

pub fn can_handle_audio_script_command(command: &ScriptCommand) -> bool {
    command.namespace == "audio"
}

pub fn handle_audio_script_command(
    ctx: AudioScriptCommandContext<'_>,
    command: ScriptCommand,
    resolve_asset: impl Fn(&str) -> AssetKey,
) -> AudioScriptCommandOutcome {
    match (command.name.as_str(), command.arguments.as_slice()) {
        ("preload", [clip_name]) => {
            let asset_key = resolve_asset(clip_name);
            AudioScriptCommandOutcome::Preloaded {
                asset_key,
                mode: AudioPlaybackMode::OneShot,
            }
        }
        ("play", [clip_name]) => {
            let asset_key = resolve_asset(clip_name);
            ctx.audio_command_queue.push(AudioCommand::PlayOnce {
                clip: AudioClipKey::new(asset_key.as_str().to_owned()),
            });
            AudioScriptCommandOutcome::PlayOnce { asset_key }
        }
        ("play-asset", [asset_key]) => {
            let asset_key = AssetKey::new(asset_key.clone());
            ctx.audio_command_queue.push(AudioCommand::PlayOnce {
                clip: AudioClipKey::new(asset_key.as_str().to_owned()),
            });
            AudioScriptCommandOutcome::PlayOnce { asset_key }
        }
        ("cue", [cue_name]) => {
            let Some(cue) = ctx.audio_scene_service.cue(cue_name) else {
                return AudioScriptCommandOutcome::CueMissing {
                    cue_name: cue_name.clone(),
                };
            };
            if !ctx.audio_scene_service.mark_cue_played_if_ready(&cue) {
                return AudioScriptCommandOutcome::CueNotReady {
                    cue_name: cue_name.clone(),
                };
            }
            ctx.audio_command_queue.push(AudioCommand::PlayOnce {
                clip: cue.clip.clone(),
            });
            AudioScriptCommandOutcome::CueQueued {
                cue_name: cue.name,
                clip: cue.clip,
            }
        }
        ("start-realtime", [source]) => {
            let asset_key = resolve_asset(source);
            ctx.audio_command_queue.push(AudioCommand::StartSource {
                source: AudioSourceId::new(source.clone()),
                clip: AudioClipKey::new(asset_key.as_str().to_owned()),
            });
            AudioScriptCommandOutcome::SourceStarted {
                source: source.clone(),
                asset_key,
            }
        }
        ("stop", [source]) => {
            ctx.audio_command_queue.push(AudioCommand::StopSource {
                source: AudioSourceId::new(source.clone()),
            });
            AudioScriptCommandOutcome::SourceStopped {
                source: source.clone(),
            }
        }
        ("set-param", [source, param, value]) => match value.parse::<f32>() {
            Ok(value) => {
                ctx.audio_command_queue.push(AudioCommand::SetParam {
                    source: AudioSourceId::new(source.clone()),
                    param: param.clone(),
                    value,
                });
                AudioScriptCommandOutcome::ParamSet {
                    source: source.clone(),
                    param: param.clone(),
                    value,
                }
            }
            Err(error) => AudioScriptCommandOutcome::ParseError {
                message: format!("failed to parse audio param value `{value}` as f32: {error}"),
            },
        },
        ("set-volume", [bus, value]) => match value.parse::<f32>() {
            Ok(value) if bus == "master" => {
                ctx.audio_command_queue
                    .push(AudioCommand::SetMasterVolume { value });
                AudioScriptCommandOutcome::VolumeSet {
                    bus: bus.clone(),
                    value,
                }
            }
            Ok(value) => {
                ctx.audio_command_queue.push(AudioCommand::SetVolume {
                    bus: bus.clone(),
                    value,
                });
                AudioScriptCommandOutcome::VolumeSet {
                    bus: bus.clone(),
                    value,
                }
            }
            Err(error) => AudioScriptCommandOutcome::ParseError {
                message: format!("failed to parse audio volume `{value}` as f32: {error}"),
            },
        },
        _ => AudioScriptCommandOutcome::Unhandled,
    }
}

pub struct AudioScriptCommandHandler;

impl RuntimeScriptCommandHandler for AudioScriptCommandHandler {
    fn name(&self) -> &'static str {
        "audio"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        if command.namespace != "audio" {
            return false;
        }
        matches!(
            (command.name.as_str(), command.arguments.len()),
            ("play-asset", 1)
                | ("cue", 1)
                | ("stop", 1)
                | ("set-param", 3)
                | ("set-volume", 2)
        )
    }

    fn handle(&self, runtime: &Runtime, command: ScriptCommand) -> AmigoResult<()> {
        let audio_command_queue = runtime.required::<AudioCommandQueue>()?;
        let audio_scene_service = runtime.required::<AudioSceneService>()?;
        let _ = handle_audio_script_command(
            AudioScriptCommandContext {
                audio_command_queue: audio_command_queue.as_ref(),
                audio_scene_service: audio_scene_service.as_ref(),
            },
            command,
            |name| AssetKey::new(name.to_owned()),
        );
        Ok(())
    }
}
