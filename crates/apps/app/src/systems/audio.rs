use std::collections::BTreeMap;

use super::super::*;
use crate::runtime_context::RuntimeContext;

pub(crate) fn tick_audio_runtime(runtime: &Runtime, delta_seconds: f32) -> AmigoResult<()> {
    let ctx = RuntimeContext::new(runtime);
    let asset_catalog = ctx.required::<AssetCatalog>()?;
    let audio_state_service = ctx.required::<AudioStateService>()?;
    let audio_mixer_service = ctx.required::<AudioMixerService>()?;
    let audio_output_service = ctx.required::<AudioOutputBackendService>()?;
    let dev_console_state = ctx.required::<DevConsoleState>()?;

    let prepared_assets = asset_catalog
        .prepared_assets()
        .into_iter()
        .map(|asset| (asset.key.as_str().to_owned(), asset))
        .collect::<BTreeMap<_, _>>();
    let playing_sources = audio_state_service.playing_sources();
    let source_params = audio_state_service.source_params();
    let frame_sample_count = ((44_100.0 * delta_seconds.max(0.0)).round() as usize).max(1);
    let mut deferred_commands = Vec::new();

    for command in audio_state_service.drain_runtime_commands() {
        if let AudioCommand::PlayOnce { clip } = command {
            if let Some(prepared_asset) = prepared_assets.get(clip.as_str()) {
                audio_state_service.clear_deferred_one_shot_logged(clip.as_str());
                if let Err(error) = audio_mixer_service
                    .queue_generated_one_shot(clip.as_str().to_owned(), prepared_asset)
                {
                    dev_console_state.write_line(format!(
                        "audio runtime queue failed for `{}`: {error}",
                        clip.as_str()
                    ));
                }
            } else {
                if audio_state_service.mark_deferred_one_shot_logged(clip.as_str()) {
                    dev_console_state.write_line(format!(
                        "audio deferred one-shot `{}` until asset is prepared",
                        clip.as_str()
                    ));
                }
                deferred_commands.push(AudioCommand::PlayOnce { clip });
            }
        }
    }

    for command in deferred_commands {
        audio_state_service.queue_runtime_command(command);
    }

    if let Some(frame) = audio_mixer_service.tick_generated_audio(
        &prepared_assets,
        &playing_sources,
        &source_params,
        audio_state_service.master_volume(),
        frame_sample_count,
    ) {
        if audio_state_service.mark_first_mix_frame_logged() {
            dev_console_state.write_line(format!(
                "audio mixed first frame: samples={} sources={}",
                frame.samples.len(),
                frame.sources.join(", ")
            ));
        }
        audio_output_service.enqueue_mix_frame(&frame);
    }

    Ok(())
}
