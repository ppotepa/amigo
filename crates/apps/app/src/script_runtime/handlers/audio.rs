use super::super::super::*;
use super::super::{AppScriptCommandContext, ScriptCommandHandler};

pub(super) struct AudioScriptCommandHandler;

impl ScriptCommandHandler for AudioScriptCommandHandler {
    fn name(&self) -> &'static str {
        "audio"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        matches!(command.namespace.as_str(), "audio")
    }

    fn handle(&self, ctx: &AppScriptCommandContext<'_>, command: ScriptCommand) {
        match (command.name.as_str(), command.arguments.as_slice()) {
            ("preload", [clip_name]) => {
                let asset_key = crate::app_helpers::resolve_mod_audio_asset_key(
                    ctx.launch_selection,
                    clip_name,
                );
                crate::app_helpers::register_audio_clip_reference(
                    ctx.asset_catalog,
                    ctx.audio_scene_service,
                    &asset_key,
                    AudioPlaybackMode::OneShot,
                );
                ctx.dev_console_state
                    .write_line(format!("preloaded audio clip `{}`", asset_key.as_str()));
            }
            ("play", [clip_name]) => {
                let asset_key = crate::app_helpers::resolve_mod_audio_asset_key(
                    ctx.launch_selection,
                    clip_name,
                );
                crate::app_helpers::register_audio_clip_reference(
                    ctx.asset_catalog,
                    ctx.audio_scene_service,
                    &asset_key,
                    AudioPlaybackMode::OneShot,
                );
                ctx.audio_command_queue.push(AudioCommand::PlayOnce {
                    clip: AudioClipKey::new(asset_key.as_str().to_owned()),
                });
                ctx.dev_console_state
                    .write_line(format!("queued audio one-shot `{}`", asset_key.as_str()));
            }
            ("play-asset", [asset_key]) => {
                let asset_key = AssetKey::new(asset_key.clone());
                crate::app_helpers::register_audio_clip_reference(
                    ctx.asset_catalog,
                    ctx.audio_scene_service,
                    &asset_key,
                    AudioPlaybackMode::OneShot,
                );
                ctx.audio_command_queue.push(AudioCommand::PlayOnce {
                    clip: AudioClipKey::new(asset_key.as_str().to_owned()),
                });
                ctx.dev_console_state
                    .write_line(format!("queued audio one-shot `{}`", asset_key.as_str()));
            }
            ("cue", [cue_name]) => {
                let Some(cue) = ctx.audio_scene_service.cue(cue_name) else {
                    ctx.dev_console_state
                        .write_line(format!("unknown audio cue `{cue_name}`"));
                    return;
                };
                if !ctx.audio_scene_service.mark_cue_played_if_ready(&cue) {
                    return;
                }
                ctx.audio_command_queue.push(AudioCommand::PlayOnce {
                    clip: cue.clip.clone(),
                });
                ctx.dev_console_state.write_line(format!(
                    "queued audio cue `{}` as one-shot `{}`",
                    cue.name,
                    cue.clip.as_str()
                ));
            }
            ("start-realtime", [source]) => {
                let asset_key =
                    crate::app_helpers::resolve_mod_audio_asset_key(ctx.launch_selection, source);
                crate::app_helpers::register_audio_clip_reference(
                    ctx.asset_catalog,
                    ctx.audio_scene_service,
                    &asset_key,
                    AudioPlaybackMode::Looping,
                );
                ctx.audio_command_queue.push(AudioCommand::StartSource {
                    source: AudioSourceId::new(source.clone()),
                    clip: AudioClipKey::new(asset_key.as_str().to_owned()),
                });
                ctx.dev_console_state.write_line(format!(
                    "queued realtime audio source `{}` using `{}`",
                    source,
                    asset_key.as_str()
                ));
            }
            ("stop", [source]) => {
                ctx.audio_command_queue.push(AudioCommand::StopSource {
                    source: AudioSourceId::new(source.clone()),
                });
                ctx.dev_console_state
                    .write_line(format!("queued stop for audio source `{source}`"));
            }
            ("set-param", [source, param, value]) => match value.parse::<f32>() {
                Ok(value) => {
                    ctx.audio_command_queue.push(AudioCommand::SetParam {
                        source: AudioSourceId::new(source.clone()),
                        param: param.clone(),
                        value,
                    });
                }
                Err(error) => ctx.dev_console_state.write_line(format!(
                    "failed to parse audio param value `{value}` as f32: {error}"
                )),
            },
            ("set-volume", [bus, value]) => match value.parse::<f32>() {
                Ok(value) if bus == "master" => {
                    ctx.audio_command_queue
                        .push(AudioCommand::SetMasterVolume { value });
                    ctx.dev_console_state.write_line(format!(
                        "queued master audio volume = {}",
                        value.clamp(0.0, 1.0)
                    ));
                }
                Ok(value) => {
                    ctx.audio_command_queue.push(AudioCommand::SetVolume {
                        bus: bus.clone(),
                        value,
                    });
                    ctx.dev_console_state.write_line(format!(
                        "queued audio bus volume `{bus}` = {}",
                        value.clamp(0.0, 1.0)
                    ));
                }
                Err(error) => ctx.dev_console_state.write_line(format!(
                    "failed to parse audio volume `{value}` as f32: {error}"
                )),
            },
            _ => ctx.dev_console_state.write_line(format!(
                "{} could not handle command: {}",
                self.name(),
                crate::app_helpers::format_script_command(&command)
            )),
        }
    }
}
