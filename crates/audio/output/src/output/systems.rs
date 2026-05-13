use std::collections::BTreeMap;

use amigo_assets::AssetCatalog;
use amigo_audio_api::{AudioCommand, AudioStateService};
use amigo_audio_mixer::AudioMixerService;
use amigo_core::{AmigoError, AmigoResult};
use amigo_runtime::Runtime;

fn required<T: Send + Sync + 'static>(runtime: &Runtime) -> AmigoResult<std::sync::Arc<T>> {
    runtime.resolve::<T>().ok_or_else(|| {
        AmigoError::Message(format!(
            "required service `{}` is not registered",
            std::any::type_name::<T>()
        ))
    })
}

pub fn tick_audio_runtime(runtime: &Runtime, delta_seconds: f32) -> AmigoResult<()> {
    let asset_catalog = required::<AssetCatalog>(runtime)?;
    let audio_state_service = required::<AudioStateService>(runtime)?;
    let audio_mixer_service = required::<AudioMixerService>(runtime)?;
    let audio_output_service = required::<AudioOutputBackendService>(runtime)?;

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
                let _ = audio_mixer_service
                    .queue_generated_one_shot(clip.as_str().to_owned(), prepared_asset);
            } else {
                let _ = audio_state_service.mark_deferred_one_shot_logged(clip.as_str());
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
        let _ = audio_state_service.mark_first_mix_frame_logged();
        audio_output_service.enqueue_mix_frame(&frame);
    }

    Ok(())
}

